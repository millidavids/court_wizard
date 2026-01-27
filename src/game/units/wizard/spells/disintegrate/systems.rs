use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, PrimedSpell, Wizard};
use super::components::DisintegrateBeam;
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::WIZARD_POSITION;
use crate::game::input::events::MouseLeftReleased;
use crate::game::units::components::{Health, TemporaryHitPoints, apply_damage_to_unit};

/// Marker component for disintegrate spell when it's actively being cast/channeled.
///
/// This differentiates disintegrate from magic missile casting states.
#[derive(Component)]
pub struct DisintegrateCaster;

/// System that handles disintegrate beam casting.
///
/// Left-click starts cast. Must hold for full cast time.
/// After cast completes, enters channeling state where beam is continuously active.
/// Only casts when Disintegrate is the primed spell.
///
/// Note: Spell priming, input blocking, and mouse state checks are handled by run_if conditions.
#[allow(clippy::too_many_arguments)]
pub fn handle_disintegrate_casting(
    time: Res<Time>,
    mut left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<(Entity, &mut CastingState, &mut Mana, &PrimedSpell, &Wizard)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut beams: Query<(Entity, &mut DisintegrateBeam)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((wizard_entity, mut casting_state, mut mana, primed_spell, wizard)) =
        wizard_query.single_mut()
    else {
        return;
    };

    // Check for release event - this is spell-specific logic
    if left_released.read().next().is_some() {
        // Cancel cast/channel on release
        casting_state.cancel();

        // Remove caster marker from wizard
        commands
            .entity(wizard_entity)
            .remove::<DisintegrateCaster>();

        // Despawn any existing beam
        for (entity, _) in beams.iter() {
            commands.entity(entity).despawn();
        }

        return;
    }

    // Left mouse is held - handle casting or channeling based on state
    match *casting_state {
        CastingState::Channeling { .. } => {
            // Already channeling - advance channel time
            casting_state.advance_channel(time.delta_secs());

            // Calculate mana cost for this frame
            let mana_cost = constants::MANA_COST_PER_SECOND * time.delta_secs();

            if mana.consume(mana_cost) {
                // Update beam position based on cursor
                if let Some(target_pos) = get_cursor_world_position(&camera_query, &window_query) {
                    let beam_origin =
                        WIZARD_POSITION + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                    // Clamp target position to spell range
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

                    // Update existing beam or spawn new one
                    if let Some((_, mut beam)) = beams.iter_mut().next() {
                        // Update existing beam (preserves damage timer)
                        beam.origin = beam_origin;
                        beam.direction = direction;
                        beam.length = beam_length;
                    } else {
                        // No beam exists, spawn new one with mesh
                        spawn_beam(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            beam_origin,
                            direction,
                            beam_length,
                        );
                    }
                }
            } else {
                // Out of mana - cancel channeling
                casting_state.cancel();

                // Remove caster marker from wizard
                commands
                    .entity(wizard_entity)
                    .remove::<DisintegrateCaster>();

                // Despawn beam
                for (entity, _) in beams.iter() {
                    commands.entity(entity).despawn();
                }
            }
        }
        CastingState::Casting { .. } => {
            // Currently casting - advance cast time
            casting_state.advance(time.delta_secs());

            // Check if cast is complete
            if casting_state.is_complete(primed_spell.cast_time) {
                // Cast complete - transition to channeling and spawn first beam
                casting_state.start_channeling();

                // Spawn initial beam
                if let Some(target_pos) = get_cursor_world_position(&camera_query, &window_query) {
                    let beam_origin =
                        WIZARD_POSITION + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                    // Clamp target position to spell range
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

                    spawn_beam(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        beam_origin,
                        direction,
                        beam_length,
                    );
                }
            }
        }
        CastingState::Resting => {
            // Not casting or channeling - check mana before starting cast
            // Need enough mana for at least 0.1 seconds of channeling
            if mana.can_afford(constants::MANA_COST_PER_SECOND * 0.1) {
                casting_state.start_cast();

                // Add caster marker to wizard
                commands.entity(wizard_entity).insert(DisintegrateCaster);
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
    mut beam_query: Query<&mut DisintegrateBeam>,
    mut target_query: Query<
        (&Transform, &mut Health, Option<&mut TemporaryHitPoints>),
        Without<Wizard>,
    >,
    time: Res<Time>,
) {
    for mut beam in beam_query.iter_mut() {
        beam.update_damage_timer(time.delta_secs());
        beam.update_time_alive(time.delta_secs());

        if beam.should_damage() {
            // Deal damage to all units in the beam (except wizard)
            for (transform, mut health, mut temp_hp) in target_query.iter_mut() {
                let position = transform.translation;
                if beam.contains_point(position) {
                    apply_damage_to_unit(
                        &mut health,
                        temp_hp.as_deref_mut(),
                        constants::DAMAGE_PER_TICK,
                    );
                }
            }

            beam.reset_damage_timer();
        }
    }
}

/// System that despawns beams when wizard is not actively channeling disintegrate.
pub fn cleanup_beams_on_cancel(
    mut commands: Commands,
    wizard_query: Query<&CastingState, (With<Wizard>, Without<DisintegrateCaster>)>,
    beam_query: Query<Entity, With<DisintegrateBeam>>,
) {
    // Only cleanup if wizard is not a disintegrate caster
    if wizard_query.single().is_ok() {
        // Wizard is resting or casting something else, despawn all disintegrate beams
        for entity in beam_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawns a beam entity with visual billboard mesh.
fn spawn_beam(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    origin: Vec3,
    direction: Vec3,
    length: f32,
) {
    // Calculate midpoint for the beam billboard
    let midpoint = origin + direction * (length / 2.0);

    // Create a rectangle mesh for the beam
    // We'll use a standard size and scale it later
    let rectangle = Rectangle::new(constants::BEAM_WIDTH, constants::BEAM_WIDTH);

    commands.spawn((
        DisintegrateBeam::new(origin, direction, length),
        Mesh3d(meshes.add(rectangle)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: constants::BEAM_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(midpoint),
        OnGameplayScreen,
    ));
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
