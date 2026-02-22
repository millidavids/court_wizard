use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, GuestWizard, Mana, PrimedSpell, Spell, LocalWizard, Wizard};
use super::components::DisintegrateBeam;
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::spell_commands::{GuestBeam, GuestCursorPosition, GuestInputState};
use crate::game::units::components::{Health, TemporaryHitPoints, apply_spell_damage};

/// Marker component for disintegrate spell when it's actively being cast/channeled.
///
/// This differentiates disintegrate from magic missile casting states.
#[derive(Component)]
pub struct DisintegrateCaster;

/// System that handles disintegrate beam casting for both local and guest wizards.
#[allow(clippy::too_many_arguments)]
pub fn handle_disintegrate_casting(
    time: Res<Time>,
    mut left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<
        (Entity, &Transform, &mut CastingState, &mut Mana, &PrimedSpell, &Wizard, Option<&GuestWizard>),
        Or<(With<LocalWizard>, With<GuestWizard>)>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut local_beams: Query<
        (Entity, &mut DisintegrateBeam),
        Without<GuestBeam>,
    >,
    mut guest_beams: Query<
        (Entity, &mut DisintegrateBeam),
        With<GuestBeam>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    guest_cursor: Option<Res<GuestCursorPosition>>,
    guest_input: Option<Res<GuestInputState>>,
) {
    let local_released = left_released.read().next().is_some();

    for (wizard_entity, wizard_transform, mut casting_state, mut mana, primed_spell, wizard, is_guest) in wizard_query.iter_mut() {
        if primed_spell.spell != Spell::Disintegrate { continue; }

        let is_guest = is_guest.is_some();
        let released = if is_guest {
            guest_input.as_ref().is_some_and(|i| i.just_released)
        } else {
            local_released
        };

        let wizard_pos = wizard_transform.translation;

        if released {
            casting_state.cancel();
            commands.entity(wizard_entity).remove::<DisintegrateCaster>();
            if is_guest {
                for (entity, _) in guest_beams.iter() {
                    commands.entity(entity).despawn();
                }
            } else {
                for (entity, _) in local_beams.iter() {
                    commands.entity(entity).despawn();
                }
            }
            continue;
        }

        match *casting_state {
            CastingState::Channeling { .. } => {
                casting_state.advance_channel(time.delta_secs());

                let mana_cost = constants::MANA_COST_PER_SECOND * time.delta_secs();

                if mana.consume(mana_cost) {
                    let target_pos = if is_guest {
                        guest_cursor.as_ref().and_then(|c| c.position)
                    } else {
                        get_cursor_world_position(&camera_query, &window_query)
                    };

                    if let Some(target_pos) = target_pos {
                        let beam_origin =
                            wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                        let to_target = target_pos - beam_origin;
                        let distance = to_target.length();
                        let clamped_target = if distance > wizard.spell_range {
                            beam_origin + to_target.normalize() * wizard.spell_range
                        } else {
                            target_pos
                        };

                        let direction = (clamped_target - beam_origin).normalize();
                        let beam_length = (clamped_target - beam_origin)
                            .length()
                            .min(constants::BEAM_LENGTH);

                        if is_guest {
                            if let Some((_, mut beam)) = guest_beams.iter_mut().next() {
                                beam.origin = beam_origin;
                                beam.direction = direction;
                                beam.length = beam_length;
                            } else {
                                spawn_beam_with_marker(
                                    &mut commands,
                                    &mut meshes,
                                    &mut materials,
                                    beam_origin,
                                    direction,
                                    beam_length,
                                    primed_spell.empowerment,
                                    true,
                                );
                            }
                        } else if let Some((_, mut beam)) = local_beams.iter_mut().next() {
                            beam.origin = beam_origin;
                            beam.direction = direction;
                            beam.length = beam_length;
                        } else {
                            spawn_beam_with_marker(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                beam_origin,
                                direction,
                                beam_length,
                                primed_spell.empowerment,
                                false,
                            );
                        }
                    }
                } else {
                    casting_state.cancel();
                    commands.entity(wizard_entity).remove::<DisintegrateCaster>();
                    if is_guest {
                        for (entity, _) in guest_beams.iter() {
                            commands.entity(entity).despawn();
                        }
                    } else {
                        for (entity, _) in local_beams.iter() {
                            commands.entity(entity).despawn();
                        }
                    }
                }
            }
            CastingState::Casting { .. } => {
                casting_state.advance(time.delta_secs());

                if casting_state.is_complete(primed_spell.cast_time) {
                    casting_state.start_channeling();

                    let target_pos = if is_guest {
                        guest_cursor.as_ref().and_then(|c| c.position)
                    } else {
                        get_cursor_world_position(&camera_query, &window_query)
                    };

                    if let Some(target_pos) = target_pos {
                        let beam_origin =
                            wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                        let to_target = target_pos - beam_origin;
                        let distance = to_target.length();
                        let clamped_target = if distance > wizard.spell_range {
                            beam_origin + to_target.normalize() * wizard.spell_range
                        } else {
                            target_pos
                        };

                        let direction = (clamped_target - beam_origin).normalize();
                        let beam_length = (clamped_target - beam_origin)
                            .length()
                            .min(constants::BEAM_LENGTH);

                        spawn_beam_with_marker(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            beam_origin,
                            direction,
                            beam_length,
                            primed_spell.empowerment,
                            is_guest,
                        );
                    }
                }
            }
            CastingState::Resting => {
                let has_input = if is_guest {
                    guest_input.as_ref().is_some_and(|i| i.just_pressed || i.pressed)
                } else {
                    true
                };
                if has_input && mana.can_afford(constants::MANA_COST_PER_SECOND * 0.1) {
                    casting_state.start_cast();
                    commands.entity(wizard_entity).insert(DisintegrateCaster);
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
    // Ray equation: origin + direction * t
    // Plane equation: y = 0
    // Solve for t: origin.y + direction.y * t = 0
    let t = -ray.origin.y / ray.direction.y;

    if t > 0.0 {
        Some(ray.origin + ray.direction * t)
    } else {
        None
    }
}

/// System that applies damage to all units hit by disintegrate beams.
///
/// This is a high-risk spell that damages both attackers and defenders,
/// but not the wizard.
pub fn apply_disintegrate_damage(
    mut commands: Commands,
    mut beam_query: Query<&mut DisintegrateBeam>,
    mut target_query: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<Wizard>,
    >,
    walls: Query<&crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone>,
    time: Res<Time>,
) {
    for mut beam in beam_query.iter_mut() {
        beam.update_damage_timer(time.delta_secs());
        beam.update_time_alive(time.delta_secs());

        // Find the nearest wall intersection to limit beam reach
        let beam_end = beam.origin + beam.direction * beam.current_length();
        let mut max_t = 1.0_f32;
        for wall in &walls {
            if let Some(t) = wall.line_segment_intersects(beam.origin, beam_end) {
                max_t = max_t.min(t);
            }
        }
        let effective_length = beam.current_length() * max_t;

        if beam.should_damage() {
            for (entity, transform, mut health, mut temp_hp) in target_query.iter_mut() {
                let position = transform.translation;
                // Check if point is in beam AND before the wall
                if beam.contains_point(position) {
                    let proj = (position - beam.origin).dot(beam.direction);
                    if proj <= effective_length {
                        apply_spell_damage(
                            &mut commands,
                            entity,
                            &mut health,
                            temp_hp.as_deref_mut(),
                            beam.damage_per_tick(),
                            constants::DAMAGE_TYPE,
                        );
                    }
                }
            }

            beam.reset_damage_timer();
        }
    }
}

/// System that despawns beams when wizard is not actively channeling disintegrate.
///
/// Cleans up both local wizard beams and guest wizard beams independently.
pub fn cleanup_beams_on_cancel(
    mut commands: Commands,
    local_wizard_query: Query<&CastingState, (With<LocalWizard>, Without<DisintegrateCaster>)>,
    guest_wizard_query: Query<&CastingState, (With<GuestWizard>, Without<LocalWizard>, Without<DisintegrateCaster>)>,
    local_beam_query: Query<
        Entity,
        (
            With<DisintegrateBeam>,
            Without<GuestBeam>,
        ),
    >,
    guest_beam_query: Query<
        Entity,
        (
            With<DisintegrateBeam>,
            With<GuestBeam>,
        ),
    >,
) {
    // Cleanup local wizard's beams
    if local_wizard_query.single().is_ok() {
        for entity in local_beam_query.iter() {
            commands.entity(entity).despawn();
        }
    }
    // Cleanup guest wizard's beams
    if guest_wizard_query.single().is_ok() {
        for entity in guest_beam_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawns a beam entity with visual billboard mesh.
/// If `is_guest` is true, adds `GuestBeam` marker to distinguish from local beams.
fn spawn_beam_with_marker(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    origin: Vec3,
    direction: Vec3,
    length: f32,
    empowerment: f32,
    is_guest: bool,
) {
    let midpoint = origin + direction * (length / 2.0);
    let scale = empowerment;
    let beam_width = constants::BEAM_WIDTH * scale;
    let rectangle = Rectangle::new(beam_width, beam_width);

    let mut entity_commands = commands.spawn((
        DisintegrateBeam::new(origin, direction, length, empowerment),
        Mesh3d(meshes.add(rectangle)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: constants::BEAM_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(midpoint),
        OnGameplayScreen,
    ));

    if is_guest {
        entity_commands.insert(GuestBeam);
    }
}

/// System that updates beam mesh transform to match beam data.
pub fn update_beam_visuals(mut beam_query: Query<(&DisintegrateBeam, &mut Transform)>) {
    for (beam, mut transform) in beam_query.iter_mut() {
        // Get current animated length
        let current_len = beam.current_length();

        // Update position to beam midpoint
        let midpoint = beam.origin + beam.direction * (current_len / 2.0);
        transform.translation = midpoint;

        // Calculate rotation to align the rectangle's Y axis with the beam direction
        // The rectangle mesh has its height along the Y axis by default
        let up = Vec3::Y;
        let rotation = Quat::from_rotation_arc(up, beam.direction);
        transform.rotation = rotation;

        // Scale the mesh to match current animated beam length
        // Mesh is BEAM_WIDTH x BEAM_WIDTH, so scale Y to length
        let scale_y = current_len / constants::BEAM_WIDTH;
        transform.scale = Vec3::new(1.0, scale_y, 1.0);
    }
}
