//! Guest spell command sending and host spell command processing.
//!
//! The guest captures mouse input and sends `SpellCommand` messages to the host
//! over the reliable channel. The host processes these commands to drive the
//! guest's wizard casting state and spawn spell effects.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::units::components::{Corpse, Team};
use crate::game::units::wizard::components::*;
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::disintegrate::systems::DisintegrateCaster;
use crate::game::units::wizard::spells::magic_missile::components::{
    MagicMissile, TargetTeams,
};
use crate::game::units::wizard::spells::{
    disintegrate::constants as disintegrate_constants,
    magic_missile::constants as magic_missile_constants,
};
use crate::networking::protocol::{NetworkMessage, SpellAction, SpellCommand};
use crate::networking::resources::NetworkConnection;

/// Resource tracking the guest's cursor world position on the host.
///
/// Updated each frame from incoming `ContinueCast` / `StartCast` messages.
#[derive(Resource, Default)]
pub struct GuestCursorPosition {
    pub position: Option<Vec3>,
}

/// Marker component for disintegrate beams owned by the guest wizard.
///
/// Used to distinguish guest beams from host beams in cleanup systems,
/// so each wizard's beams are cleaned up independently.
#[derive(Component)]
pub struct GuestBeam;

// ── Guest Systems ──────────────────────────────────────────────────────

/// Sends the guest's spell priming action to the host.
///
/// When the guest primes a spell locally (via action bar or keyboard),
/// the `handle_prime_spell_messages` system already updates the local
/// wizard's `PrimedSpell`. This system forwards that information to the
/// host so it can update the `GuestWizard` entity.
pub fn send_guest_prime_spell(
    mut connection: ResMut<NetworkConnection>,
    wizard_query: Query<&PrimedSpell, (With<LocalWizard>, Changed<PrimedSpell>)>,
) {
    if let Ok(primed_spell) = wizard_query.single() {
        connection
            .outgoing_messages
            .push(NetworkMessage::SpellCommand(SpellCommand {
                action: SpellAction::PrimeSpell {
                    spell: primed_spell.spell,
                },
            }));
    }
}

/// Captures the guest's mouse input and sends spell commands to the host.
///
/// Reads `ButtonInput<MouseButton>` directly (the guest doesn't run spell
/// casting systems — those run only on the host via `is_gameplay_running`).
/// Sends StartCast/ContinueCast/ReleaseCast commands with cursor world position.
pub fn send_guest_spell_commands(
    mouse: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut connection: ResMut<NetworkConnection>,
    wizard_query: Query<&PrimedSpell, With<LocalWizard>>,
) {
    let Ok(primed_spell) = wizard_query.single() else {
        return;
    };

    // Get cursor world position
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);

    if mouse.just_released(MouseButton::Left) {
        connection
            .outgoing_messages
            .push(NetworkMessage::SpellCommand(SpellCommand {
                action: SpellAction::ReleaseCast,
            }));
    } else if mouse.just_pressed(MouseButton::Left) {
        if let Some(pos) = cursor_pos {
            connection
                .outgoing_messages
                .push(NetworkMessage::SpellCommand(SpellCommand {
                    action: SpellAction::StartCast {
                        spell: primed_spell.spell,
                        cursor_pos: [pos.x, pos.y, pos.z],
                    },
                }));
        }
    } else if mouse.pressed(MouseButton::Left)
        && let Some(pos) = cursor_pos
    {
        connection
            .outgoing_messages
            .push(NetworkMessage::SpellCommand(SpellCommand {
                action: SpellAction::ContinueCast {
                    cursor_pos: [pos.x, pos.y, pos.z],
                },
            }));
    }
}

// ── Host Systems ──────────────────────────────────────────────────────

/// Processes incoming spell commands from the guest.
///
/// Drains `SpellCommand` messages from the connection and updates the
/// `GuestCursorPosition` resource and `GuestWizard` components accordingly.
pub fn process_guest_spell_commands(
    mut connection: ResMut<NetworkConnection>,
    mut cursor: ResMut<GuestCursorPosition>,
    mut wizard_query: Query<(&mut CastingState, &mut PrimedSpell), With<GuestWizard>>,
) {
    let Ok((mut casting_state, mut primed_spell)) = wizard_query.single_mut() else {
        return;
    };

    // Drain all messages, keeping non-SpellCommand messages
    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::SpellCommand(cmd) => match cmd.action {
                SpellAction::PrimeSpell { spell } => {
                    *primed_spell = spell.primed_config();
                }
                SpellAction::StartCast { cursor_pos, .. } => {
                    cursor.position = Some(Vec3::new(cursor_pos[0], cursor_pos[1], cursor_pos[2]));
                    // Start cast if resting
                    if matches!(*casting_state, CastingState::Resting) {
                        casting_state.start_cast();
                    }
                }
                SpellAction::ContinueCast { cursor_pos } => {
                    cursor.position = Some(Vec3::new(cursor_pos[0], cursor_pos[1], cursor_pos[2]));
                }
                SpellAction::ReleaseCast => {
                    cursor.position = None;
                    casting_state.cancel();
                }
            },
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Handles magic missile casting for the guest wizard (host-side).
///
/// Mirrors `handle_magic_missile_casting` but reads `GuestCursorPosition`
/// instead of mouse input. Uses `TargetTeams::DefendersAndUndead` since
/// the guest's missiles target the host's army.
#[allow(clippy::too_many_arguments)]
pub fn handle_guest_magic_missile_casting(
    time: Res<Time>,
    cursor: Res<GuestCursorPosition>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (&Transform, &mut CastingState, &mut Mana, &PrimedSpell, &Wizard),
        With<GuestWizard>,
    >,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    targets: Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
    crystals: Query<(
        Entity,
        &Transform,
        &crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal,
    )>,
) {
    let Ok((wizard_transform, mut casting_state, mut mana, primed_spell, wizard)) =
        wizard_query.single_mut()
    else {
        return;
    };

    // Only process if magic missile is primed
    if primed_spell.spell != Spell::MagicMissile {
        return;
    }

    let spawn_origin = wizard_transform.translation;
    let target_teams = TargetTeams::DefendersAndUndead;

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.advance_channel(time.delta_secs());

            if casting_state.should_channel(
                magic_missile_constants::INITIAL_CHANNEL_INTERVAL,
                magic_missile_constants::MIN_CHANNEL_INTERVAL,
                magic_missile_constants::CHANNEL_RAMP_TIME,
            ) {
                let mana_cost = magic_missile_constants::MANA_COST * wizard.mana_cost_multiplier;
                if mana.consume(mana_cost) {
                    crate::game::units::wizard::spells::magic_missile::systems::spawn_magic_missile(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &camera_query,
                        &targets,
                        &crystals,
                        wizard.spell_range,
                        primed_spell.empowerment,
                        cursor.position,
                        spawn_origin,
                        target_teams,
                    );
                    casting_state.reset_channel_interval();
                } else {
                    casting_state.cancel();
                }
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                let mana_cost = magic_missile_constants::MANA_COST * wizard.mana_cost_multiplier;
                if mana.consume(mana_cost) {
                    crate::game::units::wizard::spells::magic_missile::systems::spawn_magic_missile(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &camera_query,
                        &targets,
                        &crystals,
                        wizard.spell_range,
                        primed_spell.empowerment,
                        cursor.position,
                        spawn_origin,
                        target_teams,
                    );
                    casting_state.start_channeling();
                } else {
                    casting_state.cancel();
                }
            }
        }
        CastingState::Resting => {
            // Cast start is handled by process_guest_spell_commands
        }
    }
}

/// Handles disintegrate casting for the guest wizard (host-side).
///
/// Mirrors `handle_disintegrate_casting` but reads `GuestCursorPosition`
/// instead of mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_guest_disintegrate_casting(
    time: Res<Time>,
    cursor: Res<GuestCursorPosition>,
    mut commands: Commands,
    mut wizard_query: Query<
        (Entity, &Transform, &mut CastingState, &mut Mana, &PrimedSpell, &Wizard),
        With<GuestWizard>,
    >,
    mut beams: Query<(Entity, &mut DisintegrateBeam), With<GuestBeam>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((wizard_entity, wizard_transform, mut casting_state, mut mana, primed_spell, wizard)) =
        wizard_query.single_mut()
    else {
        return;
    };

    // Only process if disintegrate is primed
    if primed_spell.spell != Spell::Disintegrate {
        return;
    }

    let wizard_pos = wizard_transform.translation;

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.advance_channel(time.delta_secs());

            let mana_cost = disintegrate_constants::MANA_COST_PER_SECOND * time.delta_secs();

            if mana.consume(mana_cost) {
                if let Some(target_pos) = cursor.position {
                    let beam_origin = wizard_pos
                        + Vec3::new(0.0, disintegrate_constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

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
                        .min(disintegrate_constants::BEAM_LENGTH);

                    if let Some((_, mut beam)) = beams.iter_mut().next() {
                        beam.origin = beam_origin;
                        beam.direction = direction;
                        beam.length = beam_length;
                    } else {
                        spawn_disintegrate_beam(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            beam_origin,
                            direction,
                            beam_length,
                            primed_spell.empowerment,
                        );
                    }
                }
            } else {
                casting_state.cancel();
                commands
                    .entity(wizard_entity)
                    .remove::<DisintegrateCaster>();
                for (entity, _) in beams.iter() {
                    commands.entity(entity).despawn();
                }
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                casting_state.start_channeling();

                if let Some(target_pos) = cursor.position {
                    let beam_origin = wizard_pos
                        + Vec3::new(0.0, disintegrate_constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

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
                        .min(disintegrate_constants::BEAM_LENGTH);

                    spawn_disintegrate_beam(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        beam_origin,
                        direction,
                        beam_length,
                        primed_spell.empowerment,
                    );
                }
            }
        }
        CastingState::Resting => {
            // Cast start is handled by process_guest_spell_commands
        }
    }
}

/// Cleans up guest wizard beams when cast is cancelled.
///
/// Mirrors `cleanup_beams_on_cancel` but checks `GuestWizard` instead of
/// `LocalWizard`.
pub fn cleanup_guest_beams_on_cancel(
    mut commands: Commands,
    wizard_query: Query<&CastingState, (With<GuestWizard>, Without<DisintegrateCaster>)>,
    beam_query: Query<Entity, (With<DisintegrateBeam>, With<GuestBeam>)>,
) {
    if wizard_query.single().is_ok() {
        for entity in beam_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────

/// Gets the cursor position projected onto the battlefield surface (Y=0 plane).
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

/// Spawns a disintegrate beam entity with visual billboard mesh.
///
/// Extracted helper to avoid depending on `disintegrate::systems::spawn_beam`
/// which is private.
fn spawn_disintegrate_beam(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    origin: Vec3,
    direction: Vec3,
    length: f32,
    empowerment: f32,
) {
    let midpoint = origin + direction * (length / 2.0);

    let scale = empowerment;
    let beam_width = disintegrate_constants::BEAM_WIDTH * scale;
    let rectangle = Rectangle::new(beam_width, beam_width);

    commands.spawn((
        DisintegrateBeam::new(origin, direction, length, empowerment),
        Mesh3d(meshes.add(rectangle)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: disintegrate_constants::BEAM_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(midpoint),
        GuestBeam,
        crate::game::components::OnGameplayScreen,
    ));
}
