use bevy::prelude::*;

use crate::game::run_conditions;
use crate::state::{AppState, InGameState};
use crate::ui::plugin::ButtonActionSet;

use super::systems;

/// Plugin that manages the action bar UI.
#[derive(Default)]
pub struct ActionBarPlugin;

impl Plugin for ActionBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<systems::AssignSpellToSlot>()
            .add_systems(OnEnter(AppState::InGame), systems::spawn_action_bar)
            .add_systems(
                OnEnter(InGameState::Running),
                systems::spawn_action_bar.run_if(run_conditions::coming_from_game_over),
            )
            .add_systems(
                Update,
                (
                    systems::handle_slot_click.in_set(ButtonActionSet),
                    systems::handle_keyboard_input,
                )
                    .run_if(in_state(InGameState::Running)),
            )
            .add_systems(
                Update,
                (
                    systems::update_action_bar_slots,
                    systems::handle_spell_assignment,
                )
                    .run_if(in_state(InGameState::Running).or(in_state(InGameState::SpellBook))),
            );
    }
}
