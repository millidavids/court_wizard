use bevy::prelude::*;

use crate::game::run_conditions;
use crate::state::{AppState, InGameState};
use crate::ui::plugin::ButtonActionSet;

use super::systems;

/// Plugin managing the rune display UI.
pub struct RuneDisplayPlugin;

impl Plugin for RuneDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), systems::spawn_rune_display)
            .add_systems(
                OnEnter(InGameState::Running),
                systems::spawn_rune_display.run_if(run_conditions::coming_from_game_over),
            )
            .add_systems(
                Update,
                (
                    systems::handle_rune_button_click.in_set(ButtonActionSet),
                    systems::update_rune_display,
                    // This MUST run before the rune system's handle_rune_activation
                    systems::show_spell_name_on_activation
                        .before(crate::game::runes::systems::handle_rune_activation),
                    systems::update_spell_name_fade,
                )
                    .run_if(in_state(InGameState::Running)),
            );
    }
}
