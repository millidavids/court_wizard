//! Cauldron menu interaction handlers.

mod brew_actions;
mod ingredient_clicks;

pub(super) use brew_actions::button_action;
pub(super) use brew_actions::despawn_cauldron_menu_ui;
pub(super) use ingredient_clicks::sync_toggle_button_states;
pub(super) use ingredient_clicks::update_detail_panel_on_selection_change;
