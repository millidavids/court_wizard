use super::super::casting::write_grease_obstacle;
use super::super::components::{
    GreaseIgnited, GreaseOilSlickDebuff, GreaseRegenerating, GreaseZone, GreaseZonePresenceTracker,
};
use super::super::constants;
use crate::game::pathfinding::{ObstacleChanged, ObstacleType};
use crate::game::units::components::Health;
use bevy::prelude::*;

pub fn fade_grease_zone(
    time: Res<Time>,
    mut zones: Query<(
        &GreaseZone,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
        Has<GreaseIgnited>,
        Has<GreaseRegenerating>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = time.elapsed_secs();

    for (zone, mut transform, material_handle, is_ignited, is_regenerating) in &mut zones {
        // Grow animation: scale from 0 to full radius over GROW_DURATION.
        // Skip if ignited or regenerating (time_alive may have been reset by talents).
        if !is_ignited && !is_regenerating && zone.time_alive < constants::GROW_DURATION {
            let grow_progress = (zone.time_alive / constants::GROW_DURATION).min(1.0);
            let grow_scale = 1.0 - (1.0 - grow_progress) * (1.0 - grow_progress);
            transform.scale = Vec3::splat(zone.radius * grow_scale);
        } else if transform.scale.x != zone.radius {
            transform.scale = Vec3::splat(zone.radius);
        }

        let remaining = zone.duration - zone.time_alive;
        let fade = if remaining < constants::FADE_DURATION {
            (remaining / constants::FADE_DURATION).max(0.0)
        } else {
            1.0
        };

        // Fade the grease base mesh with iridescent oil-sheen emissive.
        // With Mask mode, alpha < 0.01 makes pixels disappear entirely.
        if let Some(material) = materials.get_mut(material_handle) {
            let (r, g, b, a) = constants::GREASE_COLOR;
            material.base_color = Color::srgba(r, g, b, a * fade);

            // Iridescent sheen: slow cycling through oil-slick rainbow tones
            // Only when not ignited and not regenerating (those have their own visuals)
            if !is_ignited && !is_regenerating {
                let phase = zone.origin.x * 0.01 + zone.origin.z * 0.013;
                let sheen_r = 0.3 + 0.2 * (t * 1.3 + phase).sin();
                let sheen_g = 0.2 + 0.15 * (t * 1.7 + phase * 1.4).sin();
                let sheen_b = 0.25 + 0.2 * (t * 2.1 + phase * 0.7).cos();
                material.emissive = bevy::color::LinearRgba::new(
                    sheen_r * fade,
                    sheen_g * fade,
                    sheen_b * fade,
                    0.0,
                );
            } else if !is_ignited {
                material.emissive = bevy::color::LinearRgba::NONE;
            }
        }
    }
}

/// Cleans up expired grease zones. For Endless Oil, triggers regeneration instead of despawn.
pub fn cleanup_grease_zone(
    mut commands: Commands,
    zones: Query<
        (Entity, &GreaseZone, Has<GreaseIgnited>),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, zone, is_ignited) in &zones {
        if zone.time_alive >= zone.duration {
            if is_ignited {
                // Talent: Endless Oil — regenerate instead of despawning
                if zone.talent_params.endless_oil {
                    // Remove ignited state and start regeneration
                    commands.entity(entity).try_remove::<GreaseIgnited>();
                    commands
                        .entity(entity)
                        .try_insert(GreaseRegenerating::new());

                    // Downgrade pathfinding from hazard back to slow terrain
                    write_grease_obstacle(
                        zone.origin,
                        zone.radius,
                        ObstacleType::SlowTerrain(3.0),
                        &mut obstacle_events,
                    );
                    continue;
                }

                write_grease_obstacle(
                    zone.origin,
                    zone.radius,
                    ObstacleType::Removed,
                    &mut obstacle_events,
                );
            }
            commands.entity(entity).try_despawn();
        }
    }
}

/// Handles Endless Oil regeneration: ticks the regen timer and restores the zone to slippery state.
pub fn update_grease_regeneration(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<
        (Entity, &mut GreaseZone, &mut GreaseRegenerating),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    zone_materials: Query<&MeshMaterial3d<StandardMaterial>, With<GreaseZone>>,
) {
    let delta = time.delta_secs();
    for (entity, mut zone, mut regen) in &mut zones {
        regen.time_regenerating += delta;

        // Fade the grease mesh back in as it regenerates.
        let regen_progress =
            (regen.time_regenerating / constants::ENDLESS_OIL_REGEN_DURATION).min(1.0);
        if let Ok(mat_handle) = zone_materials.get(entity)
            && let Some(material) = materials.get_mut(mat_handle)
        {
            let (r, g, b, a) = constants::GREASE_COLOR;
            material.base_color = Color::srgba(r, g, b, a * regen_progress);
        }

        if regen.time_regenerating >= constants::ENDLESS_OIL_REGEN_DURATION {
            // Regeneration complete — restore to slippery state
            zone.time_alive = (zone.duration - constants::ENDLESS_OIL_EXTRA_DURATION).max(0.0);
            zone.time_since_last_tick = 0.0;
            commands.entity(entity).try_remove::<GreaseRegenerating>();
        }
    }
}

/// Cleans up Oil Slick debuffs and presence trackers when grease zones are despawned.
pub fn cleanup_grease_debuffs(
    mut commands: Commands,
    zones: Query<Entity, With<GreaseZone>>,
    #[allow(clippy::type_complexity)] mut tracked: Query<
        (
            Entity,
            &GreaseZonePresenceTracker,
            Option<&GreaseOilSlickDebuff>,
            Option<&mut Health>,
        ),
        Without<crate::game::multiplayer::components::GhostEntity>,
    >,
) {
    for (entity, tracker, oil_slick, mut health) in &mut tracked {
        // If the zone this tracker references no longer exists, clean up
        if zones.get(tracker.zone_entity).is_err() {
            commands
                .entity(entity)
                .try_remove::<GreaseZonePresenceTracker>();
            if let Some(debuff) = oil_slick {
                if let Some(ref mut health) = health {
                    health.spell_vulnerability =
                        (health.spell_vulnerability - debuff.vulnerability).max(0.0);
                }
                commands.entity(entity).try_remove::<GreaseOilSlickDebuff>();
            }
        }
    }
}
