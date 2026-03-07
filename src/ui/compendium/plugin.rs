use bevy::prelude::*;

use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::state::{MenuState, MetaGameState, PauseMenuState};
use crate::ui::plugin::ButtonActionSet;

use super::components::{BackButton, CompendiumState, ScrollableCompendiumContainer};
use crate::ui::systems::{escape_to_landing, escape_to_pause_main, handle_scroll};

use super::systems;

/// Plugin for the compendium in the main menu.
#[derive(Default)]
pub struct MainMenuCompendiumPlugin;

impl Plugin for MainMenuCompendiumPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Compendium), systems::setup_main_menu)
            .add_systems(
                OnExit(MenuState::Compendium),
                (
                    crate::ui::systems::cleanup_screen::<super::components::OnCompendiumScreen>,
                    cleanup_compendium_state,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_main_menu_back_button,
                    systems::handle_tab_click,
                    systems::handle_item_click,
                )
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MenuState::Compendium)),
            )
            .add_systems(
                Update,
                (
                    handle_scroll::<ScrollableCompendiumContainer>,
                    escape_to_landing,
                    systems::rebuild_on_state_change,
                )
                    .run_if(in_state(MenuState::Compendium)),
            );
    }
}

/// Plugin for the compendium in the pause menu.
#[derive(Default)]
pub struct PauseMenuCompendiumPlugin;

impl Plugin for PauseMenuCompendiumPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(PauseMenuState::Compendium),
            systems::setup_pause_menu,
        )
        .add_systems(
            OnExit(PauseMenuState::Compendium),
            (
                crate::ui::systems::cleanup_screen::<super::components::OnCompendiumScreen>,
                cleanup_compendium_state,
            ),
        )
        .add_systems(
            Update,
            (
                handle_pause_menu_back_button,
                systems::handle_tab_click,
                systems::handle_item_click,
            )
                .in_set(ButtonActionSet)
                .run_if(in_state(PauseMenuState::Compendium)),
        )
        .add_systems(
            Update,
            (
                handle_scroll::<ScrollableCompendiumContainer>,
                escape_to_pause_main,
                systems::rebuild_on_state_change,
            )
                .run_if(in_state(PauseMenuState::Compendium)),
        );
    }
}

/// Plugin for the compendium in the meta-game (wizard tower).
#[derive(Default)]
pub struct MetaGameCompendiumPlugin;

impl Plugin for MetaGameCompendiumPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MetaGameState::Compendium), systems::setup_meta_game)
            .add_systems(
                OnExit(MetaGameState::Compendium),
                (
                    crate::ui::systems::cleanup_screen::<super::components::OnCompendiumScreen>,
                    cleanup_compendium_state,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_meta_game_back_button,
                    systems::handle_tab_click,
                    systems::handle_item_click,
                )
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MetaGameState::Compendium)),
            )
            .add_systems(
                Update,
                (
                    handle_scroll::<ScrollableCompendiumContainer>,
                    escape_to_wizard_tower,
                    systems::rebuild_on_state_change,
                )
                    .run_if(in_state(MetaGameState::Compendium)),
            );
    }
}

// ---------------------------------------------------------------------------
// Button handlers
// ---------------------------------------------------------------------------

fn handle_main_menu_back_button(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&BackButton>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            channel_change.write(ChannelChangeMessage);
            next_state.set(MenuState::Landing);
        }
    }
}

fn handle_pause_menu_back_button(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&BackButton>,
    mut next_state: ResMut<NextState<PauseMenuState>>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            next_state.set(PauseMenuState::Main);
        }
    }
}

fn handle_meta_game_back_button(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&BackButton>,
    mut next_state: ResMut<NextState<MetaGameState>>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            next_state.set(MetaGameState::WizardTower);
        }
    }
}

fn escape_to_wizard_tower(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<MetaGameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(MetaGameState::WizardTower);
    }
}

fn cleanup_compendium_state(mut commands: Commands) {
    commands.remove_resource::<CompendiumState>();
}
