//! Spell book interaction: button actions, hotkeys, panel updates.

use super::setup::JustEnteredSpellBook;
use bevy::prelude::*;

use super::components::*;
use super::hotkey_slots::SLOT_COUNT;
use crate::config::GameConfig;
use crate::game::input::messages::{ActionBarKeyPressed, MouseClicked};
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::messages::PrimeSpellMessage;
use crate::state::{InGameState, MultiplayerGameState};
use crate::ui::concentration::ConcentrationUIRoot;

// ---------------------------------------------------------------------------
// Setup / teardown
// ---------------------------------------------------------------------------

/// Spawns the spell book UI when entering the SpellBook state.
pub(super) fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&SpellBookButtonAction>,
    mut selected: ResMut<SelectedSpellPreview>,
    mut prime_spell: MessageWriter<PrimeSpellMessage>,
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
    mut next_mp_state: Option<ResMut<NextState<MultiplayerGameState>>>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                // Clicking a spell primes it but leaves the menu open so
                // the player can assign it to an action-bar slot before
                // returning to the battle. The menu is dismissed only via
                // the explicit close button or the back/B input.
                SpellBookButtonAction::SelectSpell(spell) => {
                    selected.0 = *spell;
                    prime_spell.write(PrimeSpellMessage {
                        spell: spell.primed_config(),
                    });
                }
                SpellBookButtonAction::Close => {
                    if let Some(ref mut next_sp) = next_in_game_state {
                        next_sp.set(InGameState::Running);
                    }
                    if let Some(ref mut next_mp) = next_mp_state {
                        next_mp.set(MultiplayerGameState::Running);
                    }
                }
            }
        }
    }
}

/// Assigns or unassigns a slot. The boxes' appearance is `refresh_hotkey_slots`'
/// job — it reacts to the `GameConfig` change these two write.
fn toggle_slot(config: &mut GameConfig, slot_idx: usize, spell: Spell) {
    if config.action_bar_slots.get(slot_idx) == Some(&Some(spell)) {
        config.action_bar_slots[slot_idx] = None;
    } else {
        config.action_bar_slots[slot_idx] = Some(spell);
    }
}

/// Handles clicking a hotkey slot button to assign the selected spell.
pub(super) fn handle_hotkey_click(
    mut button_clicked: MessageReader<MouseClicked>,
    hotkey_query: Query<&HotkeySlotButton>,
    selected: Res<SelectedSpellPreview>,
    mut config: ResMut<GameConfig>,
    mut config_changed: MessageWriter<crate::config::ConfigChanged>,
) {
    for event in button_clicked.read() {
        let Ok(slot_btn) = hotkey_query.get(event.button) else {
            continue;
        };
        toggle_slot(&mut config, slot_btn.0 as usize, selected.0);
        config_changed.write(crate::config::ConfigChanged);
    }
}

/// Handles number key presses to assign/unassign the selected spell to an action bar slot.
pub(super) fn handle_number_key_assignment(
    mut action_bar_key: MessageReader<ActionBarKeyPressed>,
    selected: Res<SelectedSpellPreview>,
    mut config: ResMut<GameConfig>,
    mut config_changed: MessageWriter<crate::config::ConfigChanged>,
) {
    for event in action_bar_key.read() {
        if event.slot >= SLOT_COUNT {
            continue;
        }
        toggle_slot(&mut config, event.slot as usize, selected.0);
        config_changed.write(crate::config::ConfigChanged);
    }
}

/// Despawns spell book UI when exiting the SpellBook state.
pub(super) fn despawn_spell_book_ui(
    mut commands: Commands,
    query: Query<Entity, With<OnSpellBookScreen>>,
) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
    commands.remove_resource::<SelectedSpellPreview>();
}

/// Sets the flag when entering spell book to prevent spell casting.
pub(super) fn set_just_entered_flag(mut just_entered: ResMut<JustEnteredSpellBook>) {
    just_entered.0 = true;
}

/// Clears the flag after one frame in SpellBook state.
pub(super) fn clear_just_entered_flag(mut just_entered: ResMut<JustEnteredSpellBook>) {
    just_entered.0 = false;
}

/// Hides the concentration UI when the spell book opens.
pub(super) fn hide_concentration_ui(mut query: Query<&mut Visibility, With<ConcentrationUIRoot>>) {
    for mut vis in &mut query {
        *vis = Visibility::Hidden;
    }
}

/// Shows the concentration UI when the spell book closes.
pub(super) fn show_concentration_ui(mut query: Query<&mut Visibility, With<ConcentrationUIRoot>>) {
    for mut vis in &mut query {
        *vis = Visibility::Inherited;
    }
}
