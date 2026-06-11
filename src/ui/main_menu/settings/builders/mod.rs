//! Settings tab builders and panel setup.

mod controls_tab;
mod game_tab;
mod keybind_systems;
mod setup;
mod tab_panels;
mod tab_systems;

pub(crate) use keybind_systems::spawn_confirmation_popup;
pub use keybind_systems::{
    capture_key_input, key_binding_button_action, key_capture_inactive, update_key_binding_text,
};
pub use setup::{setup_main_menu, setup_pause_menu};
pub use tab_systems::{
    handle_settings_tab_click, rebuild_settings_content, resolution_button_action,
    update_resolution_selection, update_resolution_visibility,
};
