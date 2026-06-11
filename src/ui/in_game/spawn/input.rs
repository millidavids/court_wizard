use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use super::super::components::*;
use crate::config::GameConfig;
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::game::input::messages::{BlockSpellInput, MouseClicked};
use crate::state::{InGameState, MultiplayerGameState};

/// Blocks spell input when the mouse is interacting with a UI button so
/// clicking a HUD button doesn't simultaneously fire a spell. With a
/// gamepad, spells are triggered by RT (not by a UI cursor), and the hidden
/// OS cursor may sit over a HUD button at any time — gating on mouse mode
/// prevents those incidental hovers from silently killing RT casts.
pub(crate) fn block_spell_input_on_button_interaction(
    active: Res<ActiveInputDevice>,
    button_query: Query<&Interaction, With<Button>>,
    mut block_spell_input: MessageWriter<BlockSpellInput>,
) {
    if active.is_gamepad() {
        return;
    }
    for interaction in &button_query {
        if matches!(*interaction, Interaction::Pressed | Interaction::Hovered) {
            block_spell_input.write(BlockSpellInput);
            return;
        }
    }
}

/// Opens the spell book on gamepad X (West) and the cauldron menu on
/// gamepad Y (North) during gameplay. Matches the existing HUD-button
/// routing so state handling stays in one place.
#[allow(clippy::too_many_arguments)]
pub(crate) fn gamepad_hud_shortcuts(
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    current_state: Option<Res<State<InGameState>>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    config: Res<crate::config::GameConfig>,
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
    mut next_mp_state: Option<ResMut<NextState<MultiplayerGameState>>>,
) {
    // Gameplay must be actively running — `InGameState` exists only in
    // single-player, `MultiplayerGameState` only in multiplayer.
    let sp_running = current_state
        .as_ref()
        .is_some_and(|s| *s.get() == InGameState::Running);
    let mp_running = mp_state
        .as_ref()
        .is_some_and(|s| *s.get() == MultiplayerGameState::Running);
    if !sp_running && !mp_running {
        return;
    }
    let Some(gp_entity) = active.gamepad_entity() else {
        return;
    };
    let Ok(gamepad) = gamepads.get(gp_entity) else {
        return;
    };
    if gamepad.just_pressed(GamepadButton::West) && !config.wizard_type.uses_exclusive_casting() {
        if let Some(ref mut next_sp) = next_in_game_state {
            next_sp.set(InGameState::SpellBook);
        }
        if let Some(ref mut next_mp) = next_mp_state {
            next_mp.set(MultiplayerGameState::SpellBook);
        }
    }
    if gamepad.just_pressed(GamepadButton::North) {
        if let Some(ref mut next_sp) = next_in_game_state {
            next_sp.set(InGameState::CauldronMenu);
        }
        if let Some(ref mut next_mp) = next_mp_state {
            next_mp.set(MultiplayerGameState::CauldronMenu);
        }
    }
}

/// Handles pause input during active gameplay (single-player only):
/// Escape on keyboard or Start on the gamepad. **Not** B/East — in gameplay
/// that button is reserved for other actions and shouldn't stop the game.
///
/// In multiplayer, `mp_escape_key_handler` handles pause instead.
pub(crate) fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    current_state: Option<Res<State<InGameState>>>,
    next_in_game_state: Option<ResMut<NextState<InGameState>>>,
) {
    // Multiplayer pause is handled by `mp_escape_key_handler`; the
    // `InGameState` resources only exist in single-player anyway.
    if mp_state.is_some() {
        return;
    }
    let (Some(current_state), Some(mut next_in_game_state)) = (current_state, next_in_game_state)
    else {
        return;
    };
    if *current_state.get() != InGameState::Running {
        return;
    }
    let gamepad_start = active
        .gamepad_entity()
        .and_then(|e| gamepads.get(e).ok())
        .is_some_and(|g| g.just_pressed(GamepadButton::Start));
    if keyboard.just_pressed(KeyCode::Escape) || gamepad_start {
        next_in_game_state.set(InGameState::Paused);
    }
}

/// Handles HUD button click actions.
///
/// Sets the appropriate state for SP or MP depending on which is active.
pub(crate) fn hud_button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&HudButtonAction>,
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
    mut next_mp_state: Option<ResMut<NextState<MultiplayerGameState>>>,
    config: Res<GameConfig>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                HudButtonAction::OpenSpellBook => {
                    // RuneCaster and Randomancer can only cast via their own mechanics
                    if config.wizard_type.uses_exclusive_casting() {
                        continue;
                    }
                    if let Some(ref mut next_sp) = next_in_game_state {
                        next_sp.set(InGameState::SpellBook);
                    }
                    if let Some(ref mut next_mp) = next_mp_state {
                        next_mp.set(MultiplayerGameState::SpellBook);
                    }
                }
                HudButtonAction::OpenCauldronMenu => {
                    if let Some(ref mut next_sp) = next_in_game_state {
                        next_sp.set(InGameState::CauldronMenu);
                    }
                    if let Some(ref mut next_mp) = next_mp_state {
                        next_mp.set(MultiplayerGameState::CauldronMenu);
                    }
                }
            }
        }
    }
}
