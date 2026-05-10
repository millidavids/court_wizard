//! Top-level UI plugin.
//!
//! Aggregates all UI sub-plugins (main menu, pause menu, etc.)

use bevy::prelude::UiMaterialPlugin;
use bevy::prelude::*;
use bevy::ui::UiScale as BevyUiScale;
use bevy::window::PrimaryWindow;

use super::systems::{FrostedGlassMaterial, ParchmentMaterial};

use super::action_bar::ActionBarPlugin;
use super::arcanorouter_display::ArcanoRouterDisplayPlugin;
use super::cauldron_menu::CauldronMenuPlugin;
use super::components::{
    load_gun_icon_assets, load_spell_icon_assets, load_unit_compendium_sprite_assets,
    set_default_font,
};
use super::concentration::ConcentrationUIPlugin;
use super::link_button::handle_link_click;
use super::focus::FocusPlugin;
use super::game_over::GameOverPlugin;
use super::gamepad_glyphs::GamepadGlyphsPlugin;
use super::in_game::plugin::InGamePlugin;
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

/// Updates the global UI scale based on the 16:9 viewport width.
///
/// Uses Bevy's built-in UiScale resource to scale all UI elements.
/// Calculates scale factor relative to a base width of 1920px, then applies
/// a 1.5x multiplier to make everything larger.
/// Uses the CRT viewport width (which enforces 16:9) rather than the
/// full window width, so UI scales correctly with letterboxing/pillarboxing.
fn update_ui_scale(
    mut ui_scale: ResMut<BevyUiScale>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    crt_query: Query<&crate::game::crt_effect::CrtEffectSettings>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    // Use viewport width from CRT settings (fraction of window)
    let viewport_fraction = if let Ok(settings) = crt_query.single() {
        settings.viewport_w
    } else {
        1.0
    };
    let logical_width = window.width() * viewport_fraction;

    const BASE_WIDTH: f32 = 1920.0;
    const SCALE_MULTIPLIER: f32 = 1.5;
    let new_scale = (logical_width / BASE_WIDTH) * SCALE_MULTIPLIER;

    if (ui_scale.0 - new_scale).abs() > 0.001 {
        ui_scale.0 = new_scale;
    }
}
