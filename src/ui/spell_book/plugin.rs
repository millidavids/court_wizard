use bevy::prelude::*;

use crate::state::{InGameState, MultiplayerGameState};
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::{consume_mouse_on_exit, escape_to_running, handle_scroll};

use super::components::ScrollableSpellList;
use super::systems;

/// Plugin that handles the spell book UI.
pub struct SpellBookPlugin;

impl Plugin for SpellBookPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<systems::JustEnteredSpellBook>()
            // SP spell book enter/exit
            .add_systems(
                OnEnter(InGameState::SpellBook),
                (
                    systems::set_just_entered_flag,
                    systems::spawn_spell_book_ui,
                    systems::hide_concentration_ui,
                )
                    .chain(),
            )
            .add_systems(
                OnExit(InGameState::SpellBook),
                (
                    systems::despawn_spell_book_ui,
                    consume_mouse_on_exit,
                    systems::show_concentration_ui,
                ),
            )
            // MP spell book enter/exit (same systems, gameplay continues underneath)
            .add_systems(
                OnEnter(MultiplayerGameState::SpellBook),
                (
                    systems::set_just_entered_flag,
                    systems::spawn_spell_book_ui,
                    systems::hide_concentration_ui,
                )
                    .chain(),
            )
            .add_systems(
                OnExit(MultiplayerGameState::SpellBook),
                (
                    systems::despawn_spell_book_ui,
                    consume_mouse_on_exit,
                    systems::show_concentration_ui,
                ),
            )
            // Update systems run in either SP or MP spell book
            .add_systems(
                Update,
                (
                    systems::button_action.in_set(ButtonActionSet),
                    systems::handle_hotkey_click.in_set(ButtonActionSet),
                    escape_to_running,
                    handle_scroll::<ScrollableSpellList>,
                    systems::handle_number_key_assignment,
                    systems::update_detail_panel,
                    // After the two assignment handlers so a click or number
                    // key repaints the boxes in the same frame it lands.
                    systems::refresh_hotkey_slots
                        .after(systems::handle_hotkey_click)
                        .after(systems::handle_number_key_assignment),
                    systems::refresh_spell_list_selection,
                )
                    .run_if(
                        in_state(InGameState::SpellBook)
                            .or_else(in_state(MultiplayerGameState::SpellBook)),
                    ),
            )
            .add_systems(
                Update,
                systems::clear_just_entered_flag
                    .run_if(
                        in_state(InGameState::SpellBook)
                            .or_else(in_state(MultiplayerGameState::SpellBook)),
                    )
                    .run_if(resource_exists::<systems::JustEnteredSpellBook>),
            );
    }
}
