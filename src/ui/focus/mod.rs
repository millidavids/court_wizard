//! UI focus navigation for gamepad input.
//!
//! A flat spatial navigation model: any button tagged with [`Focusable`] is a
//! candidate. The currently focused entity is tracked in [`FocusedEntity`];
//! D-pad / left-stick presses move focus to the nearest `Focusable` in the
//! pressed direction. `A` presses and activates the focused button, reusing
//! the mouse's `Pressed` + `Activate` → `MouseClicked` pipeline.

pub(crate) mod components;
mod confirm;
pub(crate) mod constants;
mod memory;
pub(super) mod navigation;
mod plugin;
pub(crate) mod resources;
pub(super) mod run_conditions;
pub(super) mod scroll;
mod systems;

pub(crate) use components::{
    ConsumeHorizontalNav, CrossRowHorizontalNav, DisabledTab, Focusable, FocusableFlatBackground,
    GamepadFocused, GamepadScrollTarget, ModalOverlay, NoGamepadFocus, ScrollRevealBounds,
    TabFocusable,
};
pub(crate) use plugin::FocusPlugin;
pub(crate) use resources::FocusNavInhibit;
