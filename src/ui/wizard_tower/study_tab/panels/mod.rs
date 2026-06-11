//! Study tab panel setup, graph rendering, and node systems.

pub(crate) mod commit;
pub(crate) mod graph_nav;
pub(crate) mod helpers;
pub(crate) mod positions;
pub(crate) mod spawn;
pub(crate) mod spawn_content;

// pub(crate) items — same surface as the original panels.rs
pub(crate) use commit::handle_study_button_actions;
pub(crate) use graph_nav::{
    animate_graph_view, animate_to_default_view, clamp_view_offset, handle_graph_node_clicks,
    handle_graph_pan, handle_graph_zoom, process_pending_graph_layout_refresh,
};
pub(crate) use helpers::{
    calculate_talent_font_size, compute_slider_fracs, count_researched_spells, element_color,
    is_prereq_met, is_spell_unlocked,
};
pub(crate) use positions::{
    update_graph_edge_positions, update_graph_node_borders, update_graph_node_positions,
    update_insight_edge_positions, update_insight_node_borders, update_insight_node_positions,
};
pub(crate) use spawn::{PendingGraphLayoutRefresh, build_study_panels, cleanup_study_resources};

#[cfg(debug_assertions)]
pub(crate) use spawn::DebugInsightButton;
