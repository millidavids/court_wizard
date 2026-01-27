use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, PrimedSpell, Wizard};
use super::components::*;
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::WIZARD_POSITION;
use crate::game::input::MouseButtonState;
use crate::game::input::events::MouseLeftReleased;
use crate::game::units::components::{Health, TemporaryHitPoints, apply_damage_to_unit};

/// Handles Finger of Death casting with left-click.
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
    mut wizard_query: Query<(Entity, &mut CastingState, &Mana, &PrimedSpell, &Wizard)>,
    awaiting_release_query: Query<(), With<AwaitingFingerOfDeathRelease>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut beams: Query<(Entity, &mut FingerOfDeathBeam)>,
) {
    let Ok((wizard_entity, mut casting_state, mana, primed_spell, wizard)) =
        wizard_query.single_mut()
    else {
        return;
    };

    // Check for release event - this is spell-specific logic
    if mouse_left_released.read().next().is_some() {
        // Remove awaiting release marker (allows next cast)
        commands
            .entity(wizard_entity)
            .remove::<AwaitingFingerOfDeathRelease>();

        // Cancel cast on release - despawn beam
        casting_state.cancel();

        // Despawn any existing beams
        for (beam_entity, _) in beams.iter() {
            commands.entity(beam_entity).despawn();
        }

        return;
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

            // Update beam position/direction to follow cursor
            if let Some(cursor_pos) = get_cursor_world_position(&camera_query, &window_query) {
                let beam_origin =
                    WIZARD_POSITION + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

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
                if let Some((_, mut beam)) = beams.iter_mut().next() {
                    // Update existing beam
                    beam.origin = beam_origin;
                    beam.direction = direction;
                    beam.length = beam_length;
                    beam.cast_progress = cast_progress;
                    beam.time_alive += time.delta_secs();
                } else {
                    // No beam exists, spawn new one
                    let mut new_beam = FingerOfDeathBeam::new(beam_origin, direction, beam_length);
                    new_beam.cast_progress = cast_progress;
                    spawn_beam(&mut commands, &mut meshes, &mut materials, new_beam);
                }
            }
        }
        CastingState::Resting => {
            // Not casting - check if we're waiting for mouse release first
            // If so, don't start a new cast even if mana is full
            if awaiting_release_query.get(wizard_entity).is_ok() {
                return;
            }

            // Check for 100% mana requirement before starting cast
            if mana.percentage() >= constants::MANA_REQUIREMENT_PERCENT {
                casting_state.start_cast();

                // Spawn initial beam
                if let Some(cursor_pos) = get_cursor_world_position(&camera_query, &window_query) {
                    let beam_origin =
                        WIZARD_POSITION + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

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

                    let beam = FingerOfDeathBeam::new(beam_origin, direction, beam_length);
                    spawn_beam(&mut commands, &mut meshes, &mut materials, beam);
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
fn spawn_beam(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    beam: FingerOfDeathBeam,
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

    commands.spawn((
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
}

/// Applies Finger of Death damage when cast completes.
///
/// Checks beams where has_fired == false and cast_progress >= 1.0.
/// Applies 1000 damage instantly to all units along beam (hitscan).
/// Drains wizard's entire mana bar and cancels casting state.
/// Adds AwaitingFingerOfDeathRelease component to prevent immediate recast.
pub fn apply_finger_of_death_damage(
    mut mouse_state: ResMut<MouseButtonState>,
    mut beams: Query<&mut FingerOfDeathBeam>,
    mut targets: Query<(&Transform, &mut Health, Option<&mut TemporaryHitPoints>), Without<Wizard>>,
    mut wizard_query: Query<(&mut Mana, &mut CastingState), With<Wizard>>,
) {
    for mut beam in beams.iter_mut() {
        // Only apply damage if cast is complete and hasn't fired yet
        if beam.has_fired || beam.cast_progress < 1.0 {
            continue;
        }

        // Mark as fired
        beam.has_fired = true;

        // Apply damage to all units along beam
        for (transform, mut health, mut temp_hp) in targets.iter_mut() {
            if beam.contains_point(transform.translation, constants::BEAM_WIDTH) {
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), constants::DAMAGE);
            }
        }

        // Drain entire mana bar, cancel casting state, and add awaiting release marker
        if let Ok((mut mana, mut casting_state)) = wizard_query.single_mut() {
            mana.current = 0.0;
            casting_state.cancel(); // Return to Resting immediately

            // Mark mouse hold as consumed to prevent immediate recast
            mouse_state.left_consumed = true;
        }
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
        let scale_y = current_len / constants::BEAM_WIDTH;
        let scale_x = if beam.has_fired {
            constants::BEAM_WIDTH_FIRED / constants::BEAM_WIDTH // Wider after fire
        } else {
            1.0 // Normal width during cast
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
pub fn cleanup_finger_of_death_beams(
    mut commands: Commands,
    beams: Query<(Entity, &FingerOfDeathBeam)>,
    wizard_query: Query<&CastingState, With<Wizard>>,
) {
    let wizard_state = wizard_query.single();

    for (entity, beam) in beams.iter() {
        let should_despawn = if beam.has_fired {
            // Despawn after fade out completes (0.3s after firing)
            beam.time_since_fired >= constants::POST_FIRE_DURATION
        } else {
            // Despawn if wizard is no longer casting (cancelled)
            matches!(wizard_state, Ok(CastingState::Resting))
        };

        if should_despawn {
            commands.entity(entity).despawn();
        }
    }
}
