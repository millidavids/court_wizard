//! Spell book interaction: button actions, hotkeys, panel updates.

use super::setup::JustEnteredSpellBook;
use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::config::GameConfig;
use crate::game::input::messages::{ActionBarKeyPressed, MouseClicked};
use crate::game::units::wizard::messages::PrimeSpellMessage;
use crate::state::{InGameState, MultiplayerGameState};
use crate::ui::components::ButtonColors;
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

/// Updates the detail panel text and hotkey highlights when the selected spell changes.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_detail_panel(
    selected: Res<SelectedSpellPreview>,
    config: Res<GameConfig>,
    mut name_query: Query<
        &mut Text,
        (
            With<DetailName>,
            Without<DetailDamageType>,
            Without<DetailDescription>,
            Without<DetailInstructions>,
        ),
    >,
    mut type_query: Query<
        &mut Text,
        (
            With<DetailDamageType>,
            Without<DetailName>,
            Without<DetailDescription>,
            Without<DetailInstructions>,
        ),
    >,
    mut desc_query: Query<
        &mut Text,
        (
            With<DetailDescription>,
            Without<DetailName>,
            Without<DetailDamageType>,
            Without<DetailInstructions>,
        ),
    >,
    mut instr_query: Query<
        &mut Text,
        (
            With<DetailInstructions>,
            Without<DetailName>,
            Without<DetailDamageType>,
            Without<DetailDescription>,
        ),
    >,
    mut hotkey_query: Query<(
        Entity,
        &HotkeySlotButton,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut ButtonColors,
    )>,
    children_query: Query<&Children>,
    mut hotkey_text_query: Query<&mut TextColor>,
    mut spell_list_query: Query<(&SpellListButton, &mut BorderColor), Without<HotkeySlotButton>>,
) {
    if !selected.is_changed() && !config.is_changed() {
        return;
    }

    let spell = selected.0;

    // Update detail text
    if let Ok(mut text) = name_query.single_mut() {
        **text = spell.display_name().to_string();
    }
    if let Ok(mut text) = type_query.single_mut() {
        **text = spell.damage_type().display_name().to_string();
    }
    if let Ok(mut text) = desc_query.single_mut() {
        **text = spell.description().to_string();
    }
    if let Ok(mut text) = instr_query.single_mut() {
        **text = spell.instructions().to_string();
    }

    // Update hotkey box highlights — collect entity→color mapping first
    let mut hotkey_text_updates: Vec<(Entity, Color)> = Vec::new();
    for (entity, slot_btn, mut bg, mut border, mut colors) in &mut hotkey_query {
        let is_active = config.action_bar_slots[slot_btn.0 as usize] == Some(spell);
        let (new_bg, new_border, new_text_color) = if is_active {
            (HOTKEY_ACTIVE_BG, HOTKEY_ACTIVE_BORDER, HOTKEY_ACTIVE_TEXT)
        } else {
            (
                HOTKEY_INACTIVE_BG,
                HOTKEY_INACTIVE_BORDER,
                HOTKEY_INACTIVE_TEXT,
            )
        };
        bg.0 = new_bg;
        *border = BorderColor::all(new_border);
        colors.background = new_bg;
        colors.border = new_border;
        hotkey_text_updates.push((entity, new_text_color));
    }

    // Apply text color updates to hotkey button descendants
    for (btn_entity, new_text_color) in &hotkey_text_updates {
        if let Ok(children) = children_query.get(*btn_entity) {
            for child in children.iter() {
                if let Ok(mut tc) = hotkey_text_query.get_mut(child) {
                    tc.0 = *new_text_color;
                }
                if let Ok(grandchildren) = children_query.get(child) {
                    for gc in grandchildren.iter() {
                        if let Ok(mut tc) = hotkey_text_query.get_mut(gc) {
                            tc.0 = *new_text_color;
                        }
                    }
                }
            }
        }
    }

    // Update spell list borders
    for (list_btn, mut border) in &mut spell_list_query {
        let is_selected = list_btn.0 == spell;
        *border = BorderColor::all(if is_selected {
            SPELL_BUTTON_SELECTED_BORDER
        } else {
            SPELL_BUTTON_BORDER
        });
    }
}

/// Handles clicking a hotkey slot button to assign the selected spell.
pub(super) fn handle_hotkey_click(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    hotkey_query: Query<(Entity, &HotkeySlotButton)>,
    all_hotkey_buttons: Query<(Entity, &HotkeySlotButton), With<Button>>,
    selected: Res<SelectedSpellPreview>,
    mut config: ResMut<GameConfig>,
    mut config_changed: MessageWriter<crate::config::ConfigChanged>,
) {
    for event in button_clicked.read() {
        let Ok((_, slot_btn)) = hotkey_query.get(event.button) else {
            continue;
        };
        let slot_idx = slot_btn.0 as usize;

        // Toggle: if already assigned to this slot, unassign; otherwise assign
        if config.action_bar_slots.get(slot_idx) == Some(&Some(selected.0)) {
            config.action_bar_slots[slot_idx] = None;
        } else {
            config.action_bar_slots[slot_idx] = Some(selected.0);
        }
        config_changed.write(crate::config::ConfigChanged);

        // Update ButtonActive on all hotkey buttons to reflect new state
        for (entity, btn) in &all_hotkey_buttons {
            let is_active = config.action_bar_slots[btn.0 as usize] == Some(selected.0);
            if is_active {
                commands.entity(entity).insert((
                    crate::ui::components::ButtonActive,
                    ButtonColors {
                        background: HOTKEY_ACTIVE_BG,
                        border: HOTKEY_ACTIVE_BORDER,
                    },
                ));
            } else {
                commands
                    .entity(entity)
                    .remove::<crate::ui::components::ButtonActive>();
                commands.entity(entity).insert(ButtonColors {
                    background: HOTKEY_INACTIVE_BG,
                    border: HOTKEY_INACTIVE_BORDER,
                });
            }
        }
    }
}

/// Handles number key presses to assign/unassign the selected spell to an action bar slot.
pub(super) fn handle_number_key_assignment(
    mut commands: Commands,
    mut action_bar_key: MessageReader<ActionBarKeyPressed>,
    all_hotkey_buttons: Query<(Entity, &HotkeySlotButton), With<Button>>,
    selected: Res<SelectedSpellPreview>,
    mut config: ResMut<GameConfig>,
    mut config_changed: MessageWriter<crate::config::ConfigChanged>,
) {
    for event in action_bar_key.read() {
        if event.slot >= 5 {
            continue;
        }
        let slot_idx = event.slot as usize;

        // Toggle: if already assigned, unassign; otherwise assign
        if config.action_bar_slots.get(slot_idx) == Some(&Some(selected.0)) {
            config.action_bar_slots[slot_idx] = None;
        } else {
            config.action_bar_slots[slot_idx] = Some(selected.0);
        }
        config_changed.write(crate::config::ConfigChanged);

        // Update ButtonActive on all hotkey buttons
        for (entity, btn) in &all_hotkey_buttons {
            let is_active = config.action_bar_slots[btn.0 as usize] == Some(selected.0);
            if is_active {
                commands.entity(entity).insert((
                    crate::ui::components::ButtonActive,
                    ButtonColors {
                        background: HOTKEY_ACTIVE_BG,
                        border: HOTKEY_ACTIVE_BORDER,
                    },
                ));
            } else {
                commands
                    .entity(entity)
                    .remove::<crate::ui::components::ButtonActive>();
                commands.entity(entity).insert(ButtonColors {
                    background: HOTKEY_INACTIVE_BG,
                    border: HOTKEY_INACTIVE_BORDER,
                });
            }
        }
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
