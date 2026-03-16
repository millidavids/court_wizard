//! Components for teleport visual effects.

use bevy::prelude::*;

/// World-space distortion point that drives the screen-space ripple shader.
///
/// Spawned at source and destination when a teleport fires. The CRT effect
/// system projects these to screen UV each frame and writes to
/// `TeleportDistortionSettings`.
#[derive(Component)]
pub(crate) struct TeleportWarpEffect {
    /// World-space center of the distortion.
    pub position: Vec3,
    /// World-space radius (projected to screen-space for influence area).
    pub radius: f32,
    /// Seconds since this effect was spawned.
    pub time_alive: f32,
    /// Total duration before despawn (0.0 = persistent, managed externally).
    pub duration: f32,
    /// Current intensity (decays over duration for one-shot effects).
    pub intensity: f32,
    /// The rift entity this warp is attached to (for cleanup when rift despawns).
    pub rift_entity: Option<Entity>,
}
