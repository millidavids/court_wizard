//! Compendium setup and click handlers.

mod endless;
mod handlers;
mod item_spawners;
mod layout;
mod rebuild;
mod roguelite;

pub(super) use handlers::{
    handle_copy_seed, handle_item_click, handle_tab_click, handle_toggle_save_run,
    update_item_active_state,
};
pub(super) use item_spawners::team_label_color;
pub(super) use layout::{setup_main_menu, setup_pause_menu};
pub(super) use rebuild::rebuild_on_state_change;
