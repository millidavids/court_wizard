use bevy::prelude::*;

// Color constants using HSL for better color manipulation
pub const NORMAL_BUTTON: Color = Color::hsla(0.0, 0.0, 0.15, 1.0);
pub const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.9, 1.0);
pub const HEADER_COLOR: Color = Color::hsla(0.0, 0.0, 1.0, 1.0);
pub const BACKGROUND_COLOR: Color = Color::hsla(0.0, 0.0, 0.05, 1.0);

/// Adjusts a color's lightness for hover state
pub fn hovered_color(base: Color) -> Color {
    match base {
        Color::Hsla(color) => Color::hsla(
            color.hue,
            color.saturation,
            color.lightness + 0.1,
            color.alpha,
        ),
        _ => base,
    }
}

/// Adjusts a color's lightness for pressed state
pub fn pressed_color(base: Color) -> Color {
    match base {
        Color::Hsla(color) => Color::hsla(
            color.hue,
            color.saturation,
            color.lightness + 0.2,
            color.alpha,
        ),
        _ => base,
    }
}

// Size constants
pub const BUTTON_WIDTH: Val = Val::Px(300.0);
pub const BUTTON_HEIGHT: Val = Val::Px(65.0);
pub const BUTTON_MARGIN: Val = Val::Px(10.0);
pub const FONT_SIZE_HEADER: f32 = 60.0;
pub const FONT_SIZE_BUTTON: f32 = 32.0;
pub const FONT_SIZE_LABEL: f32 = 24.0;

/// Creates the standard button bundle with consistent styling
pub fn button_bundle() -> Node {
    Node {
        width: BUTTON_WIDTH,
        height: BUTTON_HEIGHT,
        margin: UiRect::all(BUTTON_MARGIN),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

/// Creates a settings control button (smaller than main menu buttons)
pub fn settings_button_bundle() -> Node {
    Node {
        width: Val::Px(200.0),
        height: Val::Px(45.0),
        margin: UiRect::all(Val::Px(5.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

/// Creates a small increment/decrement button
pub fn adjust_button_bundle() -> Node {
    Node {
        width: Val::Px(40.0),
        height: Val::Px(40.0),
        margin: UiRect::all(Val::Px(5.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

/// Creates a row for a single setting with label and control
pub fn setting_row() -> Node {
    Node {
        width: Val::Percent(100.0),
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        margin: UiRect::all(Val::Px(10.0)),
        ..default()
    }
}
