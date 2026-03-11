//! Shared utility functions for spell systems.
//!
//! These functions are used across many spell implementations to avoid duplication.

use bevy::prelude::*;

use crate::game::components::OnGameplayScreen;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::units::components::{Health, Team};
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Returns the shortest XZ-plane distance from a point to a line segment defined by start/end.
pub(crate) fn distance_to_line_segment_xz(point: Vec3, start: Vec3, end: Vec3) -> f32 {
    let p = Vec2::new(point.x, point.z);
    let a = Vec2::new(start.x, start.z);
    let b = Vec2::new(end.x, end.z);
    let ab = b - a;
    let ap = p - a;
    let ab_len_sq = ab.length_squared();
    if ab_len_sq < 0.0001 {
        return ap.length();
    }
    let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);
    let closest = a + ab * t;
    (p - closest).length()
}

/// Resource representing a pending heal to be applied to the nearest injured defender.
///
/// Used by spells that deal damage and heal defenders as a side-effect (e.g., Void Siphon,
/// Siphon Life). The heal is deferred to a separate system to avoid query conflicts when
/// both the damage system and healing system need mutable access to Health components.
#[derive(Resource)]
pub(crate) struct PendingDefenderHeal {
    /// Total heal amount to apply.
    pub amount: f32,
    /// World-space origin position (for finding the nearest injured defender).
    pub origin: Vec3,
}

/// Applies a pending defender heal to the nearest injured defender, then removes the resource.
///
/// Finds the closest defender (by world-space distance to `origin`) that is alive but not
/// at full health, and heals them for the pending amount.
pub(crate) fn apply_pending_defender_heal(
    mut commands: Commands,
    pending: Option<Res<PendingDefenderHeal>>,
    mut defenders: Query<(Entity, &Transform, &mut Health, &Team), Without<Wizard>>,
) {
    let Some(heal) = pending else {
        return;
    };

    let mut best: Option<(Entity, f32)> = None;
    for (entity, transform, health, team) in defenders.iter() {
        if *team != Team::Defenders || health.current <= 0.0 || health.current >= health.max {
            continue;
        }
        let dist = transform.translation.distance(heal.origin);
        match best {
            None => best = Some((entity, dist)),
            Some((_, best_dist)) if dist < best_dist => best = Some((entity, dist)),
            _ => {}
        }
    }

    if let Some((entity, _)) = best {
        if let Ok((_, _, mut health, _)) = defenders.get_mut(entity) {
            health.heal(heal.amount);
        }
    }

    commands.remove_resource::<PendingDefenderHeal>();
}

/// Projects the cursor position onto the Y=0 ground plane via raycasting.
///
/// Returns the world-space intersection point of the camera ray through the cursor
/// with the horizontal ground plane (Y=0), or `None` if the cursor is not over the window,
/// the ray is parallel to the ground, or the intersection is behind the camera.
///
/// Uses the barrel-distortion-corrected cursor position so that raycasting
/// matches the visually distorted CRT output.
pub(crate) fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: &Res<CorrectedCursorPosition>,
) -> Option<Vec3> {
    let (camera, camera_transform) = camera_query.single().ok()?;
    let cursor_pos = corrected_cursor.0?;

    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;

    // Check if ray is parallel to ground plane
    if ray.direction.y.abs() < 0.0001 {
        return None;
    }

    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return None; // Intersection is behind camera
    }

    Some(ray.origin + ray.direction * t)
}

/// Clamps a target position to be within the wizard's spell range using 3D distance.
///
/// If the target is beyond `spell_range` from `wizard_pos`, it is moved along the
/// direction vector to sit exactly at `spell_range` distance.
pub(crate) fn clamp_to_spell_range(target: Vec3, wizard_pos: Vec3, spell_range: f32) -> Vec3 {
    let diff = target - wizard_pos;
    let distance = diff.length();

    if distance > spell_range {
        wizard_pos + diff.normalize() * spell_range
    } else {
        target
    }
}

/// Clamps a target position to be within the wizard's spell range on the ground plane,
/// accounting for the wizard's height above ground and an optional effect radius.
///
/// Uses the Pythagorean theorem to compute the maximum ground-plane radius from the
/// wizard's XZ position. If `effect_radius` is non-zero, the clamp ensures the entire
/// effect circle stays within range.
pub(crate) fn clamp_to_spell_range_ground(
    target: Vec3,
    wizard_pos: Vec3,
    spell_range: f32,
    effect_radius: f32,
) -> Vec3 {
    let wizard_height = wizard_pos.y;

    // Calculate max ground radius using Pythagorean theorem
    let max_ground_radius = if wizard_height < spell_range {
        (spell_range * spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    };

    // Account for effect radius so entire circle stays within range
    let max_center_distance = (max_ground_radius - effect_radius).max(0.0);

    // Calculate XZ plane distance from wizard to target
    let direction = target - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();

    if distance > max_center_distance && distance > 0.001 {
        let normalized_direction = direction / distance;
        wizard_pos + normalized_direction * max_center_distance
    } else {
        target
    }
}

/// Convenience wrapper that clamps an optional cursor position to spell range on the ground plane.
///
/// Returns `None` if `cursor_pos` is `None`, otherwise clamps the position using
/// [`clamp_to_spell_range_ground`] with `SPELL_ORIGIN` as the wizard position.
pub(crate) fn clamp_cursor_to_spell_range(
    cursor_pos: Option<Vec3>,
    spell_range: f32,
    effect_radius: f32,
) -> Option<Vec3> {
    let pos = cursor_pos?;
    Some(clamp_to_spell_range_ground(
        pos,
        crate::game::constants::SPELL_ORIGIN,
        spell_range,
        effect_radius,
    ))
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

/// Spawns a circle indicator entity on the ground plane.
///
/// Creates a unit circle mesh with the given material, positioned at the target location
/// on the ground plane at `circle_y_position`, scaled to `radius`. The caller must add
/// any spell-specific marker components to the returned entity.
///
/// Returns the `EntityCommands` so the caller can insert additional components.
pub(crate) fn spawn_circle_indicator<'a>(
    commands: &'a mut Commands,
    assets: &SpellVisualAssets,
    material: Handle<StandardMaterial>,
    position: Vec3,
    radius: f32,
    circle_y_position: f32,
) -> EntityCommands<'a> {
    commands.spawn((
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(material),
        Transform::from_translation(Vec3::new(position.x, circle_y_position, position.z))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(radius)),
        OnGameplayScreen,
    ))
}

/// Trait for circle indicator components that support pulse animation and position tracking.
///
/// Implement this trait on spell-specific indicator components to use
/// [`update_circle_indicator`] as a shared system helper.
pub(crate) trait CircleIndicator:
    Component<Mutability = bevy::ecs::component::Mutable>
{
    /// Returns a mutable reference to the position field.
    fn position(&self) -> Vec3;
    /// Returns the current time alive.
    fn time_alive(&self) -> f32;
    /// Sets the time alive value.
    fn set_time_alive(&mut self, time: f32);
    /// Returns the base radius for this indicator (before pulse).
    fn base_radius(&self) -> f32;
    /// Returns the Y position for the circle on the ground plane.
    fn circle_y_position(&self) -> f32;
    /// Returns the current pulse scale factor.
    fn pulse_scale(&self) -> f32;
}

/// Updates circle indicator transforms with pulse animation and position tracking.
///
/// This is a generic system helper that works with any component implementing [`CircleIndicator`].
pub(crate) fn update_circle_indicator<T: CircleIndicator>(
    time: Res<Time>,
    mut indicators: Query<(&mut T, &mut Transform)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        // Update time alive for pulse animation
        let new_time = indicator.time_alive() + time.delta_secs();
        indicator.set_time_alive(new_time);

        // Apply pulse scale
        let radius = indicator.base_radius();
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(radius * pulse);

        // Update position
        let pos = indicator.position();
        transform.translation.x = pos.x;
        transform.translation.y = indicator.circle_y_position();
        transform.translation.z = pos.z;
    }
}
