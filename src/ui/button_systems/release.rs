//! Drops mouse presses once the mouse button is up.
//!
//! Bevy only releases what's hovered at release time, so a widget that was
//! covered or scrolled away while held keeps `Pressed`. A stuck press blocks
//! spell casting and drives the study +/- auto-repeat.

use bevy::picking::pointer::{PointerId, PointerPressState};
use bevy::prelude::*;
use bevy::ui::Pressed;

use crate::ui::focus::GamepadFocused;

/// Runs after picking, so the release has already clicked what it hit. The
/// controller's press on its focused button is left alone.
pub(crate) fn clear_stale_presses(
    pointers: Query<(&PointerId, &PointerPressState)>,
    pressed: Query<Entity, (With<Pressed>, Without<GamepadFocused>)>,
    mut commands: Commands,
) {
    let mouse_down = pointers
        .iter()
        .any(|(id, press)| *id == PointerId::Mouse && press.is_primary_pressed());
    if mouse_down {
        return;
    }
    for entity in &pressed {
        commands.entity(entity).try_remove::<Pressed>();
    }
}
