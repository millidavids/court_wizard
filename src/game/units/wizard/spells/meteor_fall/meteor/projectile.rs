use super::super::casting::{
    MeteorProjectileTalentFlags, spawn_explosion_entity, spawn_meteor_projectile_entity,
};
use bevy::prelude::*;

use super::super::components::{MeteorFallStorm, MeteorGroundFire, MeteorProjectile};
use super::super::constants::*;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::game_mode::components::ActiveToggles;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::OBSTACLE_BUFFER;
use crate::game::pathfinding::resources::PathfindingGrid;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Health, Knockback, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets,
};
use crate::networking::session::MultiplayerSession;
use crate::networking::snapshot::SpellEffectKind;

/// Maximum height at which to spawn VFX (above this is off-screen).
const VFX_VISIBLE_HEIGHT: f32 = 1000.0;

pub(crate) fn update_meteor_projectiles(
    time: Res<Time>,
    mut projectiles: Query<(&mut Transform, &mut MeteorProjectile)>,
    enemies: Query<(&Transform, &Team), Without<MeteorProjectile>>,
) {
    let delta = time.delta_secs();

    for (mut transform, mut projectile) in projectiles.iter_mut() {
        // Apply gravity
        projectile.velocity.y += METEOR_GRAVITY * delta;

        // Apply tracking force toward nearest enemy (only when visible)
        if projectile.tracking && transform.translation.y <= VFX_VISIBLE_HEIGHT {
            let proj_xz = Vec2::new(transform.translation.x, transform.translation.z);
            if let Some((enemy_xz, _)) = find_nearest_non_defender_xz(
                enemies
                    .iter()
                    .map(|(t, team)| (Vec2::new(t.translation.x, t.translation.z), *team)),
                proj_xz,
                None,
            ) {
                let dir = (enemy_xz - proj_xz).normalize_or_zero();
                projectile.velocity.x += dir.x * TRACKING_FORCE * delta;
                projectile.velocity.z += dir.y * TRACKING_FORCE * delta;
            }
        }

        // Move projectile
        transform.translation += projectile.velocity * delta;
    }
}

/// Finds the nearest non-defender enemy on the XZ plane within an optional max radius.
/// Returns the enemy's XZ position and distance from origin.
pub(crate) fn find_nearest_non_defender_xz(
    enemies: impl Iterator<Item = (Vec2, Team)>,
    origin: Vec2,
    max_radius: Option<f32>,
) -> Option<(Vec2, f32)> {
    let mut nearest_dist = f32::MAX;
    let mut nearest_pos = None;
    for (enemy_xz, team) in enemies {
        if team == Team::Defenders {
            continue;
        }
        let dist = enemy_xz.distance(origin);
        if dist < nearest_dist && max_radius.is_none_or(|r| dist < r) && dist > 1.0 {
            nearest_dist = dist;
            nearest_pos = Some(enemy_xz);
        }
    }
    nearest_pos.map(|pos| (pos, nearest_dist))
}

/// Spawns smoke trail wisps and deferred glow for falling meteors (only when on-screen).
pub(crate) fn spawn_meteor_smoke_trail(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &Transform, &mut MeteorProjectile)>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    // Spawn deferred glow for meteors entering visible range (runs every frame)
    for (entity, transform, mut projectile) in projectiles.iter_mut() {
        if !projectile.has_glow && transform.translation.y <= VFX_VISIBLE_HEIGHT {
            projectile.has_glow = true;
            vfx::systems::spawn_fire_glow(
                &mut commands,
                &visual_assets,
                entity,
                transform.translation,
                projectile.mesh_radius,
                OnGameplayScreen,
            );
        }
    }

    // Smoke trail on timer
    *timer += time.delta_secs();
    if *timer < vfx::constants::SMOKE_SPAWN_INTERVAL {
        return;
    }
    *timer -= vfx::constants::SMOKE_SPAWN_INTERVAL;

    let t = time.elapsed_secs();

    for (_entity, transform, _projectile) in projectiles.iter() {
        if transform.translation.y > VFX_VISIBLE_HEIGHT {
            continue;
        }

        vfx::systems::spawn_fire_smoke_wisps(
            &mut commands,
            &visual_assets,
            transform.translation,
            vfx::constants::SMOKE_COUNT_PER_SPAWN,
            t,
            vfx::constants::SMOKE_LIFETIME,
            vfx::constants::SMOKE_SIZE,
            vfx::constants::SMOKE_RISE_SPEED,
            vfx::constants::SMOKE_SPREAD_SPEED,
        );

        vfx::systems::spawn_heat_shimmer(
            &mut commands,
            &visual_assets,
            transform.translation,
            1,
            t,
        );
    }
}

/// Checks for meteor collisions with the ground, spawns explosions and ground fires.
/// Also handles Aftershock knockback and Volcanic Eruption.
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_meteor_collisions(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    projectiles: Query<(Entity, &Transform, &MeteorProjectile)>,
    mut pathfinding: ResMut<PathfindingGrid>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    mut ground_fires: Query<&mut MeteorGroundFire>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        Without<MeteorProjectile>,
    >,
    active_toggles: Option<Res<ActiveToggles>>,
    session: Option<Res<MultiplayerSession>>,
) {
    let scorched_mult =
        crate::game::game_mode::components::scorched_earth_mult(active_toggles.as_deref());
    let caster_team = local_player_team(session.as_deref());
    for (entity, transform, projectile) in projectiles.iter() {
        let projectile_pos = transform.translation;

        // Check ground collision (Y <= 0)
        if projectile_pos.y <= 0.0 {
            let pos = Vec3::new(projectile_pos.x, 0.0, projectile_pos.z);
            let t = pos.x * 0.01 + pos.z * 0.01; // deterministic pseudo-time per position

            // Spawn explosion visual and damage
            spawn_explosion_entity(
                &mut commands,
                &visual_assets,
                &mut sphere_materials,
                pos,
                projectile.explosion_radius,
                projectile.damage,
                true,
            );

            // Impact sparks
            vfx::systems::spawn_fire_sparks(
                &mut commands,
                &visual_assets,
                pos,
                vfx::constants::SPARK_COUNT,
                t,
            );

            // Explosion smoke burst
            vfx::systems::spawn_explosion_smoke(&mut commands, &visual_assets, pos, t);

            // Heat shimmer burst at impact
            vfx::systems::spawn_heat_shimmer(
                &mut commands,
                &visual_assets,
                pos,
                vfx::constants::EXPLOSION_SHIMMER_COUNT,
                t,
            );
            vfx::systems::spawn_explosion_dark_smoke(&mut commands, &visual_assets, pos, t);

            // Impact sound (fireball explosion)
            audio::play_impact_sfx(&mut commands, &sfx.fireball_impact, pos, &game_config, &sfx);

            // Aftershock: knockback + bonus damage to nearby enemies
            if projectile.aftershock {
                for (
                    unit_entity,
                    unit_transform,
                    mut health,
                    mut temp_hp,
                    has_spell_shield,
                    team,
                ) in &mut units
                {
                    let dx = unit_transform.translation.x - pos.x;
                    let dz = unit_transform.translation.z - pos.z;
                    let dist = (dx * dx + dz * dz).sqrt();
                    if dist <= AFTERSHOCK_RADIUS {
                        // Apply bonus damage
                        apply_spell_damage_with_team(
                            &mut commands,
                            unit_entity,
                            &mut health,
                            temp_hp.as_deref_mut(),
                            AFTERSHOCK_DAMAGE * projectile.empowerment,
                            DamageType::Fire,
                            has_spell_shield,
                            caster_team,
                            *team,
                        );
                        // Apply knockback
                        let direction = if dist > 0.1 {
                            Vec3::new(dx, 0.0, dz)
                        } else {
                            Vec3::X
                        };
                        commands.entity(unit_entity).insert(Knockback::new(
                            direction,
                            AFTERSHOCK_KNOCKBACK_SPEED,
                            AFTERSHOCK_KNOCKBACK_DURATION,
                        ));
                    }
                }
            }

            // Volcanic Eruption: check nearby ground fires and trigger eruption
            if projectile.volcanic_eruption {
                for mut fire in ground_fires.iter_mut() {
                    let fire_dx = fire.origin.x - pos.x;
                    let fire_dz = fire.origin.z - pos.z;
                    let fire_dist = (fire_dx * fire_dx + fire_dz * fire_dz).sqrt();
                    if fire_dist <= VOLCANIC_ERUPTION_RADIUS {
                        fire.eruption_charges += 1;
                        let eruption_damage = (VOLCANIC_ERUPTION_BASE_DAMAGE
                            + VOLCANIC_ERUPTION_STACK_BONUS * fire.eruption_charges as f32)
                            * projectile.empowerment;

                        // Spawn eruption VFX (reuse explosion visual, scaled).
                        // `networked: true` so the remote MP peer sees the
                        // mini-explosion via the existing MeteorExplosion
                        // SpellEffectKind path (visual + damage on both peers
                        // via CRDT). Without this, Volcanic Eruption damage
                        // was the caster's-peer-only.
                        spawn_explosion_entity(
                            &mut commands,
                            &visual_assets,
                            &mut sphere_materials,
                            fire.origin,
                            VOLCANIC_ERUPTION_RADIUS,
                            eruption_damage,
                            true,
                        );

                        // Eruption smoke
                        vfx::systems::spawn_explosion_smoke(
                            &mut commands,
                            &visual_assets,
                            fire.origin,
                            t,
                        );

                        // Only trigger eruption on the first matching fire zone
                        break;
                    }
                }
            }

            // Spawn ground fire hazard (only if empowered — boss meteors skip this)
            if projectile.empowerment > 0.0 {
                let fire_radius = GROUND_FIRE_RADIUS
                    * projectile.empowerment
                    * projectile.ground_fire_radius_mult;
                let fire_damage = GROUND_FIRE_DAMAGE
                    * projectile.empowerment
                    * projectile.ground_fire_damage_mult;
                let fire_duration =
                    GROUND_FIRE_DURATION * projectile.ground_fire_duration_mult * scorched_mult;

                let mut ground_fire = MeteorGroundFire::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    fire_radius,
                    fire_damage,
                    GROUND_FIRE_TICK,
                    fire_duration,
                );
                ground_fire.is_extinction = projectile.is_extinction;

                commands.spawn((
                    Transform::from_translation(Vec3::new(pos.x, 0.5, pos.z))
                        .with_scale(Vec3::splat(fire_radius)),
                    Visibility::default(),
                    ground_fire,
                    NetworkedSpellEffect {
                        kind: SpellEffectKind::MeteorGroundFire,
                    },
                    OnGameplayScreen,
                ));

                // Mark fire zone in pathfinding base_costs so future rebuilds avoid it
                let origin_2d = Vec2::new(pos.x, pos.z);
                let buffered = fire_radius + OBSTACLE_BUFFER;
                let bounds = Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0));
                let shape = crate::game::pathfinding::ObstacleShape::circle(origin_2d, buffered);
                let cells = pathfinding.shape_filtered_cells(bounds, &shape);
                pathfinding.set_terrain_cost(&cells, 8.0);

                // Continuous flow field rebuilds will pick up the cost change automatically
            }

            // Despawn the projectile
            commands.entity(entity).try_despawn();
        }
    }
}

/// Processes the Extinction Event: after 5s of channeling, spawns one massive meteor.
pub(crate) fn process_extinction_event(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut storms: Query<&mut MeteorFallStorm>,
) {
    for mut storm in storms.iter_mut() {
        if storm.extinction_event && !storm.extinction_fired && storm.time_alive >= EXTINCTION_DELAY
        {
            storm.extinction_fired = true;

            // Spawn massive meteor at storm center
            let spawn_pos = Vec3::new(storm.position.x, METEOR_SPAWN_HEIGHT, storm.position.z);
            let damage = EXTINCTION_DAMAGE * storm.empowerment * storm.damage_mult;
            let explosion_radius = storm.radius; // Covers entire storm area

            // Scale ground fire radius so the fire covers the entire storm area
            // Fire formula: GROUND_FIRE_RADIUS * empowerment * mult → we want storm.radius
            let extinction_fire_mult = storm.radius / (GROUND_FIRE_RADIUS * storm.empowerment);
            spawn_meteor_projectile_entity(
                &mut commands,
                &visual_assets,
                spawn_pos,
                Vec3::new(0.0, METEOR_INITIAL_VELOCITY, 0.0),
                damage,
                explosion_radius,
                storm.empowerment,
                EXTINCTION_MESH_RADIUS,
                MeteorProjectileTalentFlags {
                    aftershock: storm.aftershock,
                    volcanic_eruption: storm.volcanic_eruption,
                    ground_fire_duration_mult: storm.ground_fire_duration_mult,
                    ground_fire_damage_mult: storm.ground_fire_damage_mult,
                    ground_fire_radius_mult: extinction_fire_mult,
                    tracking: false,
                    is_extinction: true,
                },
            );
        }
    }
}
