use bevy::prelude::*;

use super::super::components::*;
use crate::game::cauldron::brews::Recipe;
use crate::game::cauldron::components::{Cauldron, CauldronState};
use crate::game::cauldron::messages::{CancelBrewMessage, StartBrewMessage};
use crate::game::input::messages::MouseClicked;
use crate::state::{InGameState, MultiplayerGameState};

/// Handles cauldron menu button clicks: starts or cancels a brew, or closes
/// the menu when the back/close button is pressed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&CauldronMenuButtonAction>,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    mut selection: ResMut<IngredientSelection>,
    mut start_brew: MessageWriter<StartBrewMessage>,
    mut cancel_brew: MessageWriter<CancelBrewMessage>,
    // BOTH optional: `InGameState` is a SubState that is REMOVED during
    // `AppState::MultiplayerGame` (and `MultiplayerGameState` is absent in SP), so a
    // non-optional `ResMut<NextState<InGameState>>` would PANIC on any cauldron-menu
    // click in multiplayer. Set whichever exists (mirrors `escape_to_running`).
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
    mut next_mp_state: Option<ResMut<NextState<MultiplayerGameState>>>,
) {
    let is_brewing = cauldron_query
        .single()
        .is_ok_and(|state| state.is_brewing());

    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                CauldronMenuButtonAction::ToggleIngredient(ingredient) => {
                    // `toggle` enforces the 3-ingredient cap (a 4th click is a
                    // no-op). `sync_toggle_button_states` restyles every button —
                    // including disabling the unselected ones at the limit — so we
                    // only mutate the selection here.
                    selection.toggle(*ingredient);
                }
                CauldronMenuButtonAction::TogglePhilosophersStone => {
                    selection.toggle_stone();
                }
                CauldronMenuButtonAction::StartBrew => {
                    if !is_brewing && !selection.is_empty() {
                        let recipe = Recipe::new(selection.build_ingredients());
                        start_brew.write(StartBrewMessage { recipe });
                        selection.clear();
                        if let Some(ref mut s) = next_in_game_state {
                            s.set(InGameState::Running);
                        }
                        if let Some(ref mut mp) = next_mp_state {
                            mp.set(MultiplayerGameState::Running);
                        }
                    }
                }
                CauldronMenuButtonAction::CancelBrew => {
                    cancel_brew.write(CancelBrewMessage);
                    if let Some(ref mut s) = next_in_game_state {
                        s.set(InGameState::Running);
                    }
                    if let Some(ref mut mp) = next_mp_state {
                        mp.set(MultiplayerGameState::Running);
                    }
                }
                CauldronMenuButtonAction::Close => {
                    if let Some(ref mut s) = next_in_game_state {
                        s.set(InGameState::Running);
                    }
                    if let Some(ref mut mp) = next_mp_state {
                        mp.set(MultiplayerGameState::Running);
                    }
                }
            }
        }
    }
}

/// Despawns cauldron menu UI when exiting the CauldronMenu state.
pub(crate) fn despawn_cauldron_menu_ui(
    mut commands: Commands,
    query: Query<Entity, With<OnCauldronMenuScreen>>,
) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
}
