//! UI button systems: interaction, 3D structure, gamepad focus, color sync.

pub(crate) mod active;
pub(crate) mod button_animation;
pub(crate) mod click;
pub(crate) mod focus_tint;
pub(crate) mod structure;
pub(crate) mod sync;

// click
pub use click::{button_click_detection, button_interaction, on_message};

// structure
pub use structure::apply_3d_button_structure;
pub(crate) use structure::{edge_color, insert_material_background};

// button_animation
pub use button_animation::animate_button_3d;

// active
pub use active::{enforce_active_button_state, reset_deactivated_buttons};

// focus_tint
pub use focus_tint::{apply_flat_gamepad_focus_tint, apply_gamepad_focus_tint};

// sync
pub use sync::sync_front_face_colors;
pub(crate) use sync::{opaque, scale_font_by_text_width};
