use bevy::prelude::*;

use crate::state::InGameState;
use crate::ui::plugin::ButtonActionSet;

use super::systems;

/// Plugin that handles the spell book UI.
pub struct SpellBookPlugin;

impl Plugin for SpellBookPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<systems::JustEnteredSpellBook>()
            .add_systems(
                OnEnter(InGameState::SpellBook),
                (systems::set_just_entered_flag, systems::spawn_spell_book_ui).chain(),
            )
            .add_systems(
                OnExit(InGameState::SpellBook),
                (
                    systems::despawn_spell_book_ui,
                    systems::consume_mouse_on_exit,
                ),
            )
            .add_systems(
                Update,
                (
                    systems::button_action.in_set(ButtonActionSet),
                    systems::handle_hotkey_click.in_set(ButtonActionSet),
                    systems::keyboard_input,
                    systems::handle_spell_scroll,
                    systems::handle_number_key_assignment,
                    systems::update_detail_panel,
                )
                    .run_if(in_state(InGameState::SpellBook)),
            )
            .add_systems(
                Update,
                systems::clear_just_entered_flag
                    .run_if(in_state(InGameState::SpellBook))
                    .run_if(resource_exists::<systems::JustEnteredSpellBook>),
            );
    }
}
