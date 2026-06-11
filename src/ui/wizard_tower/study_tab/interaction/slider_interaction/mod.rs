pub(super) mod slider_spawn;
pub(super) mod slider_systems;

pub(crate) use slider_spawn::{spawn_detail_unified_slider, spawn_insight_bonus_slider};
pub(crate) use slider_systems::{
    handle_detail_slider_interaction, handle_insight_bonus_slider_interaction,
    update_allocation_text, update_detail_sliders, update_graph_node_label_scale,
    update_insight_bonus_allocation_text, update_insight_bonus_rings, update_insight_bonus_sliders,
    update_pending_insight_display, update_star_sky_time,
};
