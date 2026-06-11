//! Manual screen plugin for both main menu and pause menu.

use bevy::prelude::*;

use crate::state::{MenuState, PauseMenuState};
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::{escape_to_landing, escape_to_pause_main, handle_scroll};

use super::components::{ManualTab, OnManualScreen, ScrollableManualContainer};
use super::systems;

/// Plugin that manages the manual screen UI for the main menu.
#[derive(Default)]
pub struct MainMenuManualPlugin;

impl Plugin for MainMenuManualPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Manual), systems::setup_main_menu)
            .add_systems(
                OnExit(MenuState::Manual),
                (
                    crate::ui::systems::cleanup_screen::<OnManualScreen>,
                    systems::cleanup_resources,
                ),
            )
            .add_systems(
                Update,
                (
                    systems::handle_main_menu_back_button.in_set(ButtonActionSet),
                    systems::handle_tab_click.in_set(ButtonActionSet),
                    systems::rebuild_content_on_tab_change
                        .run_if(resource_exists::<ManualTab>.and(resource_changed::<ManualTab>)),
                    systems::update_tab_active_state
                        .run_if(resource_exists::<ManualTab>.and(resource_changed::<ManualTab>)),
                    handle_scroll::<ScrollableManualContainer>,
                    escape_to_landing,
                )
                    .run_if(in_state(MenuState::Manual)),
            );
    }
}

/// Plugin that manages the manual screen UI for the pause menu.
#[derive(Default)]
pub struct PauseMenuManualPlugin;

impl Plugin for PauseMenuManualPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PauseMenuState::Manual), systems::setup_pause_menu)
            .add_systems(
                OnExit(PauseMenuState::Manual),
                (
                    crate::ui::systems::cleanup_screen::<OnManualScreen>,
                    systems::cleanup_resources,
                ),
            )
            .add_systems(
                Update,
                (
                    systems::handle_pause_menu_back_button.in_set(ButtonActionSet),
                    systems::handle_tab_click.in_set(ButtonActionSet),
                    systems::rebuild_content_on_tab_change
                        .run_if(resource_exists::<ManualTab>.and(resource_changed::<ManualTab>)),
                    systems::update_tab_active_state
                        .run_if(resource_exists::<ManualTab>.and(resource_changed::<ManualTab>)),
                    handle_scroll::<ScrollableManualContainer>,
                    escape_to_pause_main,
                )
                    .run_if(in_state(PauseMenuState::Manual)),
            );
    }
}
