//! Top-level UI plugin.
//!
//! Aggregates all UI sub-plugins (main menu, pause menu, etc.)

use bevy::prelude::UiMaterialPlugin;
use bevy::prelude::*;

use super::scale::update_ui_scale;
use super::systems::{FrostedGlassMaterial, ParchmentMaterial};

use super::action_bar::ActionBarPlugin;
use super::arcanorouter_display::ArcanoRouterDisplayPlugin;
use super::cauldron_menu::CauldronMenuPlugin;
use super::components::{
    load_gun_icon_assets, load_spell_icon_assets, load_unit_compendium_sprite_assets,
    set_default_font,
};
use super::concentration::ConcentrationUIPlugin;
use super::focus::FocusPlugin;
use super::game_over::GameOverPlugin;
use super::gamepad_glyphs::GamepadGlyphsPlugin;
use super::in_game::plugin::InGamePlugin;
use super::link_button::handle_link_click;
use super::loading::LoadingUiPlugin;
use super::main_menu::MainMenuPlugin;
use super::notification::NotificationPlugin;
use super::pause_menu::plugin::PauseMenuPlugin;
use super::roulette_display::RouletteDisplayPlugin;
use super::rune_display::RuneDisplayPlugin;
use super::spell_book::SpellBookPlugin;
use super::splash_screen::SplashScreenPlugin;
use super::systems;
use super::tutorial::TutorialPlugin;
use super::version::VersionPlugin;
use super::weather_bar::WeatherBarPlugin;
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
            FocusPlugin,
            NotificationPlugin,
            SplashScreenPlugin,
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
        ))
        .add_plugins(GamepadGlyphsPlugin)
        .add_plugins((
            WizardTowerPlugin,
            VersionPlugin,
            TutorialPlugin,
            WeatherBarPlugin,
        ))
        .add_plugins(UiMaterialPlugin::<ParchmentMaterial>::default())
        .add_plugins(UiMaterialPlugin::<FrostedGlassMaterial>::default())
        .configure_sets(
            Update,
            ButtonActionSet.run_if(systems::on_message::<MouseClicked>),
        )
        .add_systems(
            Startup,
            (
                set_default_font,
                load_spell_icon_assets,
                load_gun_icon_assets,
                load_unit_compendium_sprite_assets,
            ),
        )
        .add_systems(
            Update,
            (
                update_ui_scale,
                systems::button_click_detection,
                handle_link_click.in_set(ButtonActionSet),
                systems::button_interaction,
                systems::reset_deactivated_buttons,
                systems::sync_front_face_colors,
                systems::apply_gamepad_focus_tint.after(systems::sync_front_face_colors),
                systems::apply_flat_gamepad_focus_tint.after(systems::sync_front_face_colors),
                systems::animate_button_3d,
                systems::apply_parchment_backgrounds,
                systems::apply_frosted_glass_overlays,
                systems::apply_3d_button_structure,
                systems::enforce_active_button_state.after(systems::apply_3d_button_structure),
            ),
        );
    }
}
