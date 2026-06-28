//! Gamepad input systems.
//!
//! Translates raw gamepad axes and buttons into the same messages that the
//! existing keyboard/mouse pipeline uses. Downstream gameplay and UI systems
//! are unaware of the source — they read `CorrectedCursorPosition`,
//! `MouseLeftPressed/Held/Released`, `MouseRightPressed/Held/Released`, and
//! `MouseClicked` as before.

mod connection;
mod cursor;
mod gilrs_adapter;
mod navigation;

// Re-export everything consumed by plugin.rs (pub(super) in original → now pub(crate) on items,
// re-exported here so the path `gamepad::systems::Foo` is preserved).
pub(crate) use connection::{
    detect_active_input_device, pause_on_controller_unplug, toggle_cursor_visibility,
};
pub(crate) use cursor::{apply_deadzone_and_curve, shape_stick, update_virtual_cursor};
pub(crate) use gilrs_adapter::fill_action_state_from_gilrs;
pub(crate) use navigation::{
    emit_ui_confirm_back_messages, sync_gamepad_settings, translate_triggers_to_mouse_messages,
    update_radial_hovered_slot,
};
