//! UI button systems: hover/press visuals, clicks, 3D structure, gamepad focus, color sync.

pub(crate) mod activate;
pub(crate) mod active;
pub(crate) mod button_animation;
pub(crate) mod focus_tint;
pub(crate) mod pointer_visual;
pub(crate) mod release;
pub(crate) mod structure;
pub(crate) mod sync;
pub(crate) mod visuals;

// activate
pub(crate) use activate::forward_activate_as_mouse_clicked;

// pointer_visual
pub(crate) use pointer_visual::{PointerState, PointerVisual, PointerVisualChanges};

// release
pub(crate) use release::clear_stale_presses;

// visuals
pub(crate) use visuals::update_button_visuals;

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
