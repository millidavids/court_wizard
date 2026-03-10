use bevy::prelude::*;

use crate::game::run_conditions::is_local_wizard_active;
use crate::state::{InGameState, MultiplayerGameState};
use crate::ui::plugin::ButtonActionSet;

use super::components::InfiniteMana;
use super::messages::AssignSpellToSlot;
use super::systems;

/// Plugin that manages the action bar UI.
#[derive(Default)]
pub struct ActionBarPlugin;

impl Plugin for ActionBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InfiniteMana>()
            .add_message::<AssignSpellToSlot>()
            // Clear blocked spells before spawning the action bar
            .add_systems(
                OnEnter(InGameState::Running),
                systems::clear_blocked_action_bar_spells,
            )
            // SP spawn
            .add_systems(
                OnEnter(InGameState::Running),
                systems::spawn_action_bar
                    .after(systems::clear_blocked_action_bar_spells),
            )
            // MP spawn
            .add_systems(
                OnEnter(MultiplayerGameState::Running),
                systems::spawn_action_bar,
            )
            .add_systems(
                Update,
                (
                    systems::handle_slot_click.in_set(ButtonActionSet),
                    systems::handle_debug_mana_click.in_set(ButtonActionSet),
                    systems::handle_keyboard_input,
                    systems::highlight_keyboard_pressed_slots,
                )
                    .run_if(is_local_wizard_active),
            )
            .add_systems(
                Update,
                (
                    systems::update_action_bar_slots,
                    systems::handle_spell_assignment,
                )
                    .run_if(
                        is_local_wizard_active
                            .or(in_state(InGameState::SpellBook))
                            .or(in_state(MultiplayerGameState::SpellBook)),
                    ),
            );
    }
}
