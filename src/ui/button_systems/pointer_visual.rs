//! Shared hover/press state for UI widgets: mouse picking plus controller focus.

use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::system::SystemParam;
use bevy::picking::hover::DirectlyHovered;
use bevy::prelude::*;
use bevy::ui::Pressed;

use crate::ui::focus::GamepadFocused;

/// How a widget should look right now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PointerVisual {
    Rest,
    Hover,
    Press,
}

/// Query data for [`PointerVisual::of`].
pub(crate) type PointerState = (&'static DirectlyHovered, Has<Pressed>, Has<GamepadFocused>);

impl PointerVisual {
    /// Controller focus counts as hover. A press only shows while the widget is
    /// still hovered, so dragging off a held button looks released.
    pub(crate) fn resolve(hovered: bool, pressed: bool, gamepad_focused: bool) -> Self {
        match (hovered || gamepad_focused, pressed) {
            (true, true) => Self::Press,
            (true, false) => Self::Hover,
            (false, _) => Self::Rest,
        }
    }

    pub(crate) fn of((hovered, pressed, gamepad_focused): (&DirectlyHovered, bool, bool)) -> Self {
        Self::resolve(hovered.get(), pressed, gamepad_focused)
    }
}

/// Entities whose [`PointerVisual`] may have changed since this system last ran.
#[derive(SystemParam)]
pub(crate) struct PointerVisualChanges<'w, 's> {
    changed: Query<
        'w,
        's,
        Entity,
        Or<(
            Changed<DirectlyHovered>,
            Added<Pressed>,
            Added<GamepadFocused>,
        )>,
    >,
    released: RemovedComponents<'w, 's, Pressed>,
    unfocused: RemovedComponents<'w, 's, GamepadFocused>,
}

impl PointerVisualChanges<'_, '_> {
    pub(crate) fn entities(&mut self) -> EntityHashSet {
        let mut entities: EntityHashSet = self.changed.iter().collect();
        entities.extend(self.released.read());
        entities.extend(self.unfocused.read());
        entities
    }
}

#[cfg(test)]
mod tests {
    use super::PointerVisual::{self, Hover, Press, Rest};

    #[test]
    fn resolves_hover_press_and_focus() {
        assert_eq!(PointerVisual::resolve(false, false, false), Rest);
        assert_eq!(PointerVisual::resolve(true, false, false), Hover);
        assert_eq!(PointerVisual::resolve(true, true, false), Press);
        assert_eq!(PointerVisual::resolve(false, false, true), Hover);
        assert_eq!(PointerVisual::resolve(false, true, true), Press);
        // Held, then dragged off.
        assert_eq!(PointerVisual::resolve(false, true, false), Rest);
    }
}
