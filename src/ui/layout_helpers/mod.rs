mod buttons;
mod materials;
mod navigation;
mod panels;
mod scroll;
mod slider;

// pub items
pub use buttons::{cleanup_screen, spawn_button, spawn_page_header, spawn_title_with_shadow};
pub use materials::{apply_frosted_glass_overlays, apply_parchment_backgrounds};
pub use navigation::{
    consume_mouse_on_exit, escape_to_landing, escape_to_pause_main, escape_to_running,
};
pub use panels::{
    default_content_node, spawn_left_detail_panel, spawn_page_container, spawn_right_scroll_panel,
    spawn_scrollable_left_detail_panel,
};
pub use scroll::handle_scroll;

// pub(crate) items
pub(crate) use materials::{FrostedGlassMaterial, ParchmentMaterial};
pub(crate) use navigation::{handle_main_menu_back_button, handle_pause_menu_back_button};
pub(crate) use panels::scroll_area_style;
pub(crate) use slider::{SliderRowConfig, spawn_dot_leader, spawn_slider_row};
