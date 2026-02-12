//! Top-level UI plugin.
//!
//! Aggregates all UI sub-plugins (main menu, pause menu, etc.)

use bevy::prelude::*;
use bevy::ui::UiScale as BevyUiScale;
use bevy::window::PrimaryWindow;

use super::achievement_popup::AchievementPopupPlugin;
use super::action_bar::ActionBarPlugin;
use super::arcanorouter_display::ArcanoRouterDisplayPlugin;
use super::cauldron_menu::CauldronMenuPlugin;
use super::concentration::ConcentrationUIPlugin;
use super::game_over::GameOverPlugin;
use super::in_game::plugin::InGamePlugin;
use super::loading::LoadingUiPlugin;
use super::main_menu::MainMenuPlugin;
use super::pause_menu::plugin::PauseMenuPlugin;
use super::roulette_display::RouletteDisplayPlugin;
use super::rune_display::RuneDisplayPlugin;
use super::spell_book::SpellBookPlugin;
use super::systems;
use super::version::VersionPlugin;
use super::wizard_tower::WizardTowerPlugin;
use crate::game::input::messages::MouseClicked;

/// System set for all button action handlers.
/// Systems in this set only run when a MouseClicked message exists.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ButtonActionSet;

/// Top-level UI plugin that manages all UI systems.
///
/// This plugin aggregates all menu-specific plugins and provides
/// a single entry point for UI functionality.
#[derive(Default)]
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AchievementPopupPlugin,
            MainMenuPlugin,
            LoadingUiPlugin,
            InGamePlugin,
            PauseMenuPlugin,
            SpellBookPlugin,
            CauldronMenuPlugin,
            ActionBarPlugin,
            ConcentrationUIPlugin,
            RuneDisplayPlugin,
            RouletteDisplayPlugin,
            ArcanoRouterDisplayPlugin,
            GameOverPlugin,
            WizardTowerPlugin,
            VersionPlugin,
        ))
        .configure_sets(
            Update,
            ButtonActionSet.run_if(systems::on_message::<MouseClicked>),
        )
        .add_systems(
            Update,
            (
                update_ui_scale,
                systems::button_click_detection,
                systems::button_interaction,
            ),
        );
    }
}

/// Updates the global UI scale based on window width.
///
/// Uses Bevy's built-in UiScale resource to scale all UI elements.
/// Calculates scale factor relative to a base width of 1920px, then applies
/// a 1.5x multiplier to make everything larger.
/// This ensures UI elements shrink/grow proportionally with window size.
fn update_ui_scale(
    mut ui_scale: ResMut<BevyUiScale>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = window_query.single() {
        const BASE_WIDTH: f32 = 1920.0;
        const SCALE_MULTIPLIER: f32 = 1.5;
        let new_scale = (window.width() / BASE_WIDTH) * SCALE_MULTIPLIER;

        if (ui_scale.0 - new_scale).abs() > 0.001 {
            ui_scale.0 = new_scale;
        }
    }
}
