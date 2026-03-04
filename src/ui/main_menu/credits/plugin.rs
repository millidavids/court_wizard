//! Plugin for credits screen.

use bevy::prelude::*;

use super::components::ScrollableCreditsContainer;
use super::systems;
use crate::state::MenuState;
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::{escape_to_landing, handle_back_to_landing, handle_scroll};

/// Plugin that handles the credits screen.
pub struct CreditsPlugin;

impl Plugin for CreditsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Credits), systems::setup)
            .add_systems(
                Update,
                (
                    handle_back_to_landing,
                    systems::handle_sprite_credits_button,
                )
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MenuState::Credits)),
            )
            .add_systems(
                Update,
                (
                    handle_scroll::<ScrollableCreditsContainer>,
                    escape_to_landing,
                )
                    .run_if(in_state(MenuState::Credits)),
            )
            .add_systems(
                OnExit(MenuState::Credits),
                crate::ui::systems::cleanup_screen::<super::components::OnCreditsScreen>,
            );
    }
}
