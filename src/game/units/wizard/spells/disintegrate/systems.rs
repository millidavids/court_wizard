use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::components::OnGameplayScreen;
use crate::game::constants::WIZARD_POSITION;
use crate::game::input::events::{MouseRightHeld, MouseRightReleased};
use crate::game::units::components::Health;
use crate::game::units::wizard::components::{CastingState, Mana, Wizard};

use super::components::DisintegrateBeam;
use super::constants;

/// Marker component for disintegrate spell when it's actively being cast/channeled.
///
/// This differentiates disintegrate from magic missile casting states.
#[derive(Component)]
pub struct DisintegrateCaster;

/// System that handles disintegrate beam casting.
///
/// Right-click starts cast. Must hold for full cast time.
/// After cast completes, enters channeling state where beam is continuously active.
#[allow(clippy::too_many_arguments)]
pub fn handle_disintegrate_casting(
    time: Res<Time>,
    mut right_held: MessageReader<MouseRightHeld>,
    mut right_released: MessageReader<MouseRightReleased>,
    mut commands: Commands,
    mut wizard_query: Query<(Entity, &mut CastingState, &mut Mana), With<Wizard>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut existing_beam: Query<&mut DisintegrateBeam>,
    beam_entities: Query<Entity, With<DisintegrateBeam>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((wizard_entity, mut casting_state, mut mana)) = wizard_query.single_mut() else {
        return;
    };

    // Check for release event
    if right_released.read().next().is_some() {
        // Cancel cast/channel on release
        casting_state.cancel();

        // Remove caster marker from wizard
        commands
            .entity(wizard_entity)
            .remove::<DisintegrateCaster>();

        // Despawn any existing beam
        for entity in beam_entities.iter() {
            commands.entity(entity).despawn();
        }

        return;
    }

    // Check for hold event
    if right_held.read().next().is_none() {
        return;
    }

    // Right mouse is held - handle casting or channeling based on state
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
                    let direction = (target_pos - beam_origin).normalize();

                    // Update existing beam or spawn new one
                    if let Some(mut beam) = existing_beam.iter_mut().next() {
                        // Update existing beam (preserves damage timer)
                        beam.origin = beam_origin;
                        beam.direction = direction;
                    } else {
                        // No beam exists, spawn new one with mesh
                        spawn_beam(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            beam_origin,
                            direction,
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
                for entity in beam_entities.iter() {
                    commands.entity(entity).despawn();
                }
            }
        }
        CastingState::Casting { .. } => {
            // Currently casting - advance cast time
            casting_state.advance(time.delta_secs());

            // Check if cast is complete
            if casting_state.is_complete(constants::CAST_TIME) {
                // Cast complete - transition to channeling and spawn first beam
                casting_state.start_channeling();

                // Spawn initial beam
                if let Some(target_pos) = get_cursor_world_position(&camera_query, &window_query) {
                    let beam_origin =
                        WIZARD_POSITION + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);
                    let direction = (target_pos - beam_origin).normalize();

                    spawn_beam(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        beam_origin,
                        direction,
                    );
                }
            }
        }
        CastingState::Resting => {
            // Not casting or channeling - start new cast
            casting_state.start_cast();

            // Add caster marker to wizard
            commands.entity(wizard_entity).insert(DisintegrateCaster);
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
    mut target_query: Query<(&Transform, &mut Health), Without<Wizard>>,
    time: Res<Time>,
) {
    for mut beam in beam_query.iter_mut() {
        beam.update_damage_timer(time.delta_secs());
        beam.update_time_alive(time.delta_secs());

        if beam.should_damage() {
            // Deal damage to all units in the beam (except wizard)
            for (transform, mut health) in target_query.iter_mut() {
                let position = transform.translation;
                if beam.contains_point(position) {
                    health.take_damage(constants::DAMAGE_PER_TICK);
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
) {
    // Calculate midpoint for the beam billboard
    let midpoint = origin + direction * (constants::BEAM_LENGTH / 2.0);

    // Create a rectangle mesh for the beam
    // We'll use a standard size and scale it later
    let rectangle = Rectangle::new(constants::BEAM_WIDTH, constants::BEAM_WIDTH);

    commands.spawn((
        DisintegrateBeam::new(origin, direction, constants::BEAM_LENGTH),
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
