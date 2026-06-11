//! Study tab detail panels, sliders, talent cards, cursor systems.

pub(super) mod allocation;
pub(super) mod cursor;
pub(super) mod detail_panel;
pub(super) mod slider_interaction;
pub(super) mod talent_interaction;

// Re-export the full public surface that the original interaction.rs exposed,
// so all paths at `study_tab::interaction::Item` remain valid.

pub(crate) use allocation::{
    default_graph_offset, handle_alloc_adjust_buttons, handle_alloc_adjust_buttons_hold,
    reset_previous_alloc_on_selection_change, study_back_to_cursor,
    toggle_focus_nav_on_study_selection,
};
pub(crate) use cursor::{
    StudyCursorMode, cleanup_study_cursor_on_area_removed, detect_study_cursor_hover,
    spawn_study_cursor_on_area_added, study_cursor_confirm, study_cursor_edge_scroll,
    study_cursor_trigger_zoom, sync_study_cursor_visual, update_reticle_appearance,
    update_study_cursor,
};
pub(crate) use detail_panel::{update_insight_detail_panel, update_study_detail_panel};
pub(crate) use slider_interaction::{
    handle_detail_slider_interaction, handle_insight_bonus_slider_interaction,
    update_allocation_text, update_detail_sliders, update_graph_node_label_scale,
    update_insight_bonus_allocation_text, update_insight_bonus_rings, update_insight_bonus_sliders,
    update_pending_insight_display, update_star_sky_time,
};
pub(crate) use talent_interaction::{
    clear_talent_hover_description, handle_talent_card_clicks, update_talent_hover_description,
};
