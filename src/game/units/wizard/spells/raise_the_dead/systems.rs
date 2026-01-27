use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, PrimedSpell};
use super::components::*;
use super::constants::*;
use crate::game::components::{Acceleration, Billboard, Velocity};
use crate::game::constants::{DEFENDER_HITBOX_HEIGHT, UNIT_HEALTH, UNIT_MOVEMENT_SPEED};
use crate::game::input::events::MouseLeftReleased;
use crate::game::units::components::{
    AttackTiming, Corpse, Health, Hitbox, MovementSpeed, PermanentCorpse, RoughTerrain, Team,
    Teleportable,
};
use crate::game::units::infantry::components::Infantry;

/// Unit radius for infantry hitboxes (matches infantry/styles.rs::UNIT_RADIUS)
const UNIT_RADIUS: f32 = 8.0;

/// Handles Raise The Dead spell casting and channeling.
///
/// Left-click starts cast. Must hold for full cast time.
/// After cast completes, enters channeling state where corpses are resurrected continuously.
/// Only casts when Raise The Dead is the primed spell.
///
/// Note: Spell priming, input blocking, and mouse state checks are handled by run_if conditions.
#[allow(clippy::too_many_arguments)]
pub fn handle_raise_the_dead_casting(
    time: Res<Time>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<(&mut CastingState, &mut Mana, &PrimedSpell)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    corpse_query: Query<(Entity, &Transform, &Team), (With<Corpse>, Without<PermanentCorpse>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    let Ok((mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };

    // Check for release event - this is spell-specific logic
    if mouse_left_released.read().next().is_some() {
        // Cancel cast/channel on release
        casting_state.cancel();
        return;
    }

    // Mouse is held - handle casting or channeling based on state
    match *casting_state {
        CastingState::Channeling { .. } => {
            // Already channeling - advance channel time
            casting_state.advance_channel(time.delta_secs());

            // Check if enough time has passed to resurrect another corpse
            if casting_state.should_channel(
                INITIAL_CHANNEL_INTERVAL,
                MIN_CHANNEL_INTERVAL,
                CHANNEL_RAMP_TIME,
            ) {
                // Try to resurrect corpse if we have mana
                if mana.consume(MANA_COST_PER_CORPSE) {
                    // Find corpse near cursor
                    if let Some(cursor_pos) =
                        get_cursor_world_position(&camera_query, &window_query)
                    {
                        resurrect_nearest_corpse(
                            &mut commands,
                            cursor_pos,
                            &corpse_query,
                            &mut materials,
                            &material_query,
                        );
                        casting_state.reset_channel_interval();
                    }
                } else {
                    // Out of mana - cancel channeling
                    casting_state.cancel();
                }
            }
        }
        CastingState::Casting { .. } => {
            // Currently casting - advance cast time
            casting_state.advance(time.delta_secs());

            // Check if cast is complete
            if casting_state.is_complete(primed_spell.cast_time) {
                // Cast complete - transition to channeling and resurrect first corpse
                if mana.consume(MANA_COST_PER_CORPSE) {
                    if let Some(cursor_pos) =
                        get_cursor_world_position(&camera_query, &window_query)
                    {
                        resurrect_nearest_corpse(
                            &mut commands,
                            cursor_pos,
                            &corpse_query,
                            &mut materials,
                            &material_query,
                        );
                        casting_state.start_channeling();
                    }
                } else {
                    // Out of mana - cancel cast
                    casting_state.cancel();
                }
            }
        }
        CastingState::Resting => {
            // Not casting yet - start cast if we have mana
            if mana.can_afford(MANA_COST_PER_CORPSE) {
                casting_state.start_cast();
            }
        }
    }
}

/// Resurrects the nearest corpse to the target position.
///
/// Searches for corpses within RESURRECTION_RADIUS and resurrects the closest one.
/// Changes sprite to green, restores health and combat components, sets team to Undead.
fn resurrect_nearest_corpse(
    commands: &mut Commands,
    target_pos: Vec3,
    corpse_query: &Query<(Entity, &Transform, &Team), (With<Corpse>, Without<PermanentCorpse>)>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    material_query: &Query<&MeshMaterial3d<StandardMaterial>>,
) {
    // Find nearest corpse within radius
    if let Some((corpse_entity, corpse_transform, _)) = corpse_query
        .iter()
        .filter(|(_, transform, _)| {
            target_pos.distance(transform.translation) <= RESURRECTION_RADIUS
        })
        .min_by(|a, b| {
            let dist_a = target_pos.distance(a.1.translation);
            let dist_b = target_pos.distance(b.1.translation);
            dist_a.partial_cmp(&dist_b).unwrap()
        })
    {
        // Change sprite to green
        if let Ok(material_handle) = material_query.get(corpse_entity)
            && let Some(material) = materials.get_mut(&material_handle.0)
        {
            material.base_color = UNDEAD_COLOR; // Bright green
        }

        // Calculate upright position: bottom edge 1 unit above battlefield
        let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);
        let spawn_y = hitbox.height / 2.0 + 1.0;
        let upright_transform = Transform::from_xyz(
            corpse_transform.translation.x,
            spawn_y,
            corpse_transform.translation.z,
        );

        // Restore combat components but change team
        commands
            .entity(corpse_entity)
            .remove::<Corpse>()
            .remove::<RoughTerrain>()
            .insert(upright_transform) // Stand upright
            .insert(Team::Undead)
            .insert(Health::new(UNIT_HEALTH)) // Full health restoration
            .insert(Velocity::default())
            .insert(Acceleration::new())
            .insert(MovementSpeed::new(UNIT_MOVEMENT_SPEED * 0.5)) // Half speed
            .insert(AttackTiming::new())
            .insert(Billboard)
            .insert(hitbox) // Restore collision
            .insert(Infantry) // Add infantry marker for movement systems
            .insert(Teleportable) // Can be teleported
            .insert(RaisedUndead); // Marker for tracking
    }
}

/// Gets cursor position projected onto Y=0 plane (same as other spells).
///
/// Returns None if cursor is not in window or ray doesn't intersect Y=0 plane.
fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec3> {
    let (camera, camera_transform) = camera_query.single().ok()?;
    let window = window_query.single().ok()?;
    let cursor_pos = window.cursor_position()?;

    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;
    let t = -ray.origin.y / ray.direction.y;

    if t > 0.0 {
        Some(ray.origin + ray.direction * t)
    } else {
        None
    }
}
