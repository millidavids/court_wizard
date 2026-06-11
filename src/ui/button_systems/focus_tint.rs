use bevy::prelude::*;

use super::super::color_utils::blend_over;
use super::super::components::{ButtonColors, ButtonEdge, ButtonFront};
use super::super::constants::BUTTON_HOVER_BG_TINT;
use super::super::focus::{FocusableFlatBackground, GamepadFocused};
use super::sync::opaque;

/// Tints the 3D front-face background purple when a button is gamepad-focused,
/// and restores the base (charcoal) color when focus leaves. This is the only
/// visual that distinguishes controller focus from mouse hover — mouse hover
/// keeps the default bg and only tweaks the border / outline / glow.
#[allow(clippy::type_complexity)]
pub fn apply_gamepad_focus_tint(
    focused: Query<
        (&ButtonColors, &Children),
        Or<(
            Added<GamepadFocused>,
            (With<GamepadFocused>, Changed<ButtonColors>),
        )>,
    >,
    mut removed: RemovedComponents<GamepadFocused>,
    all_buttons: Query<(&ButtonColors, &Children)>,
    mut front_query: Query<&mut BackgroundColor, (With<ButtonFront>, Without<ButtonEdge>)>,
) {
    for (colors, children) in &focused {
        let tinted = blend_over(opaque(colors.background), BUTTON_HOVER_BG_TINT);
        for child in children.iter() {
            if let Ok(mut bg) = front_query.get_mut(child) {
                *bg = BackgroundColor(tinted);
            }
        }
    }

    for entity in removed.read() {
        if let Ok((colors, children)) = all_buttons.get(entity) {
            let base = opaque(colors.background);
            for child in children.iter() {
                if let Ok(mut bg) = front_query.get_mut(child) {
                    *bg = BackgroundColor(base);
                }
            }
        }
    }
}

/// Tints the entity's own `BackgroundColor` purple when gamepad-focused, for
/// flat focusables that don't wrap a `ButtonFront` child (e.g. text input
/// fields). Stores the base color in `FocusableFlatBackground` so the tint
/// can be cleanly removed on unfocus.
pub fn apply_flat_gamepad_focus_tint(
    mut just_focused: Query<
        (&FocusableFlatBackground, &mut BackgroundColor),
        Added<GamepadFocused>,
    >,
    mut removed: RemovedComponents<GamepadFocused>,
    mut un_focused: Query<
        (&FocusableFlatBackground, &mut BackgroundColor),
        Without<GamepadFocused>,
    >,
) {
    for (tint, mut bg) in &mut just_focused {
        *bg = BackgroundColor(blend_over(opaque(tint.base), BUTTON_HOVER_BG_TINT));
    }
    for entity in removed.read() {
        if let Ok((tint, mut bg)) = un_focused.get_mut(entity) {
            *bg = BackgroundColor(tint.base);
        }
    }
}
