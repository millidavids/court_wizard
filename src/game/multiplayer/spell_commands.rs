//! Guest spell command sending and host spell command processing.
//!
//! The guest captures mouse input and sends `SpellCommand` messages to the host
//! over the reliable channel. The host processes these commands to update the
//! `GuestCursorPosition` and `GuestInputState` resources, which the widened SP
//! casting systems read to drive the guest wizard's spell effects.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::units::wizard::components::*;
use crate::networking::protocol::{NetworkMessage, SpellAction, SpellCommand};
use crate::networking::resources::NetworkConnection;

/// Resource tracking the guest's cursor world position on the host.
///
/// Updated each frame from incoming `ContinueCast` / `StartCast` messages.
#[derive(Resource, Default)]
pub struct GuestCursorPosition {
    pub position: Option<Vec3>,
}

/// Resource translating guest network messages into input signals.
///
/// The SP casting systems check mouse state for the local wizard. For the guest
/// wizard, they read this resource instead. Updated each frame by
/// `process_guest_spell_commands` before the casting systems run.
#[derive(Resource, Default)]
pub struct GuestInputState {
    /// True on the frame a `StartCast` message is received.
    pub just_pressed: bool,
    /// True while the guest is holding mouse (StartCast or ContinueCast received).
    pub pressed: bool,
    /// True on the frame a `ReleaseCast` message is received.
    pub just_released: bool,
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
/// Drains `SpellCommand` messages from the connection and updates
/// `GuestCursorPosition`, `GuestInputState`, and the guest wizard's `PrimedSpell`.
/// The actual `CastingState` is driven by the widened SP casting systems.
pub fn process_guest_spell_commands(
    mut connection: ResMut<NetworkConnection>,
    mut cursor: ResMut<GuestCursorPosition>,
    mut guest_input: ResMut<GuestInputState>,
    mut wizard_query: Query<&mut PrimedSpell, With<GuestWizard>>,
) {
    // Reset per-frame input flags
    guest_input.just_pressed = false;
    guest_input.just_released = false;
    guest_input.pressed = false;

    let Ok(mut primed_spell) = wizard_query.single_mut() else {
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
                    guest_input.just_pressed = true;
                    guest_input.pressed = true;
                }
                SpellAction::ContinueCast { cursor_pos } => {
                    cursor.position = Some(Vec3::new(cursor_pos[0], cursor_pos[1], cursor_pos[2]));
                    guest_input.pressed = true;
                }
                SpellAction::ReleaseCast => {
                    guest_input.just_released = true;
                }
            },
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
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
