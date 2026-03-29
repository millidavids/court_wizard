use bevy::prelude::*;

use crate::state::MenuState;
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::cleanup_screen;

use super::components::OnRogueliteModifiersScreen;
use super::systems;

/// Plugin that manages the roguelite modifier selection screen.
#[derive(Default)]
pub struct RogueliteModifiersPlugin;

impl Plugin for RogueliteModifiersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::RogueliteModifiers), systems::setup)
            .add_systems(
                OnExit(MenuState::RogueliteModifiers),
                cleanup_screen::<OnRogueliteModifiersScreen>,
            )
            .add_systems(
                Update,
                (
                    systems::button_action.in_set(ButtonActionSet),
                    systems::slider_button_action.in_set(ButtonActionSet),
                    systems::slider_interaction,
                    systems::update_slider_text,
                    systems::update_sliders,
                    systems::escape_to_game_mode_select,
                )
                    .run_if(in_state(MenuState::RogueliteModifiers)),
            );
    }
}
