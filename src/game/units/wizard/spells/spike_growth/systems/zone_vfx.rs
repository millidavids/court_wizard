use rand::Rng;

use super::super::components::{SpikeGrowthZone, SpikeStormProjectile};
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::components::Health;
use crate::game::units::wizard::spells::utils;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// Death Garden: grows zone radius over time and updates pathfinding obstacles.
pub fn update_death_garden(
    time: Res<Time>,
    mut zones: Query<&mut SpikeGrowthZone>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let delta = time.delta_secs();

    for mut zone in &mut zones {
        if !zone.talent_params.death_garden {
            continue;
        }

        let new_radius = zone.effective_radius();

        // Throttle obstacle updates to every 1 second
        zone.death_garden_obstacle_timer += delta;
        if zone.death_garden_obstacle_timer >= 1.0 {
            zone.death_garden_obstacle_timer = 0.0;
            let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
            let buffered_radius = new_radius + OBSTACLE_BUFFER;
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
                obstacle_type: ObstacleType::Hazard(15.0),
                shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
                rebuild: false,
            });
        }
    }
}

/// Spike Storm: launches projectiles at nearby enemies periodically.
pub fn spike_storm_volley(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<&mut SpikeGrowthZone>,
    targets: Query<(Entity, &Transform, &Health), Without<SpikeGrowthZone>>,
    visual_assets: Res<SpellVisualAssets>,
) {
    let delta = time.delta_secs();

    for mut zone in &mut zones {
        if !zone.talent_params.spike_storm {
            continue;
        }

        zone.spike_storm_timer += delta;
        if zone.spike_storm_timer < constants::SPIKE_STORM_INTERVAL {
            continue;
        }
        zone.spike_storm_timer = 0.0;

        let targeting_range = zone.effective_radius() * constants::SPIKE_STORM_RANGE_MULT;

        // Find nearest enemies
        let mut candidates: Vec<(Vec3, f32)> = targets
            .iter()
            .filter(|(_, _, health)| !health.is_dead())
            .filter_map(|(_, transform, _)| {
                let dist = utils::xz_distance(zone.origin, transform.translation);
                if dist <= targeting_range {
                    Some((transform.translation, dist))
                } else {
                    None
                }
            })
            .collect();

        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates.truncate(constants::SPIKE_STORM_MAX_TARGETS);

        let Some(ref material) = zone.spike_storm_material else {
            continue;
        };

        for (target_pos, _) in &candidates {
            let spawn_pos = Vec3::new(zone.origin.x, 3.0, zone.origin.z);
            let direction = Vec3::new(target_pos.x - spawn_pos.x, 0.0, target_pos.z - spawn_pos.z);
            let direction = if direction.length_squared() > 0.001 {
                direction.normalize()
            } else {
                Vec3::X
            };

            let max_lifetime = targeting_range / constants::SPIKE_STORM_PROJECTILE_SPEED + 0.5;

            commands.spawn((
                Mesh3d(visual_assets.cross_plane_sphere.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(spawn_pos)
                    .with_scale(Vec3::splat(constants::SPIKE_STORM_PROJECTILE_SCALE)),
                SpikeStormProjectile {
                    direction,
                    speed: constants::SPIKE_STORM_PROJECTILE_SPEED,
                    damage: constants::SPIKE_STORM_DAMAGE,
                    radius: constants::SPIKE_STORM_PROJECTILE_RADIUS,
                    time_alive: 0.0,
                    max_lifetime,
                },
                OnGameplayScreen,
            ));
        }
    }
}

/// Spawns animated green vine rings and red spike rings from active spike growth zones.
pub fn emit_spike_growth_rings(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut zones: Query<&mut SpikeGrowthZone>,
) {
    let delta = time.delta_secs();
    for mut zone in &mut zones {
        if zone.time_alive >= zone.effective_duration() {
            continue;
        }

        zone.ring_timer += delta;
        if zone.ring_timer < utils::RING_SPAWN_INTERVAL {
            continue;
        }
        zone.ring_timer -= utils::RING_SPAWN_INTERVAL;

        // Alternate between green vine rings and red spike rings
        let material = if game_rng.0.random::<f32>() < 0.35 {
            visual_assets.spike_growth_spike.clone()
        } else {
            visual_assets.spike_growth_vine.clone()
        };

        utils::spawn_ring_particle(
            &mut game_rng.0,
            &mut commands,
            visual_assets.entangle_vine_ring.clone(),
            material,
            zone.origin,
            zone.effective_radius(),
        );
    }
}

/// Despawns expired spike growth zones.
pub fn cleanup_spike_growth_zone(
    mut commands: Commands,
    zones: Query<(Entity, &SpikeGrowthZone)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, zone) in &zones {
        if zone.time_alive >= zone.effective_duration() {
            let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
            let buffered_radius = zone.effective_radius() + OBSTACLE_BUFFER;
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
                rebuild: false,
            });
            commands.entity(entity).try_despawn();
        }
    }
}
