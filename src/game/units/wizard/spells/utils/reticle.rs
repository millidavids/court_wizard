use bevy::prelude::Annulus;
use bevy::prelude::*;

use crate::game::components::OnGameplayScreen;

/// Y position for all spell aiming reticles (hovering just above the ground).
pub(crate) const RETICLE_Y: f32 = 2.0;

/// Fixed world-space width of the reticle ring (in world units).
/// The annulus inner radius is computed per-spawn so this width stays constant
/// regardless of the spell's radius.
const RETICLE_RING_WIDTH: f32 = 3.0;

/// Resolution (number of segments) for the reticle annulus mesh.
const RETICLE_RING_RESOLUTION: u32 = 48;

/// Creates a reticle annulus mesh with a fixed world-space ring width for the given radius.
pub(crate) fn make_reticle_mesh(radius: f32) -> Mesh {
    let inner = (1.0 - RETICLE_RING_WIDTH / radius).max(0.0);
    Annulus::new(inner, 1.0)
        .mesh()
        .resolution(RETICLE_RING_RESOLUTION)
        .into()
}

/// Computes a standard pulse scale factor for circle indicators.
///
/// Returns a value oscillating around 1.0 with a small amplitude, creating
/// a subtle breathing/pulsing effect on the indicator circle.
pub(crate) fn indicator_pulse_scale(time_alive: f32) -> f32 {
    let pulse_freq = 2.0;
    let pulse_amplitude = 0.05;
    1.0 + (time_alive * pulse_freq * std::f32::consts::TAU).sin() * pulse_amplitude
}

/// Shared aiming reticle component for all spells.
///
/// Attached to the annulus ring entity spawned during casting. Tracks position
/// and effective radius; the shared [`update_spell_indicators`] system handles
/// pulse animation and transform updates.
#[derive(Component)]
pub(crate) struct SpellCircleIndicator {
    /// World-space position (XZ plane). Updated by spell casting systems.
    pub position: Vec3,
    /// Time this indicator has been alive (drives pulse animation).
    pub time_alive: f32,
    /// Pre-computed radius including empowerment and talent multipliers.
    pub effective_radius: f32,
}

impl SpellCircleIndicator {
    pub fn new(position: Vec3, effective_radius: f32) -> Self {
        Self {
            position,
            time_alive: 0.0,
            effective_radius,
        }
    }
}

/// Spawns a spell aiming reticle (annulus ring) on the ground plane.
///
/// The annulus mesh is generated per-spawn so that the ring width is constant
/// in world space regardless of the spell radius.
///
/// Inserts the shared [`SpellCircleIndicator`] component. The caller can chain
/// `.insert(...)` to add spell-specific marker components if needed.
pub(crate) fn spawn_circle_indicator<'a>(
    commands: &'a mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    position: Vec3,
    radius: f32,
) -> EntityCommands<'a> {
    let mesh = meshes.add(make_reticle_mesh(radius));
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(Vec3::new(position.x, RETICLE_Y, position.z))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(radius)),
        SpellCircleIndicator::new(position, radius),
        OnGameplayScreen,
    ))
}

/// Shared system that updates all spell aiming reticles: pulse animation and position tracking.
pub(crate) fn update_spell_indicators(
    time: Res<Time>,
    mut indicators: Query<(&mut SpellCircleIndicator, &mut Transform)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        indicator.time_alive += time.delta_secs();
        let pulse = indicator_pulse_scale(indicator.time_alive);
        transform.scale = Vec3::splat(indicator.effective_radius * pulse);
        transform.translation.x = indicator.position.x;
        transform.translation.y = RETICLE_Y;
        transform.translation.z = indicator.position.z;
    }
}
