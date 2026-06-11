//! Tab interaction systems: tab click, content rebuild, resolution controls.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::config::{DisplayMode, GameConfig, SavedWindowedGeometry};
use crate::game::input::messages::MouseClicked;
use crate::ui::gamepad_glyphs::{CurrentControllerGlyphStyle, GamepadGlyphFonts};

use super::super::components::{
    ButtonColors, ResolutionPreset, ResolutionRow, SelectedOption, SettingsContentContainer,
    SettingsTab, SettingsTabButton, SettingsTabState,
};
use super::super::constants::{
    ACTIVE_TAB_BG, ACTIVE_TAB_BORDER, BUTTON_BACKGROUND, BUTTON_BORDER, INACTIVE_TAB_BG,
    SELECTED_BACKGROUND, SELECTED_BORDER, TAB_BORDER_COLOR,
};
use super::controls_tab::spawn_controls_tab;
use super::game_tab::spawn_game_tab;
use super::tab_panels::{
    spawn_accessibility_tab, spawn_audio_tab, spawn_controller_tab, spawn_graphics_tab,
};

/// Handles settings tab button clicks.
pub fn handle_settings_tab_click(
    mut button_clicked: MessageReader<MouseClicked>,
    tab_query: Query<&SettingsTabButton>,
    mut state: ResMut<SettingsTabState>,
) {
    for event in button_clicked.read() {
        if let Ok(tab_btn) = tab_query.get(event.button)
            && state.active_tab != tab_btn.0
        {
            state.active_tab = tab_btn.0;
        }
    }
}

/// Rebuilds the settings content area when the active tab changes.
/// Also updates tab button styling.
#[allow(clippy::too_many_arguments)]
pub fn rebuild_settings_content(
    mut commands: Commands,
    state: Res<SettingsTabState>,
    glyph_style: Res<CurrentControllerGlyphStyle>,
    glyph_fonts: Option<Res<GamepadGlyphFonts>>,
    game_config: Res<GameConfig>,
    saved_geometry: Res<SavedWindowedGeometry>,
    bindings: Res<crate::config::InputBindings>,
    container_query: Query<Entity, With<SettingsContentContainer>>,
    tab_buttons: Query<(Entity, &SettingsTabButton, Option<&Children>)>,
    mut tab_colors: Query<(&mut BackgroundColor, &mut BorderColor, &mut ButtonColors)>,
    mut front_colors: Query<
        (&mut BackgroundColor, &mut BorderColor),
        (
            With<crate::ui::components::ButtonFront>,
            Without<ButtonColors>,
        ),
    >,
) {
    let state_changed = state.is_changed();
    let glyph_style_changed =
        glyph_style.is_changed() && state.active_tab == SettingsTab::Controller;
    if !state_changed && !glyph_style_changed {
        return;
    }

    // Update tab button styling
    for (entity, tab_btn, children) in &tab_buttons {
        let is_active = tab_btn.0 == state.active_tab;
        let (bg, border) = if is_active {
            commands
                .entity(entity)
                .insert(crate::ui::components::ButtonActive);
            (ACTIVE_TAB_BG, ACTIVE_TAB_BORDER)
        } else {
            commands
                .entity(entity)
                .remove::<crate::ui::components::ButtonActive>();
            (INACTIVE_TAB_BG, TAB_BORDER_COLOR)
        };
        if let Ok((mut bg_color, mut border_color, mut colors)) = tab_colors.get_mut(entity) {
            *bg_color = BackgroundColor(bg);
            *border_color = BorderColor::all(border);
            colors.background = bg;
            colors.border = border;
        }
        // Also update the 3D front face child.
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok((mut front_bg, mut front_border)) = front_colors.get_mut(child) {
                    *front_bg = crate::ui::systems::opaque(bg).into();
                    *front_border = BorderColor::all(border);
                }
            }
        }
    }

    // Rebuild content
    let Ok(container) = container_query.single() else {
        return;
    };
    commands.entity(container).despawn_related::<Children>();
    commands
        .entity(container)
        .with_children(|parent| match state.active_tab {
            SettingsTab::Graphics => spawn_graphics_tab(parent, &game_config, &saved_geometry),
            SettingsTab::Audio => spawn_audio_tab(parent, &game_config),
            SettingsTab::Game => spawn_game_tab(parent, &game_config),
            SettingsTab::Controls => spawn_controls_tab(parent, &bindings),
            SettingsTab::Controller => {
                spawn_controller_tab(parent, &game_config, glyph_fonts.as_deref(), glyph_style.0)
            }
            SettingsTab::Accessibility => spawn_accessibility_tab(parent, &game_config),
        });
}

/// Handles resolution preset button clicks.
/// Clears the camera viewport before resizing to avoid wgpu scissor rect crashes.
pub fn resolution_button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&ResolutionPreset>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<Camera3d>>,
    mut saved_geometry: ResMut<SavedWindowedGeometry>,
    game_config: Res<GameConfig>,
) {
    if game_config.display_mode != DisplayMode::Windowed {
        button_clicked.read();
        return;
    }
    for event in button_clicked.read() {
        if let Ok(preset) = button_query.get(event.button)
            && let Ok(mut window) = windows.single_mut()
        {
            for mut camera in &mut cameras {
                camera.viewport = None;
            }
            window.resolution.set(preset.width, preset.height);
            saved_geometry.width = preset.width;
            saved_geometry.height = preset.height;
        }
    }
}

/// Updates resolution button highlighting based on current window geometry.
pub fn update_resolution_selection(
    mut commands: Commands,
    saved_geometry: Res<SavedWindowedGeometry>,
    mut preset_buttons: Query<(
        Entity,
        &ResolutionPreset,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut ButtonColors,
        Option<&Children>,
    )>,
    mut front_query: Query<
        (&mut BackgroundColor, &mut BorderColor),
        (
            With<crate::ui::components::ButtonFront>,
            Without<ResolutionPreset>,
        ),
    >,
) {
    if !saved_geometry.is_changed() {
        return;
    }
    for (entity, preset, mut bg, mut border, mut colors, children) in &mut preset_buttons {
        let matches = (preset.width - saved_geometry.width).abs() < 1.0
            && (preset.height - saved_geometry.height).abs() < 1.0;
        let (new_bg, new_border) = if matches {
            commands
                .entity(entity)
                .insert((SelectedOption, crate::ui::components::ButtonActive));
            (SELECTED_BACKGROUND, SELECTED_BORDER)
        } else {
            commands
                .entity(entity)
                .remove::<SelectedOption>()
                .remove::<crate::ui::components::ButtonActive>();
            (BUTTON_BACKGROUND, BUTTON_BORDER)
        };
        *bg = BackgroundColor(new_bg);
        *border = BorderColor::all(new_border);
        colors.background = new_bg;
        colors.border = new_border;

        if let Some(children) = children {
            for child in children.iter() {
                if let Ok((mut front_bg, mut front_border)) = front_query.get_mut(child) {
                    *front_bg = crate::ui::systems::opaque(new_bg).into();
                    *front_border = BorderColor::all(new_border);
                }
            }
        }
    }
}

/// Shows/hides the resolution row based on display mode.
pub fn update_resolution_visibility(
    game_config: Res<GameConfig>,
    mut resolution_row: Query<&mut Visibility, With<ResolutionRow>>,
) {
    if !game_config.is_changed() {
        return;
    }
    for mut vis in &mut resolution_row {
        *vis = if game_config.display_mode == DisplayMode::Windowed {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
