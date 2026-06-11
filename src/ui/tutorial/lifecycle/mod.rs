//! Tutorial triggers and lifecycle.

mod handlers;
mod highlight;
mod overlay;
mod persistence;
mod triggers;

// `pub(super)` items from the original lifecycle.rs — visible to the tutorial
// module (our parent), which is the same scope `pub(super)` granted before.
pub(super) use handlers::{
    cleanup_tutorial, handle_next_button, handle_skip_button, update_tutorial_content,
};
pub(super) use highlight::{animate_glow, apply_highlight};
pub(super) use overlay::{position_tutorial_panel, spawn_tutorial_overlay};
pub(super) use triggers::{
    drain_pending_tutorials, enforce_tutorial_modality, trigger_cauldron_tutorial,
    trigger_in_game_tutorial, trigger_kbm_menus_tutorial, trigger_spell_book_tutorial,
    trigger_study_spell_selected_tutorial, trigger_wizard_select_tutorial,
    trigger_wizard_tower_tab_tutorial,
};

// `pub(crate)` items from the original lifecycle.rs — visible crate-wide.
pub(crate) use persistence::{load_tutorial_progress, reset_tutorial_progress};
