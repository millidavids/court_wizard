//! Teleport VFX systems: warp effect lifecycle.

use bevy::prelude::*;

use super::components::DimensionalRift;
use super::vfx_components::TeleportWarpEffect;
use super::vfx_constants;
use crate::game::components::OnGameplayScreen;

/// Ticks one-shot warp effects and despawns them when expired.
/// Persistent effects (duration == 0.0) are cleaned up by `cleanup_rift_warp_effects`.
pub(crate) fn tick_warp_effects(
    mut commands: Commands,
    mut effects: Query<(Entity, &mut TeleportWarpEffect)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut warp) in effects.iter_mut() {
        warp.time_alive += dt;

        // One-shot: decay intensity and despawn when expired
        if warp.duration > 0.0 {
            let progress = (warp.time_alive / warp.duration).min(1.0);
            // Smooth decay: intensity fades from starting value to 0
            warp.intensity = vfx_constants::RIPPLE_INTENSITY * (1.0 - progress);

            if warp.time_alive >= warp.duration {
                commands.entity(entity).try_despawn();
            }
        }
    }
}

/// Cleans up persistent warp effects when their associated rift entity is despawned.
pub(crate) fn cleanup_rift_warp_effects(
    mut commands: Commands,
    effects: Query<(Entity, &TeleportWarpEffect)>,
    rifts: Query<Entity, With<DimensionalRift>>,
) {
    for (entity, warp) in effects.iter() {
        if let Some(rift_entity) = warp.rift_entity {
            if rifts.get(rift_entity).is_err() {
                commands.entity(entity).try_despawn();
            }
        }
    }
}

/// Spawns warp effects at two positions.
/// Used by both one-shot teleports and persistent Dimensional Rifts.
fn spawn_warp_effects(
    commands: &mut Commands,
    source_pos: Vec3,
    dest_pos: Vec3,
    radius: f32,
    duration: f32,
    intensity: f32,
    rift_entity: Option<Entity>,
) {
    // Warp at source
    commands.spawn((
        TeleportWarpEffect {
            position: source_pos,
            radius,
            time_alive: 0.0,
            duration,
            intensity,
            rift_entity,
        },
        OnGameplayScreen,
    ));

    // Warp at destination
    commands.spawn((
        TeleportWarpEffect {
            position: dest_pos,
            radius,
            time_alive: 0.0,
            duration,
            intensity,
            rift_entity,
        },
        OnGameplayScreen,
    ));
}

/// Spawns one-shot warp effects at teleport source and destination.
pub(crate) fn spawn_teleport_vfx(
    commands: &mut Commands,
    source_pos: Vec3,
    dest_pos: Vec3,
    radius: f32,
) {
    spawn_warp_effects(
        commands,
        source_pos,
        dest_pos,
        radius,
        vfx_constants::RIPPLE_DURATION,
        vfx_constants::RIPPLE_INTENSITY,
        None,
    );
}

/// Spawns persistent warp effects for a Dimensional Rift.
pub(crate) fn spawn_rift_vfx(
    commands: &mut Commands,
    rift_entity: Entity,
    source_pos: Vec3,
    dest_pos: Vec3,
    radius: f32,
) {
    spawn_warp_effects(
        commands,
        source_pos,
        dest_pos,
        radius,
        0.0, // persistent
        vfx_constants::RIFT_RIPPLE_INTENSITY,
        Some(rift_entity),
    );
}
