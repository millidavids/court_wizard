//! Meteor Fall spell systems.

use bevy::prelude::*;
use rand::Rng;

use super::components::{MeteorExplosion, MeteorFallStorm, MeteorGroundFire, MeteorProjectile};
use super::constants::*;
use crate::config::GameConfig;
use crate::game::components::{ConcentrationSpell, OnGameplayScreen};
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::OBSTACLE_BUFFER;
use crate::game::pathfinding::resources::PathfindingGrid;
use crate::game::terrain::messages::TerrainDamageMessage;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Health, Knockback, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_to_spell_range_ground, cleanup_spell_caster, spawn_circle_indicator,
    update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, clone_sphere_material, explosion_fade_opacity,
};
use crate::game::game_mode::components::ActiveToggles;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellEffectKind;

/// Talent configuration computed once from ActiveTalents.
struct MeteorTalentConfig {
    spawn_interval: f32,
    damage_mult: f32,
    explosion_radius_mult: f32,
    ground_fire_duration_mult: f32,
    ground_fire_damage_mult: f32,
    ground_fire_radius_mult: f32,
    storm_radius_mult: f32,
    tracking: bool,
    aftershock: bool,
    extinction_event: bool,
    volcanic_eruption: bool,
    mana_cost_mult: f32,
    mesh_radius_mult: f32,
}

impl Default for MeteorTalentConfig {
    fn default() -> Self {
        Self {
            spawn_interval: METEOR_SPAWN_INTERVAL,
            damage_mult: 1.0,
            explosion_radius_mult: 1.0,
            ground_fire_duration_mult: 1.0,
            ground_fire_damage_mult: 1.0,
            ground_fire_radius_mult: 1.0,
            storm_radius_mult: 1.0,
            tracking: false,
            aftershock: false,
            extinction_event: false,
            volcanic_eruption: false,
            mana_cost_mult: 1.0,
            mesh_radius_mult: 1.0,
        }
    }
}

fn compute_meteor_talent_config(active_talents: Option<&ActiveTalents>) -> MeteorTalentConfig {
    let t1 = active_talents.and_then(|t| t.get_selection(Spell::MeteorFall, 0));
    let t2 = active_talents.and_then(|t| t.get_selection(Spell::MeteorFall, 1));
    let t3 = active_talents.and_then(|t| t.get_selection(Spell::MeteorFall, 2));

    let mut cfg = MeteorTalentConfig::default();

    // Tier 1
    match t1 {
        Some(0) => {
            // Dense Barrage: spawn rate +30%
            cfg.spawn_interval = METEOR_SPAWN_INTERVAL / DENSE_BARRAGE_SPAWN_RATE_MULT;
        }
        Some(1) => {
            // Scorching Impact: explosion + ground fire damage +30%
            cfg.damage_mult *= SCORCHING_IMPACT_DAMAGE_MULT;
            cfg.ground_fire_damage_mult *= SCORCHING_IMPACT_DAMAGE_MULT;
        }
        Some(2) => {
            // Wide Devastation: storm radius +30%
            cfg.storm_radius_mult *= WIDE_DEVASTATION_RADIUS_MULT;
        }
        _ => {}
    }

    // Tier 2
    match t2 {
        Some(0) => {
            // Molten Core: ground fire duration 2x, damage +50%
            cfg.ground_fire_duration_mult *= MOLTEN_CORE_DURATION_MULT;
            cfg.ground_fire_damage_mult *= MOLTEN_CORE_DAMAGE_MULT;
        }
        Some(1) => {
            // Tracking Meteors
            cfg.tracking = true;
        }
        Some(2) => {
            // Aftershock
            cfg.aftershock = true;
        }
        _ => {}
    }

    // Tier 3
    match t3 {
        Some(0) => {
            // Extinction Event
            cfg.extinction_event = true;
        }
        Some(1) => {
            // Volcanic Eruption
            cfg.volcanic_eruption = true;
        }
        Some(2) => {
            // Meteor Shower: 3x spawn rate, reduced damage/radius, half mana
            cfg.spawn_interval /= METEOR_SHOWER_SPAWN_RATE_MULT;
            cfg.damage_mult *= METEOR_SHOWER_DAMAGE_MULT;
            cfg.explosion_radius_mult *= METEOR_SHOWER_RADIUS_MULT;
            cfg.ground_fire_radius_mult *= METEOR_SHOWER_RADIUS_MULT;
            cfg.mana_cost_mult *= METEOR_SHOWER_MANA_MULT;
            cfg.mesh_radius_mult *= METEOR_SHOWER_MESH_MULT;
        }
        _ => {}
    }

    cfg
}

/// Local wizard meteor fall casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_meteor_fall_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    existing_storms: Query<Entity, With<MeteorFallStorm>>,
    active_talents: Option<Res<ActiveTalents>>,
    target_assist: Res<TargetAssistWorldPos>,
) {
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::MeteorFall {
        return;
    }

    let talent_cfg = compute_meteor_talent_config(active_talents.as_deref());

    let completed = meteor_fall_casting_logic(
        &input,
        &time,
        wizard_entity,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut indicator_query,
        &existing_storms,
        &mut commands,
        &visual_assets,
        &mut meshes,
        &talent_cfg,
    );

    if completed {
        vfx::systems::spawn_school_flare(
            &mut commands,
            &visual_assets,
            vfx::systems::SpellSchool::Fire,
            time.elapsed_secs(),
        );
        mouse_state.left_consumed = true;
    }
}

/// Core meteor fall casting logic.
#[allow(clippy::too_many_arguments)]
fn meteor_fall_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut SpellCircleIndicator>,
    existing_storms: &Query<Entity, With<MeteorFallStorm>>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    meshes: &mut Assets<Mesh>,
    talent_cfg: &MeteorTalentConfig,
) -> bool {
    let mut completed = false;

    // Check for release event - cancel cast
    if input.just_released {
        cleanup_spell_caster(commands, wizard_entity, caster_query);
        casting_state.cancel();
        return false;
    }

    // Get cursor world position and clamp to wizard's spell range
    let Some(mut cursor_world_pos) = input.cursor_pos else {
        return false;
    };

    let wizard_pos = SPELL_ORIGIN;
    let scale = primed_spell.empowerment;
    let storm_radius = STORM_RADIUS * scale * talent_cfg.storm_radius_mult;
    let effective_mana_cost = MANA_COST * talent_cfg.mana_cost_mult;

    cursor_world_pos = clamp_to_spell_range_ground(
        cursor_world_pos,
        wizard_pos,
        wizard.spell_range,
        storm_radius,
    );

    // Handle casting based on state
    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(effective_mana_cost)
            {
                // Start casting - spawn circle indicator
                let circle_entity = spawn_circle_indicator(
                    commands,
                    meshes,
                    assets.meteor_fall_indicator.clone(),
                    cursor_world_pos,
                    storm_radius,
                )
                .id();
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));

                // Start the cast
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            // Currently casting - advance cast time
            casting_state.advance(time.delta_secs());

            // Update circle position to follow cursor
            update_indicator_position(
                wizard_entity,
                cursor_world_pos,
                caster_query,
                indicator_query,
            );

            // Check if cast is complete
            if casting_state.is_complete(primed_spell.cast_time) {
                // Cast complete - spawn storm entity
                if mana.consume(effective_mana_cost) {
                    // Despawn any existing storms (only one storm at a time)
                    for existing_storm in existing_storms.iter() {
                        commands.entity(existing_storm).try_despawn();
                    }

                    // Get final circle position and spawn storm
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            let mut storm = MeteorFallStorm::new(
                                indicator.position,
                                storm_radius,
                                primed_spell.empowerment,
                            );
                            // Apply talent config to storm
                            storm.spawn_interval = talent_cfg.spawn_interval;
                            storm.damage_mult = talent_cfg.damage_mult;
                            storm.explosion_radius_mult = talent_cfg.explosion_radius_mult;
                            storm.ground_fire_duration_mult = talent_cfg.ground_fire_duration_mult;
                            storm.ground_fire_damage_mult = talent_cfg.ground_fire_damage_mult;
                            storm.ground_fire_radius_mult = talent_cfg.ground_fire_radius_mult;
                            storm.tracking = talent_cfg.tracking;
                            storm.aftershock = talent_cfg.aftershock;
                            storm.extinction_event = talent_cfg.extinction_event;
                            storm.volcanic_eruption = talent_cfg.volcanic_eruption;
                            storm.mesh_radius_mult = talent_cfg.mesh_radius_mult;

                            // Extinction Event: fixed-duration cast, no concentration
                            if talent_cfg.extinction_event {
                                storm.duration = Some(EXTINCTION_STORM_DURATION);
                                commands.spawn((storm, OnGameplayScreen));
                            } else {
                                commands.spawn((
                                    storm,
                                    ConcentrationSpell {
                                        spell_name: "Meteor Fall",
                                        mana_cost: MANA_COST,
                                    },
                                    OnGameplayScreen,
                                ));
                            }
                        }

                        // Despawn circle indicator
                        commands.entity(indicator_entity).try_despawn();
                    }

                    // Remove caster marker immediately
                    commands.entity(wizard_entity).remove::<SpellCaster>();

                    // Return to resting state
                    casting_state.cancel();
                    completed = true;
                } else {
                    // Out of mana - cancel cast
                    cleanup_spell_caster(commands, wizard_entity, caster_query);
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            // Meteor Fall doesn't use channeling, cancel if we somehow get here
            cleanup_spell_caster(commands, wizard_entity, caster_query);
            casting_state.cancel();
        }
    }

    completed
}

/// Spawns meteor projectiles periodically from active storms.
///
/// Projectiles spawn at random positions within the storm radius, high above the battlefield.
pub(super) fn spawn_meteor_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut storms: Query<(Entity, &mut MeteorFallStorm)>,
    enemies: Query<(&Transform, &Team)>,
) {
    let rng = &mut game_rng.0;

    for (storm_entity, mut storm) in storms.iter_mut() {
        storm.update_timers(time.delta_secs());

        // Check if fixed-duration storm has expired (Extinction Event)
        if let Some(duration) = storm.duration
            && storm.time_alive >= duration
        {
            commands.entity(storm_entity).try_despawn();
            continue;
        }

        // Check if it's time to spawn another meteor
        if storm.time_since_spawn >= storm.spawn_interval {
            storm.reset_spawn_timer();

            // Random position within storm circle (on XZ plane)
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = rng.gen_range(0.0..storm.radius);
            let offset = Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);

            let spawn_pos = Vec3::new(
                storm.position.x + offset.x,
                METEOR_SPAWN_HEIGHT,
                storm.position.z + offset.z,
            );

            // Spawn projectile with talent-modified values
            let damage = METEOR_DAMAGE * storm.empowerment * storm.damage_mult;
            let explosion_radius =
                EXPLOSION_RADIUS * storm.empowerment * storm.explosion_radius_mult;
            let mesh_radius = METEOR_MESH_RADIUS * storm.mesh_radius_mult;

            let entity = spawn_meteor_projectile_entity(
                &mut commands,
                &visual_assets,
                spawn_pos,
                Vec3::new(0.0, METEOR_INITIAL_VELOCITY, 0.0),
                damage,
                explosion_radius,
                storm.empowerment,
                mesh_radius,
                MeteorProjectileTalentFlags {
                    aftershock: storm.aftershock,
                    volcanic_eruption: storm.volcanic_eruption,
                    ground_fire_duration_mult: storm.ground_fire_duration_mult,
                    ground_fire_damage_mult: storm.ground_fire_damage_mult,
                    ground_fire_radius_mult: storm.ground_fire_radius_mult,
                    tracking: storm.tracking,
                    is_extinction: false,
                },
            );

            // For tracking meteors, bias spawn position toward nearest enemy
            if storm.tracking {
                let storm_center_xz = Vec2::new(storm.position.x, storm.position.z);
                if let Some((enemy_xz, _)) = find_nearest_non_defender_xz(
                    enemies
                        .iter()
                        .map(|(t, team)| (Vec2::new(t.translation.x, t.translation.z), *team)),
                    storm_center_xz,
                    Some(storm.radius),
                ) {
                    // Bias 50% toward nearest enemy
                    let biased_x = spawn_pos.x * 0.5 + enemy_xz.x * 0.5;
                    let biased_z = spawn_pos.z * 0.5 + enemy_xz.y * 0.5;
                    commands.entity(entity).insert(
                        Transform::from_translation(Vec3::new(
                            biased_x,
                            METEOR_SPAWN_HEIGHT,
                            biased_z,
                        ))
                        .with_scale(Vec3::splat(mesh_radius)),
                    );
                }
            }
        }
    }
}

/// Talent flags to apply to a meteor projectile at spawn time.
pub(crate) struct MeteorProjectileTalentFlags {
    pub aftershock: bool,
    pub volcanic_eruption: bool,
    pub ground_fire_duration_mult: f32,
    pub ground_fire_damage_mult: f32,
    pub ground_fire_radius_mult: f32,
    pub tracking: bool,
    pub is_extinction: bool,
}

impl Default for MeteorProjectileTalentFlags {
    fn default() -> Self {
        Self {
            aftershock: false,
            volcanic_eruption: false,
            ground_fire_duration_mult: 1.0,
            ground_fire_damage_mult: 1.0,
            ground_fire_radius_mult: 1.0,
            tracking: false,
            is_extinction: false,
        }
    }
}

/// Spawns a raw meteor projectile entity with explicit parameters.
///
/// Used by both storm spawning and crystal absorption/auto-cast.
/// Pass `MeteorProjectileTalentFlags::default()` for non-talented projectiles.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_meteor_projectile_entity(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    spawn_pos: Vec3,
    velocity: Vec3,
    damage: f32,
    explosion_radius: f32,
    empowerment: f32,
    mesh_radius: f32,
    talent_flags: MeteorProjectileTalentFlags,
) -> Entity {
    let mut projectile =
        MeteorProjectile::new(velocity, damage, explosion_radius, empowerment, mesh_radius);
    projectile.aftershock = talent_flags.aftershock;
    projectile.volcanic_eruption = talent_flags.volcanic_eruption;
    projectile.ground_fire_duration_mult = talent_flags.ground_fire_duration_mult;
    projectile.ground_fire_damage_mult = talent_flags.ground_fire_damage_mult;
    projectile.ground_fire_radius_mult = talent_flags.ground_fire_radius_mult;
    projectile.tracking = talent_flags.tracking;
    projectile.is_extinction = talent_flags.is_extinction;

    commands
        .spawn((
            projectile,
            Mesh3d(assets.cross_plane_sphere.clone()),
            MeshMaterial3d(assets.meteor_projectile.clone()),
            Transform::from_translation(spawn_pos).with_scale(Vec3::splat(mesh_radius)),
            OnGameplayScreen,
        ))
        .id()
}

/// Spawns a meteor explosion visual entity.
fn spawn_explosion_entity(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
    pos: Vec3,
    radius: f32,
    damage: f32,
    networked: bool,
) {
    let mat_handle =
        clone_sphere_material(sphere_materials, &assets.fireball_explosion_sphere);

    let mut entity = commands.spawn((
        Mesh3d(assets.explosion_sphere.clone()),
        MeshMaterial3d(mat_handle),
        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
            .with_scale(Vec3::splat(0.1)),
        MeteorExplosion::new(pos, radius, damage),
        OnGameplayScreen,
    ));
    if networked {
        entity.insert(NetworkedSpellEffect {
            kind: SpellEffectKind::MeteorExplosion,
        });
    }
}

/// Updates meteor projectile physics - applies gravity and moves projectiles.
/// Also applies tracking force for Tracking Meteors talent.
pub(super) fn update_meteor_projectiles(
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
fn find_nearest_non_defender_xz(
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

/// Maximum height at which to spawn VFX (above this is off-screen).
const VFX_VISIBLE_HEIGHT: f32 = 1000.0;

/// Spawns smoke trail wisps and deferred glow for falling meteors (only when on-screen).
pub(super) fn spawn_meteor_smoke_trail(
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
pub(super) fn check_meteor_collisions(
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
        ),
        Without<MeteorProjectile>,
    >,
    active_toggles: Option<Res<ActiveToggles>>,
) {
    let scorched_mult = crate::game::game_mode::components::scorched_earth_mult(active_toggles.as_deref());
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
                for (unit_entity, unit_transform, mut health, mut temp_hp, has_spell_shield) in
                    &mut units
                {
                    let dx = unit_transform.translation.x - pos.x;
                    let dz = unit_transform.translation.z - pos.z;
                    let dist = (dx * dx + dz * dz).sqrt();
                    if dist <= AFTERSHOCK_RADIUS {
                        // Apply bonus damage
                        apply_spell_damage(
                            &mut commands,
                            unit_entity,
                            &mut health,
                            temp_hp.as_deref_mut(),
                            AFTERSHOCK_DAMAGE * projectile.empowerment,
                            DamageType::Fire,
                            has_spell_shield,
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

                        // Spawn eruption VFX (reuse explosion visual, scaled)
                        spawn_explosion_entity(
                            &mut commands,
                            &visual_assets,
                            &mut sphere_materials,
                            fire.origin,
                            VOLCANIC_ERUPTION_RADIUS,
                            eruption_damage,
                            false,
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
pub(super) fn process_extinction_event(
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

/// Updates explosion visuals and applies one-time impact damage.
/// Also tracks talent progress.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_meteor_explosions(
    time: Res<Time>,
    mut commands: Commands,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    mut explosions: Query<(
        Entity,
        &mut MeteorExplosion,
        &mut Transform,
        Option<&MeshMaterial3d<FireExplosionSphereMaterial>>,
    )>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        Without<MeteorExplosion>,
    >,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    mut terrain_damage: MessageWriter<TerrainDamageMessage>,
) {
    for (explosion_entity, mut explosion, mut transform, material_handle) in explosions.iter_mut() {
        explosion.time_alive += time.delta_secs();

        // Update visual scale (growth animation)
        let current_radius = explosion.current_radius(EXPLOSION_GROWTH_TIME);
        transform.scale = Vec3::splat(current_radius);

        // Fade out over the last portion of lifetime
        if let Some(handle) = material_handle
            && let Some(mat) = sphere_materials.get_mut(handle)
        {
            mat.opacity =
                explosion_fade_opacity(explosion.time_alive / EXPLOSION_LIFETIME);
        }

        // Apply damage once when explosion spawns
        if !explosion.damage_applied {
            explosion.damage_applied = true;

            terrain_damage.write(TerrainDamageMessage {
                position: explosion.origin,
                radius: explosion.max_radius,
                damage: explosion.damage,
                damage_type: DamageType::Fire,
            });

            let mut hit_count = 0u32;

            for (unit_entity, unit_transform, mut health, mut temp_hp, has_spell_shield) in
                units.iter_mut()
            {
                let distance = crate::game::units::wizard::spells::utils::xz_distance(
                    unit_transform.translation,
                    explosion.origin,
                );

                if distance <= explosion.max_radius {
                    apply_spell_damage(
                        &mut commands,
                        unit_entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        explosion.damage,
                        DamageType::Fire,
                        has_spell_shield,
                    );
                    hit_count += 1;
                }
            }

            // Track talent progress
            if hit_count > 0
                && let Some(ref mut progress) = talent_progress
            {
                progress.increment(Spell::MeteorFall, hit_count);
            }
        }

        // Despawn explosion after lifetime
        if explosion.time_alive >= EXPLOSION_LIFETIME {
            commands.entity(explosion_entity).try_despawn();
        }
    }
}

/// Spawns procedural fire particles rising off meteor ground fire pools.
pub(super) fn spawn_ground_fire_particles(
    mut commands: Commands,
    fires: Query<&MeteorGroundFire>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < GROUND_FIRE_SMOKE_INTERVAL {
        return;
    }
    *timer -= GROUND_FIRE_SMOKE_INTERVAL;

    let t = time.elapsed_secs();

    for fire in fires.iter() {
        // Don't emit smoke during the fade-out period
        let remaining = fire.duration - fire.time_alive;
        if remaining < GROUND_FIRE_FADE_DURATION {
            continue;
        }

        vfx::systems::spawn_fire_orange_smoke(
            &mut commands,
            &visual_assets,
            Vec3::new(fire.origin.x, 0.0, fire.origin.z),
            fire.radius,
            GROUND_FIRE_PARTICLE_COUNT,
            t,
        );
    }
}

/// Applies periodic fire damage to units standing in ground fire zones.
pub(super) fn apply_ground_fire_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut fires: Query<&mut MeteorGroundFire>,
    mut units: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
) {
    let delta = time.delta_secs();

    for mut fire in &mut fires {
        fire.time_alive += delta;
        fire.time_since_last_tick += delta;

        if fire.time_since_last_tick >= fire.tick_interval {
            fire.time_since_last_tick = 0.0;

            for (entity, transform, mut health, mut temp_hp, has_spell_shield) in &mut units {
                let dist = Vec3::new(
                    fire.origin.x - transform.translation.x,
                    0.0,
                    fire.origin.z - transform.translation.z,
                )
                .length();

                if dist <= fire.radius {
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        fire.damage_per_tick,
                        DamageType::Fire,
                        has_spell_shield,
                    );
                }
            }
        }
    }
}

/// Fades ground fire by scaling down as it approaches expiration.
pub(super) fn fade_ground_fire(mut fires: Query<(&MeteorGroundFire, &mut Transform)>) {
    for (fire, mut transform) in &mut fires {
        let remaining = fire.duration - fire.time_alive;
        if remaining < GROUND_FIRE_FADE_DURATION {
            let fade = (remaining / GROUND_FIRE_FADE_DURATION).max(0.0);
            let base_radius = fire.radius;
            transform.scale = Vec3::splat(base_radius * fade);
        }
    }
}

/// Cleans up expired ground fire zones and resets pathfinding costs.
pub(super) fn cleanup_ground_fire(
    mut commands: Commands,
    fires: Query<(Entity, &MeteorGroundFire)>,
    mut pathfinding: ResMut<PathfindingGrid>,
) {
    for (entity, fire) in &fires {
        if fire.time_alive >= fire.duration {
            // Reset terrain cost for the fire zone
            let origin_2d = Vec2::new(fire.origin.x, fire.origin.z);
            let buffered = fire.radius + OBSTACLE_BUFFER;
            let bounds = Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0));
            let shape = crate::game::pathfinding::ObstacleShape::circle(origin_2d, buffered);
            let cells = pathfinding.shape_filtered_cells(bounds, &shape);
            pathfinding.set_terrain_cost(&cells, 1.0);

            // Continuous flow field rebuilds will pick up the cost change automatically

            commands.entity(entity).try_despawn();
        }
    }
}
