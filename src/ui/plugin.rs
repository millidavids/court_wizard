use bevy::prelude::*;

use crate::state::{GameState, MenuState};

use super::systems::{menu, scroll, settings};

/// Plugin that manages all UI systems for menus
#[derive(Default)]
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize menu state resource
            .init_resource::<MenuState>()
            // Systems that run when entering the StartMenu state
            .add_systems(OnEnter(GameState::StartMenu), menu::spawn_main_menu)
            // Systems that run when exiting the StartMenu state
            .add_systems(OnExit(GameState::StartMenu), menu::cleanup_menu_ui)
            // Systems that run every frame while in StartMenu state
            .add_systems(
                Update,
                (
                    scroll::send_scroll_events,
                    menu::handle_main_menu_buttons,
                    menu::update_button_colors,
                    menu::handle_menu_state_transitions,
                    settings::handle_settings_buttons,
                    settings::update_settings_ui,
                    settings::update_button_colors,
                ).run_if(in_state(GameState::StartMenu)),
            )
            // Add scroll event observer
            .add_observer(scroll::on_scroll_handler);
    }
}
