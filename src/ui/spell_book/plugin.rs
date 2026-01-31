use bevy::prelude::*;

use crate::state::InGameState;

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
                systems::despawn_spell_book_ui,
            )
            .add_systems(
                Update,
                (
                    systems::button_action,
                    systems::keyboard_input,
                    systems::handle_spell_scroll,
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
