//! UI focus navigation for gamepad input.
//!
//! A flat spatial navigation model: any button tagged with [`Focusable`] is a
//! candidate. The currently focused entity is tracked in [`FocusedEntity`];
//! D-pad / left-stick presses move focus to the nearest `Focusable` in the
//! pressed direction. `A` emits a `MouseClicked` on the focused entity,
//! reusing the existing button-action pipeline.

pub(crate) mod components;
pub(crate) mod constants;
pub(super) mod navigation;
mod plugin;
pub(crate) mod resources;
pub(super) mod run_conditions;
pub(super) mod scroll;
mod systems;

pub(crate) use components::{
    CrossRowHorizontalNav, Focusable, FocusableFlatBackground, GamepadFocused, GamepadScrollTarget,
    ModalOverlay, NoGamepadFocus, ScrollRevealBounds, TabFocusable,
};
pub(crate) use plugin::FocusPlugin;
pub(crate) use resources::FocusNavInhibit;
