use std::collections::HashSet;

use bevy::prelude::*;

use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Corpse, Health};
use crate::game::units::wizard::components::{CastingState, Mana, SpellCaster, WizardInput};

use super::reticle::{SpellCircleIndicator, spawn_circle_indicator};
use super::spell_math::xz_distance;

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

/// Pre-computed target-assist snap position, updated once per frame by
/// [`compute_target_assist`]. Spells read this via [`build_wizard_input`].
#[derive(Resource, Default)]
pub(crate) struct TargetAssistWorldPos(pub Option<Vec3>);

/// Each frame, finds the nearest living unit to the cursor and stores its position
/// if within the configured snap radius. When targeting_assistance is 0, clears the snap.
#[allow(clippy::too_many_arguments)]
pub(crate) fn compute_target_assist(
    mut assist: ResMut<TargetAssistWorldPos>,
    config: Res<crate::config::GameConfig>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    units: Query<&Transform, (With<Health>, Without<Corpse>)>,
) {
    let snap_radius = config.target_assist_snap_radius();
    if snap_radius <= 0.0 {
        assist.0 = None;
        return;
    }

    let Some(cursor_world) = get_cursor_world_position(&camera_query, &corrected_cursor) else {
        assist.0 = None;
        return;
    };

    let mut best_dist = snap_radius;
    let mut best_pos: Option<Vec3> = None;

    for transform in &units {
        let unit_pos = transform.translation;
        let dist = xz_distance(cursor_world, unit_pos);
        if dist < best_dist {
            best_dist = dist;
            best_pos = Some(Vec3::new(unit_pos.x, 0.0, unit_pos.z));
        }
    }

    assist.0 = best_pos;
}

/// Builds a `WizardInput` from mouse state and camera raycasting.
///
/// Every spell casting system constructs an identical `WizardInput` — this
/// centralizes that logic. When targeting assistance is active, snaps the
/// cursor to the nearest unit via [`TargetAssistWorldPos`].
pub(crate) fn build_wizard_input(
    mouse_left_released: &mut MessageReader<MouseLeftReleased>,
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: &Res<CorrectedCursorPosition>,
) -> WizardInput {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(camera_query, corrected_cursor);
    WizardInput {
        just_pressed: true, // Run conditions already ensure mouse is held
        pressed: true,
        just_released: released,
        cursor_pos,
    }
}

/// Applies targeting assistance snap to a `WizardInput`. Call after `build_wizard_input`.
pub(crate) fn apply_target_assist(input: &mut WizardInput, assist: &TargetAssistWorldPos) {
    if let Some(snap_pos) = assist.0
        && input.cursor_pos.is_some()
    {
        input.cursor_pos = Some(snap_pos);
    }
}

/// Cleans up the spell caster indicator and removes the `SpellCaster` component.
///
/// Called on release, completion, and channeling-cancel — identical logic every time.
pub(crate) fn cleanup_spell_caster(
    commands: &mut Commands,
    wizard_entity: Entity,
    caster_query: &Query<&SpellCaster>,
) {
    if let Ok(caster) = caster_query.get(wizard_entity) {
        if let Some(indicator_entity) = caster.indicator_entity {
            commands.entity(indicator_entity).try_despawn();
        }
        commands.entity(wizard_entity).remove::<SpellCaster>();
    }
}

/// Handles the mouse-release event during casting: cleans up indicator and cancels cast.
///
/// Returns `true` if released (caller should return early), `false` otherwise.
pub(crate) fn handle_spell_release(
    input: &WizardInput,
    commands: &mut Commands,
    wizard_entity: Entity,
    casting_state: &mut CastingState,
    caster_query: &Query<&SpellCaster>,
) -> bool {
    if input.just_released {
        cleanup_spell_caster(commands, wizard_entity, caster_query);
        casting_state.cancel();
        return true;
    }
    false
}

/// Updates the spell circle indicator position to track the cursor.
///
/// Used in the `CastingState::Casting` arm of every spell.
pub(crate) fn update_indicator_position(
    wizard_entity: Entity,
    cursor_pos: Vec3,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut SpellCircleIndicator>,
) {
    if let Ok(caster) = caster_query.get(wizard_entity)
        && let Some(indicator_entity) = caster.indicator_entity
        && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
    {
        indicator.position = cursor_pos;
    }
}

/// Spawns a spell circle indicator in the Resting state and starts the cast.
///
/// Returns `true` if the indicator was spawned (cast started), `false` otherwise.
#[allow(clippy::too_many_arguments)]
pub(crate) fn try_start_cast_with_indicator(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    indicator_material: Handle<StandardMaterial>,
    wizard_entity: Entity,
    casting_state: &mut CastingState,
    mana: &Mana,
    mana_cost: f32,
    cursor_pos: Vec3,
    indicator_radius: f32,
    caster_query: &Query<&SpellCaster>,
) -> bool {
    if caster_query.get(wizard_entity).is_err() && mana.can_afford(mana_cost) {
        let circle_entity = spawn_circle_indicator(
            commands,
            meshes,
            indicator_material,
            cursor_pos,
            indicator_radius,
        )
        .id();
        commands
            .entity(wizard_entity)
            .insert(SpellCaster::with_indicator(circle_entity));
        casting_state.start_cast();
        true
    } else {
        false
    }
}

/// Tracks unique entities hit by a persistent spell effect for talent progress.
///
/// Attach this component to any persistent spell entity (zone, beam, cloud, etc.)
/// that tracks talent progress. Call [`track_hit`] when a unit is damaged — it
/// returns `true` only for the first hit on each entity, preventing inflated
/// progress from repeated per-tick counting.
#[derive(Component, Default)]
pub(crate) struct UniqueHitTracker {
    hits: HashSet<Entity>,
}

impl UniqueHitTracker {
    /// Records a hit on the given entity. Returns `true` if this is the first
    /// time this entity has been hit by this spell instance.
    pub fn track_hit(&mut self, entity: Entity) -> bool {
        self.hits.insert(entity)
    }
}
