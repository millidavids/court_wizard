use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, GuestWizard, Mana, PrimedSpell, Spell, LocalWizard, Wizard};
use super::components::*;
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::spell_commands::{GuestBeam, GuestCursorPosition, GuestInputState};
use crate::game::units::components::{
    Health, SpellDamaged, TemporaryHitPoints, apply_damage_to_unit,
};

/// Handles Finger of Death casting with left-click for both local and guest wizards.
///
/// Left-click starts cast (if mana > 0). Beam spawns immediately and grows during cast.
/// After 2s cast completes, beam fires instantly dealing massive damage.
/// Only casts when Finger of Death is the primed spell.
///
/// Note: Spell priming, input blocking, and mouse state checks are handled by run_if conditions.
#[allow(clippy::too_many_arguments)]
pub fn handle_finger_of_death_casting(
    time: Res<Time>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (Entity, &Transform, &mut CastingState, &Mana, &PrimedSpell, &Wizard, Option<&GuestWizard>),
        Or<(With<LocalWizard>, With<GuestWizard>)>,
    >,
    awaiting_release_query: Query<(), With<AwaitingFingerOfDeathRelease>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut local_beams: Query<(Entity, &mut FingerOfDeathBeam), Without<GuestBeam>>,
    mut guest_beams: Query<(Entity, &mut FingerOfDeathBeam), With<GuestBeam>>,
    guest_cursor: Option<Res<GuestCursorPosition>>,
    guest_input: Option<Res<GuestInputState>>,
) {
    let local_released = mouse_left_released.read().next().is_some();

    for (wizard_entity, wizard_transform, mut casting_state, mana, primed_spell, wizard, is_guest) in wizard_query.iter_mut() {
        if primed_spell.spell != Spell::FingerOfDeath { continue; }

        let is_guest = is_guest.is_some();
        let released = if is_guest {
            guest_input.as_ref().is_some_and(|i| i.just_released)
        } else {
            local_released
        };

        let wizard_pos = wizard_transform.translation;

        // Check for release event - this is spell-specific logic
        if released {
            // Remove awaiting release marker (allows next cast)
            commands
                .entity(wizard_entity)
                .remove::<AwaitingFingerOfDeathRelease>();

            // Cancel cast on release - despawn beam
            casting_state.cancel();

            // Despawn beams for this wizard
            if is_guest {
                for (beam_entity, _) in guest_beams.iter() {
                    commands.entity(beam_entity).despawn();
                }
            } else {
                for (beam_entity, _) in local_beams.iter() {
                    commands.entity(beam_entity).despawn();
                }
            }

            continue;
        }

        // Mouse is held - handle casting based on state
        match *casting_state {
            CastingState::Channeling { .. } => {
                // Finger of Death doesn't channel - just cancel
                casting_state.cancel();
            }
            CastingState::Casting { .. } => {
                // Currently casting - advance cast time
                casting_state.advance(time.delta_secs());

                // Get cursor position based on wizard type
                let cursor_pos = if is_guest {
                    guest_cursor.as_ref().and_then(|c| c.position)
                } else {
                    get_cursor_world_position(&camera_query, &window_query)
                };

                // Update beam position/direction to follow cursor
                if let Some(cursor_pos) = cursor_pos {
                    let beam_origin =
                        wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                    // Clamp target position to spell range
                    let to_target = cursor_pos - beam_origin;
                    let distance = to_target.length();
                    let clamped_target = if distance > wizard.spell_range {
                        beam_origin + to_target.normalize() * wizard.spell_range
                    } else {
                        cursor_pos
                    };

                    let direction = (clamped_target - beam_origin).normalize();
                    let beam_length = (clamped_target - beam_origin)
                        .length()
                        .min(constants::BEAM_LENGTH);

                    // Calculate cast progress
                    let cast_progress = (casting_state.progress(primed_spell.cast_time)).min(1.0);

                    // Update existing beam or spawn new one
                    if is_guest {
                        if let Some((_, mut beam)) = guest_beams.iter_mut().next() {
                            beam.origin = beam_origin;
                            beam.direction = direction;
                            beam.length = beam_length;
                            beam.cast_progress = cast_progress;
                            beam.time_alive += time.delta_secs();
                        } else {
                            let mut new_beam = FingerOfDeathBeam::new(
                                beam_origin,
                                direction,
                                beam_length,
                                primed_spell.empowerment,
                            );
                            new_beam.cast_progress = cast_progress;
                            spawn_beam_with_marker(&mut commands, &mut meshes, &mut materials, new_beam, true);
                        }
                    } else if let Some((_, mut beam)) = local_beams.iter_mut().next() {
                        beam.origin = beam_origin;
                        beam.direction = direction;
                        beam.length = beam_length;
                        beam.cast_progress = cast_progress;
                        beam.time_alive += time.delta_secs();
                    } else {
                        let mut new_beam = FingerOfDeathBeam::new(
                            beam_origin,
                            direction,
                            beam_length,
                            primed_spell.empowerment,
                        );
                        new_beam.cast_progress = cast_progress;
                        spawn_beam_with_marker(&mut commands, &mut meshes, &mut materials, new_beam, false);
                    }
                }
            }
            CastingState::Resting => {
                // Not casting - check if we're waiting for mouse release first
                // If so, don't start a new cast even if mana is full
                if awaiting_release_query.get(wizard_entity).is_ok() {
                    continue;
                }

                // For guest wizard, check that they have active input
                let has_input = if is_guest {
                    guest_input.as_ref().is_some_and(|i| i.just_pressed || i.pressed)
                } else {
                    true
                };

                // Check for 50% mana requirement before starting cast
                if has_input && mana.percentage() >= constants::MANA_REQUIREMENT_PERCENT {
                    casting_state.start_cast();

                    // Get cursor position based on wizard type
                    let cursor_pos = if is_guest {
                        guest_cursor.as_ref().and_then(|c| c.position)
                    } else {
                        get_cursor_world_position(&camera_query, &window_query)
                    };

                    // Spawn initial beam
                    if let Some(cursor_pos) = cursor_pos {
                        let beam_origin =
                            wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                        // Clamp target position to spell range
                        let to_target = cursor_pos - beam_origin;
                        let distance = to_target.length();
                        let clamped_target = if distance > wizard.spell_range {
                            beam_origin + to_target.normalize() * wizard.spell_range
                        } else {
                            cursor_pos
                        };

                        let direction = (clamped_target - beam_origin).normalize();
                        let beam_length = (clamped_target - beam_origin)
                            .length()
                            .min(constants::BEAM_LENGTH);

                        let beam = FingerOfDeathBeam::new(
                            beam_origin,
                            direction,
                            beam_length,
                            primed_spell.empowerment,
                        );
                        spawn_beam_with_marker(&mut commands, &mut meshes, &mut materials, beam, is_guest);
                    }
                }
            }
        }
    }
}

/// Gets the cursor position projected onto the battlefield surface (Y=0 plane).
fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec3> {
    let (camera, camera_transform) = camera_query.single().ok()?;
    let window = window_query.single().ok()?;
    let cursor_pos = window.cursor_position()?;

    // Create a ray from the camera through the cursor position
    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;

    // Intersect ray with Y=0 plane (battlefield surface)
    let t = -ray.origin.y / ray.direction.y;

    if t > 0.0 {
        Some(ray.origin + ray.direction * t)
    } else {
        None
    }
}

/// Spawns a Finger of Death beam entity with visual mesh and spiral particles.
/// If `is_guest` is true, adds `GuestBeam` marker to distinguish from local beams.
fn spawn_beam_with_marker(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    beam: FingerOfDeathBeam,
    is_guest: bool,
) {
    // Calculate midpoint for the beam billboard (full length from start)
    let midpoint = beam.origin + beam.direction * (beam.length / 2.0);

    // Create a rectangle mesh for the beam
    let rectangle = Rectangle::new(constants::BEAM_WIDTH, constants::BEAM_WIDTH);

    // Start with alpha 0 (invisible), will fade in during cast
    let initial_color = Color::srgba(
        constants::BEAM_COLOR_CASTING.to_srgba().red,
        constants::BEAM_COLOR_CASTING.to_srgba().green,
        constants::BEAM_COLOR_CASTING.to_srgba().blue,
        0.0, // Start invisible
    );

    let mut entity_commands = commands.spawn((
        beam,
        Mesh3d(meshes.add(rectangle)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: initial_color,
            unlit: true,
            alpha_mode: AlphaMode::Blend, // Enable alpha blending for transparency
            ..default()
        })),
        Transform::from_translation(midpoint),
        OnGameplayScreen,
    ));

    if is_guest {
        entity_commands.insert(GuestBeam);
    }
}

/// Applies Finger of Death damage when cast completes.
///
/// Checks beams where has_fired == false and cast_progress >= 1.0.
/// Applies 1000 damage instantly to all units along beam (hitscan).
/// Drains 50% of the casting wizard's mana and cancels casting state.
/// Adds AwaitingFingerOfDeathRelease component to prevent immediate recast.
pub fn apply_finger_of_death_damage(
    mut commands: Commands,
    mut mouse_state: ResMut<MouseButtonState>,
    mut beams: Query<&mut FingerOfDeathBeam>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<Wizard>,
    >,
    mut wizard_query: Query<
        (Entity, &mut Mana, &mut CastingState),
        Or<(With<LocalWizard>, With<GuestWizard>)>,
    >,
    walls: Query<&crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone>,
) {
    let mut any_fired = false;

    for mut beam in beams.iter_mut() {
        // Only apply damage if cast is complete and hasn't fired yet
        if beam.has_fired || beam.cast_progress < 1.0 {
            continue;
        }

        // Mark as fired
        beam.has_fired = true;
        any_fired = true;

        // Find nearest wall intersection to limit beam reach
        let beam_end = beam.origin + beam.direction * beam.length;
        let mut max_t = 1.0_f32;
        for wall in &walls {
            if let Some(t) = wall.line_segment_intersects(beam.origin, beam_end) {
                max_t = max_t.min(t);
            }
        }
        let effective_length = beam.length * max_t;

        // Apply damage to all units along beam (before wall)
        let beam_width = beam.beam_width();
        let damage = beam.damage();
        for (entity, transform, mut health, mut temp_hp) in targets.iter_mut() {
            if beam.contains_point(transform.translation, beam_width) {
                let proj = (transform.translation - beam.origin).dot(beam.direction);
                if proj <= effective_length {
                    apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), damage);
                    commands.entity(entity).insert(SpellDamaged);
                }
            }
        }
    }

    // Drain 50% mana and cancel casting state for whichever wizard(s) are actively casting
    if any_fired {
        for (wizard_entity, mut mana, mut casting_state) in wizard_query.iter_mut() {
            if !matches!(*casting_state, CastingState::Resting) {
                mana.current -= mana.max * constants::MANA_REQUIREMENT_PERCENT;
                mana.current = mana.current.max(0.0);
                casting_state.cancel();

                // Add awaiting release marker to prevent immediate recast
                commands.entity(wizard_entity).insert(AwaitingFingerOfDeathRelease);
            }
        }

        // Mark mouse hold as consumed to prevent immediate recast
        mouse_state.left_consumed = true;
    }
}

/// Updates Finger of Death beam visuals based on cast progress and fire state.
pub fn update_finger_of_death_beam_visuals(
    time: Res<Time>,
    mut beam_query: Query<(
        &mut FingerOfDeathBeam,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut beam, mut transform, material_handle) in beam_query.iter_mut() {
        // Update time_since_fired if beam has fired
        if beam.has_fired {
            beam.time_since_fired += time.delta_secs();
        }
        // Beam is always full length, doesn't grow
        let current_len = beam.length;

        // Update position to beam midpoint
        let midpoint = beam.origin + beam.direction * (current_len / 2.0);
        transform.translation = midpoint;

        // Calculate rotation to align the rectangle's Y axis with the beam direction
        let rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);
        transform.rotation = rotation;

        // Scale the mesh to match beam length
        let base_width = beam.beam_width();
        let scale_y = current_len / constants::BEAM_WIDTH;
        let scale_x = if beam.has_fired {
            beam.beam_width_fired() / constants::BEAM_WIDTH // Wider after fire
        } else {
            base_width / constants::BEAM_WIDTH // Normal width during cast
        };
        transform.scale = Vec3::new(scale_x, scale_y, 1.0);

        // Update material color and alpha based on fire state and cast progress
        if let Some(material) = materials.get_mut(&material_handle.0) {
            if beam.has_fired {
                // After fire: fade out from 100% to 0% over POST_FIRE_DURATION
                let fade_progress = beam.time_since_fired / constants::POST_FIRE_DURATION;
                let alpha = (1.0 - fade_progress).max(0.0); // 1.0 -> 0.0

                material.base_color = Color::srgba(
                    constants::BEAM_COLOR_FIRED.to_srgba().red,
                    constants::BEAM_COLOR_FIRED.to_srgba().green,
                    constants::BEAM_COLOR_FIRED.to_srgba().blue,
                    alpha,
                );
            } else {
                // During cast: fade in alpha from 0 to ALPHA_CASTING based on cast_progress
                let alpha = constants::ALPHA_CASTING * beam.cast_progress;
                material.base_color = Color::srgba(
                    constants::BEAM_COLOR_CASTING.to_srgba().red,
                    constants::BEAM_COLOR_CASTING.to_srgba().green,
                    constants::BEAM_COLOR_CASTING.to_srgba().blue,
                    alpha,
                );
            }
        }
    }
}

/// Cleans up Finger of Death beams after firing or cancellation.
///
/// Handles both local and guest wizard beams independently.
pub fn cleanup_finger_of_death_beams(
    mut commands: Commands,
    local_beams: Query<(Entity, &FingerOfDeathBeam), Without<GuestBeam>>,
    guest_beams: Query<(Entity, &FingerOfDeathBeam), With<GuestBeam>>,
    local_wizard_query: Query<&CastingState, (With<LocalWizard>, Without<GuestWizard>)>,
    guest_wizard_query: Query<&CastingState, (With<GuestWizard>, Without<LocalWizard>)>,
) {
    // Cleanup local wizard's beams
    let local_resting = local_wizard_query
        .single()
        .map(|state| matches!(state, CastingState::Resting))
        .unwrap_or(true);

    for (entity, beam) in local_beams.iter() {
        let should_despawn = if beam.has_fired {
            beam.time_since_fired >= constants::POST_FIRE_DURATION
        } else {
            local_resting
        };

        if should_despawn {
            commands.entity(entity).despawn();
        }
    }

    // Cleanup guest wizard's beams
    let guest_resting = guest_wizard_query
        .single()
        .map(|state| matches!(state, CastingState::Resting))
        .unwrap_or(true);

    for (entity, beam) in guest_beams.iter() {
        let should_despawn = if beam.has_fired {
            beam.time_since_fired >= constants::POST_FIRE_DURATION
        } else {
            guest_resting
        };

        if should_despawn {
            commands.entity(entity).despawn();
        }
    }
}
