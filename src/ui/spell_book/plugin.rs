use bevy::prelude::*;

use crate::state::InGameState;

use super::systems;

/// Plugin that handles the spell book UI.
pub struct SpellBookPlugin;

impl Plugin for SpellBookPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameState::SpellBook),
            systems::spawn_spell_book_ui,
        )
        .add_systems(
            OnExit(InGameState::SpellBook),
            systems::despawn_spell_book_ui,
        )
        .add_systems(
            Update,
            (
                systems::button_interaction,
                systems::button_action,
                systems::keyboard_input,
            )
                .run_if(in_state(InGameState::SpellBook)),
        );
    }
}
