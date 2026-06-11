use super::super::casting::wall_obstacle_bounds;
use super::super::components::{
    FirestormMarked, InsideWallOfFire, ScorchedEarthZone, SearingHeatDebuff, WallOfFireEffect,
    WallOfFireSfx,
};
use super::super::constants;
use super::super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::terrain::messages::TerrainDamageMessage;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Health, ResidualFireDamaged, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::utils::UniqueHitTracker;
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;
use bevy::prelude::*;

/// Interval between smoke wisp spawns for wall of fire.
const WALL_SMOKE_INTERVAL: f32 = 0.25;

/// Applies periodic fire damage to all units within the wall's rectangular area.
/// Also marks units as InsideWallOfFire for talent tracking and applies Searing Heat.
#[allow(clippy::too_many_arguments)]
pub fn apply_wall_of_fire_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(&mut WallOfFireEffect, &mut UniqueHitTracker)>,
    mut targets: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        Has<InsideWallOfFire>,
        Option<&SearingHeatDebuff>,
        &Team,
    )>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    mut terrain_damage: MessageWriter<TerrainDamageMessage>,
    session: Option<Res<MultiplayerSession>>,
) {
    use crate::game::units::wizard::components::Spell;

    let delta = time.delta_secs();
    let caster_team = local_player_team(session.as_deref());

    for (mut effect, mut hit_tracker) in &mut effects {
        effect.time_alive += delta;
        effect.time_since_last_tick += delta;

        if effect.time_since_last_tick >= effect.tick_interval {
            effect.time_since_last_tick = 0.0;

            let tick_damage = effect.effective_damage();
            let mut units_hit = 0u32;

            // Broadcast terrain damage covering the wall's footprint (bounding circle at midpoint).
            let midpoint = (effect.start + effect.end) * 0.5;
            let half_length = effect.start.distance(effect.end) * 0.5;
            terrain_damage.write(TerrainDamageMessage {
                position: midpoint,
                radius: half_length + effect.half_width,
                damage: tick_damage,
                damage_type: DamageType::Fire,
            });

            for (
                entity,
                transform,
                mut health,
                mut temp_hp,
                has_spell_shield,
                is_inside,
                searing,
                team,
            ) in &mut targets
            {
                let distance = effect.distance_to_point(transform.translation);

                if distance <= effect.half_width {
                    apply_spell_damage_with_team(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        tick_damage,
                        DamageType::Fire,
                        has_spell_shield,
                        caster_team,
                        *team,
                    );
                    commands.entity(entity).insert(ResidualFireDamaged);

                    // Mark unit as inside wall for Spreading Flames / Searing Heat tracking
                    if !is_inside {
                        commands.entity(entity).insert(InsideWallOfFire);
                    }

                    // Firestorm: mark unit so it explodes on death (even after leaving)
                    if effect.talent_params.firestorm {
                        commands.entity(entity).insert(FirestormMarked);
                    }

                    // Searing Heat: apply healing reduction debuff
                    if effect.talent_params.searing_heat && searing.is_none() {
                        health.healing_reduction += constants::SEARING_HEAT_HEALING_REDUCTION;
                        commands
                            .entity(entity)
                            .insert(SearingHeatDebuff(constants::SEARING_HEAT_HEALING_REDUCTION));
                    }

                    if hit_tracker.track_hit(entity) {
                        units_hit += 1;
                    }
                }
            }

            if units_hit > 0
                && let Some(ref mut progress) = talent_progress
            {
                progress.increment(Spell::WallOfFire, units_hit);
            }
        }
    }
}

/// Despawns wall of fire effects that have expired.
/// If Scorched Earth talent is active, spawns a slow zone in its place.
pub fn cleanup_wall_of_fire(
    mut commands: Commands,
    effects: Query<(Entity, &WallOfFireEffect)>,
    mut materials: ResMut<Assets<StandardMaterial>>, // For scorched earth zones
    visual_assets: Res<SpellVisualAssets>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, effect) in &effects {
        if effect.time_alive >= effect.duration {
            // Scorched Earth: leave behind a slow zone
            if effect.talent_params.scorched_earth {
                let wall_dir = (effect.end - effect.start).normalize_or_zero();
                let wall_len = effect.start.distance(effect.end);
                let center = effect.start + wall_dir * (wall_len / 2.0);
                let rotation = Quat::from_rotation_arc(Vec3::X, wall_dir);

                commands.spawn((
                    Mesh3d(visual_assets.unit_cuboid.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgba(0.15, 0.08, 0.02, 0.4),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        cull_mode: None,
                        ..default()
                    })),
                    Transform::from_xyz(center.x, 0.5, center.z)
                        .with_rotation(rotation)
                        .with_scale(Vec3::new(wall_len, 1.0, effect.half_width * 2.0)),
                    ScorchedEarthZone {
                        start: effect.start,
                        end: effect.end,
                        half_width: effect.half_width,
                        duration: constants::SCORCHED_EARTH_DURATION,
                        time_alive: 0.0,
                        tick_timer: 0.0,
                    },
                    OnGameplayScreen,
                ));
            }

            // Clear hazard from pathfinding (same bounds as when spawned)
            obstacle_events.write(ObstacleChanged {
                bounds: wall_obstacle_bounds(effect.start, effect.end, effect.half_width),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::obb_from_wall(
                    effect.start,
                    effect.end,
                    effect.half_width + OBSTACLE_BUFFER,
                )),
                rebuild: true,
            });

            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns thick orange fire smoke along the wall, plus black smoke and heat shimmer above.
pub fn spawn_wall_of_fire_smoke(
    mut commands: Commands,
    effects: Query<&WallOfFireEffect>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < WALL_SMOKE_INTERVAL {
        return;
    }
    *timer -= WALL_SMOKE_INTERVAL;

    let t = time.elapsed_secs();

    for effect in effects.iter() {
        // Don't emit smoke during the fade-out period
        let remaining = effect.duration - effect.time_alive;
        if remaining < FADE_DURATION {
            continue;
        }

        let wall_dir = (effect.end - effect.start).normalize_or_zero();
        let wall_len = effect.start.distance(effect.end);

        // Spawn orange fire smoke puffs at multiple points along the wall.
        // Each puff will automatically emit a black smoke puff at its apex.
        let num_points = ((wall_len / 40.0) as usize).max(3);
        for j in 0..num_points {
            let frac = (j as f32 + (t * 2.3 + j as f32 * 1.7).fract()) / num_points as f32;
            let pos = effect.start + wall_dir * (wall_len * frac.clamp(0.0, 1.0));

            vfx::systems::spawn_fire_orange_smoke(
                &mut commands,
                &visual_assets,
                pos,
                effect.half_width,
                3,
                t + j as f32,
            );
        }
    }
}

/// Despawns orphaned wall of fire sound effects whose parent no longer exists.
pub fn cleanup_wall_of_fire_sfx(
    mut commands: Commands,
    sfx_entities: Query<(Entity, &WallOfFireSfx)>,
    walls: Query<&WallOfFireEffect>,
) {
    for (entity, sfx) in sfx_entities.iter() {
        if walls.get(sfx.wall_entity).is_err() {
            commands.entity(entity).try_despawn();
        }
    }
}
