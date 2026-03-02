//! Systems for the Teleport spell.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::components::{TeleportCaster, TeleportDestinationCircle, TeleportSourceCircle};
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::{BATTLEFIELD_SIZE, SPELL_ORIGIN};
use crate::game::input::MouseButtonState;
use crate::game::input::messages::{MouseLeftReleased, MouseRightPressed};
use crate::game::units::components::Teleportable;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::resources::NetworkConnection;
use crate::game::units::wizard::spells::utils::{clamp_to_spell_range, get_cursor_world_position};

/// Result from teleport casting logic, used to communicate state back to the wrapper.
struct TeleportCastResult {
    /// Whether the spell completed (teleport executed).
    completed: bool,
    /// Whether the first phase was finalized (destination locked in on release).
    first_phase_released: bool,
    /// Teleport parameters for network sync: (source_x, source_z, dest_x, dest_z, radius).
    teleport_params: Option<(f32, f32, f32, f32, f32)>,
}

/// Handles right-click to cancel/reset the teleport spell.
///
/// This system runs independently of the main casting system to ensure
/// right-click always cancels, even when other conditions would block casting.
pub fn handle_teleport_cancel(
    mut mouse_right_pressed: MessageReader<MouseRightPressed>,
    mut commands: Commands,
    mut wizard_query: Query<(&mut CastingState, Entity), With<LocalWizard>>,
    mut caster_query: Query<&mut TeleportCaster, With<LocalWizard>>,
    mut mouse_state: ResMut<MouseButtonState>,
) {
    // Only process if right-click occurred
    if mouse_right_pressed.read().next().is_none() {
        return;
    }

    // Get wizard and caster
    let Ok((mut casting_state, wizard_entity)) = wizard_query.single_mut() else {
        return;
    };

    let mut caster = if let Ok(c) = caster_query.single_mut() {
        c
    } else {
        commands.entity(wizard_entity).insert(TeleportCaster::new());
        return;
    };

    // Despawn any active circles
    if let Some(dest_entity) = caster.destination_circle {
        commands.entity(dest_entity).despawn();
    }
    if let Some(source_entity) = caster.source_circle {
        commands.entity(source_entity).despawn();
    }

    // Reset all state
    caster.destination_circle = None;
    caster.destination_position = None;
    caster.source_circle = None;
    casting_state.cancel();
    mouse_state.left_consumed = true; // Prevent immediate restart if left button still held
}

/// Local wizard Teleport casting — reads mouse input, manages indicator circles.
#[allow(clippy::too_many_arguments)]
pub fn handle_teleport_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (
            Entity,
            &Wizard,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        (
            With<LocalWizard>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
        ),
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut caster_query: Query<&mut TeleportCaster>,
    mut destination_query: Query<
        (&mut Transform, &mut TeleportDestinationCircle),
        (
            With<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
        ),
    >,
    mut source_query: Query<
        (&mut Transform, &mut TeleportSourceCircle),
        (
            With<TeleportSourceCircle>,
            Without<TeleportDestinationCircle>,
        ),
    >,
    units_query: Query<
        (Entity, &Transform),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
        ),
    >,
    mut connection: Option<ResMut<NetworkConnection>>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true, // Run conditions ensure mouse is held
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Teleport {
        return;
    }

    // Safety check
    if mouse_state.left_consumed {
        return;
    }

    let mut caster = if let Ok(c) = caster_query.get_mut(wizard_entity) {
        c
    } else {
        commands.entity(wizard_entity).insert(TeleportCaster::new());
        return;
    };

    let clamped_pos = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range(pos, SPELL_ORIGIN, wizard.spell_range));

    let cast_result = teleport_casting_logic(
        &input,
        &time,
        clamped_pos,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut caster,
        &mut commands,
        &units_query,
        &source_query,
    );

    // === Local-only: manage indicator circles ===

    // Phase 1: Spawn/update destination crosshair
    if !caster.has_destination() {
        match *casting_state {
            CastingState::Resting => {
                // If anchor was just set (transition to Casting happened then was handled),
                // we may need to spawn crosshair. But actually shared logic handles start_cast.
                // Check: if casting just started and no crosshair exists, spawn it.
            }
            CastingState::Casting { .. } => {
                // Destination crosshair — spawn if needed, update position
                if caster.destination_circle.is_none() {
                    if let Some(pos) = clamped_pos {
                        let radius = primed_spell.scale(CROSSHAIR_RADIUS);

                        let crosshair_entity = commands
                            .spawn((
                                Mesh3d(visual_assets.unit_circle.clone()),
                                MeshMaterial3d(visual_assets.teleport_destination.clone()),
                                Transform::from_xyz(pos.x, 1.0, pos.z)
                                    .with_rotation(Quat::from_rotation_x(
                                        -std::f32::consts::FRAC_PI_2,
                                    ))
                                    .with_scale(Vec3::splat(radius)),
                                TeleportDestinationCircle::new(primed_spell.empowerment),
                                OnGameplayScreen,
                            ))
                            .id();

                        caster.destination_circle = Some(crosshair_entity);
                    }
                } else if let Some(circle_entity) = caster.destination_circle
                    && let Ok((mut transform, _)) = destination_query.get_mut(circle_entity)
                    && let Some(pos) = clamped_pos
                {
                    transform.translation.x = pos.x;
                    transform.translation.z = pos.z;
                }
            }
            _ => {}
        }
    } else {
        // Phase 2: Spawn/update source circle
        if let CastingState::Casting { elapsed } = *casting_state {
            if caster.source_circle.is_none() {
                if let Some(pos) = clamped_pos {
                    let circle_entity = commands
                        .spawn((
                            Mesh3d(visual_assets.unit_circle.clone()),
                            MeshMaterial3d(visual_assets.teleport_source.clone()),
                            Transform::from_xyz(pos.x, 1.0, pos.z)
                                .with_rotation(Quat::from_rotation_x(
                                    -std::f32::consts::FRAC_PI_2,
                                ))
                                .with_scale(Vec3::ZERO),
                            TeleportSourceCircle::new(pos, primed_spell.empowerment),
                            OnGameplayScreen,
                        ))
                        .id();

                    caster.source_circle = Some(circle_entity);
                }
            } else if let Some(circle_entity) = caster.source_circle
                && let Ok((mut transform, mut indicator)) = source_query.get_mut(circle_entity)
                && let Some(pos) = clamped_pos
            {
                transform.translation.x = pos.x;
                transform.translation.z = pos.z;

                let growth = (elapsed / SECOND_CAST_TIME).min(1.0);
                let radius = CIRCLE_RADIUS * indicator.empowerment;
                transform.scale = Vec3::splat(radius * growth);

                indicator.position = pos;
                indicator.time_alive += time.delta_secs();
            }
        }
    }

    // Cleanup circles on completion or first-phase release
    if cast_result.completed {
        if let Some(dest_entity) = caster.destination_circle {
            commands.entity(dest_entity).despawn();
        }
        if let Some(source_entity) = caster.source_circle {
            commands.entity(source_entity).despawn();
        }
        caster.destination_circle = None;
        caster.source_circle = None;
        mouse_state.left_consumed = true;

        // Send teleport params over the network so the host can move units
        if let Some((source_x, source_z, dest_x, dest_z, radius)) = cast_result.teleport_params
            && let Some(ref mut conn) = connection
        {
            conn.outgoing_messages.push(
                crate::networking::protocol::NetworkMessage::TeleportUnits {
                    source_x,
                    source_z,
                    dest_x,
                    dest_z,
                    radius,
                },
            );
        }
    }

    if cast_result.first_phase_released {
        mouse_state.left_consumed = true;
    }
}

/// Core Teleport casting logic — called by the local casting system.
///
/// Handles the two-phase state machine:
/// Phase 1: Click to start casting, release to lock destination position.
/// Phase 2: Click again to start source circle growth, cast completes on timer or early release.
#[allow(clippy::too_many_arguments)]
fn teleport_casting_logic(
    input: &WizardInput,
    time: &Time,
    clamped_pos: Option<Vec3>,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    _primed_spell: &PrimedSpell,
    caster: &mut TeleportCaster,
    commands: &mut Commands,
    units_query: &Query<
        (Entity, &Transform),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
        ),
    >,
    source_query: &Query<
        (&mut Transform, &mut TeleportSourceCircle),
        (
            With<TeleportSourceCircle>,
            Without<TeleportDestinationCircle>,
        ),
    >,
) -> TeleportCastResult {
    let mut result = TeleportCastResult {
        completed: false,
        first_phase_released: false,
        teleport_params: None,
    };

    // Handle release during first cast — finalize destination position
    if input.just_released
        && !caster.has_destination()
        && matches!(*casting_state, CastingState::Casting { .. })
    {
        if let Some(pos) = clamped_pos {
            caster.destination_position = Some(pos);
            casting_state.cancel(); // Return to resting for phase 2
            result.first_phase_released = true;
        }
        return result;
    }

    // Handle release during second cast — completes teleport early
    if input.just_released && caster.has_destination() && caster.source_circle.is_some()
        && let CastingState::Casting { elapsed } = *casting_state
    {
        if let Some(source_entity) = caster.source_circle
            && let Ok((transform, source_circle)) = source_query.get(source_entity)
        {
            let source_pos = transform.translation;
            let growth = (elapsed / SECOND_CAST_TIME).min(1.0);
            let scale = source_circle.empowerment;
            let current_radius = CIRCLE_RADIUS * scale * growth;

            if mana.can_afford(MANA_COST) {
                mana.consume(MANA_COST);

                if let Some(dest_pos) = caster.destination_position {
                    teleport_units_with_radius(
                        source_pos,
                        dest_pos,
                        current_radius,
                        units_query,
                        commands,
                    );
                    result.teleport_params = Some((
                        source_pos.x,
                        source_pos.z,
                        dest_pos.x,
                        dest_pos.z,
                        current_radius,
                    ));
                }

                caster.destination_position = None;
                casting_state.cancel();
                result.completed = true;
            }
        }
        return result;
    }

    let Some(_clamped_pos) = clamped_pos else {
        return result;
    };

    // State machine based on whether destination exists
    if !caster.has_destination() {
        // PHASE 1: Placing destination
        match *casting_state {
            CastingState::Resting => {
                if input.just_pressed || input.pressed {
                    casting_state.start_cast();
                }
            }
            CastingState::Casting { .. } => {
                // Position update handled by local wrapper
            }
            _ => {}
        }
    } else {
        // PHASE 2: Placing source circle and teleporting
        match *casting_state {
            CastingState::Resting => {
                if !mana.can_afford(MANA_COST) {
                    return result;
                }
                if input.just_pressed || input.pressed {
                    casting_state.start_cast();
                }
            }
            CastingState::Casting { ref mut elapsed } => {
                *elapsed += time.delta_secs();

                // Check if cast complete
                if *elapsed >= SECOND_CAST_TIME
                    && let Some(source_entity) = caster.source_circle
                    && let Ok((transform, source_circle)) = source_query.get(source_entity)
                {
                    let source_pos = transform.translation;
                    let radius = CIRCLE_RADIUS * source_circle.empowerment;

                    mana.consume(MANA_COST);

                    if let Some(dest_pos) = caster.destination_position {
                        teleport_units_with_radius(
                            source_pos,
                            dest_pos,
                            radius,
                            units_query,
                            commands,
                        );
                        result.teleport_params =
                            Some((source_pos.x, source_pos.z, dest_pos.x, dest_pos.z, radius));
                    }

                    caster.destination_position = None;
                    casting_state.cancel();
                    result.completed = true;
                }
            }
            _ => {}
        }
    }

    result
}

/// Teleports all units within a specified radius of the source center to random positions
/// within the same radius of the destination center.
pub(crate) fn teleport_units_with_radius(
    source_center: Vec3,
    dest_center: Vec3,
    radius: f32,
    units_query: &Query<
        (Entity, &Transform),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
        ),
    >,
    commands: &mut Commands,
) {
    let mut rng = rand::thread_rng();

    for (entity, transform) in units_query.iter() {
        // Check if unit is within source circle (XZ distance only)
        let diff_x = transform.translation.x - source_center.x;
        let diff_z = transform.translation.z - source_center.z;
        let distance = (diff_x * diff_x + diff_z * diff_z).sqrt();

        if distance <= radius {
            // Generate random position within destination circle
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let random_radius = rng.gen_range(0.0..radius);

            let offset_x = angle.cos() * random_radius;
            let offset_z = angle.sin() * random_radius;

            let new_x = dest_center.x + offset_x;
            let new_z = dest_center.z + offset_z;

            // Clamp to battlefield bounds
            let clamped_x = new_x.clamp(-BATTLEFIELD_SIZE / 2.0, BATTLEFIELD_SIZE / 2.0);
            let clamped_z = new_z.clamp(-BATTLEFIELD_SIZE / 2.0, BATTLEFIELD_SIZE / 2.0);

            // Keep original Y position and rotation
            let new_position = Vec3::new(clamped_x, transform.translation.y, clamped_z);

            let mut new_transform = *transform;
            new_transform.translation = new_position;

            commands.entity(entity).insert(new_transform);
        }
    }
}

/// Updates pulse animations for both destination and source circles.
pub fn update_circle_animations(
    time: Res<Time>,
    mut destination_query: Query<
        (&mut Transform, &mut TeleportDestinationCircle),
        Without<TeleportSourceCircle>,
    >,
    mut source_query: Query<(&mut Transform, &mut TeleportSourceCircle)>,
) {
    // Update destination circles
    for (mut transform, mut indicator) in &mut destination_query {
        indicator.time_alive += time.delta_secs();

        let radius = CROSSHAIR_RADIUS * indicator.empowerment;
        // Only apply pulse animation after growth is mostly complete
        if transform.scale.x >= radius * PULSE_THRESHOLD {
            let pulse = indicator.pulse_scale();
            transform.scale = Vec3::splat(radius * pulse);
        }
    }

    // Update source circles
    for (mut transform, mut indicator) in &mut source_query {
        indicator.time_alive += time.delta_secs();

        let radius = CIRCLE_RADIUS * indicator.empowerment;
        // Only apply pulse animation after growth is mostly complete
        if transform.scale.x >= radius * PULSE_THRESHOLD {
            let pulse = indicator.pulse_scale();
            transform.scale = Vec3::splat(radius * pulse);
        }
    }
}
