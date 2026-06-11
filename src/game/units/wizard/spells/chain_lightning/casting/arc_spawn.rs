//! Spawns jagged lightning arc visuals between two world-space points.

use super::super::components::ChainLightningArc;
use super::super::constants;
use super::super::constants::arc_width_at_depth;
use crate::game::units::wizard::spells::lightning_bolt::{
    LightningBoltConfig, spawn_lightning_bolt,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// Spawns a jagged lightning arc visual between two points. Depth-0 bolts run
/// along the ground; deeper bounces gain a small parabolic arch. The bolt
/// re-jitters every frame for a crackling look (see `lightning_bolt` module).
///
/// A `ChainLightningArc` marker is attached to the parent so the multiplayer
/// snapshot collector can serialize the start/end of each bolt.
pub(crate) fn spawn_arc(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    start: Vec3,
    end: Vec3,
    depth: u32,
    empowerment: f32,
) {
    let arc_width = arc_width_at_depth(depth, empowerment);

    // Depth 0 (initial bolt from wizard) runs straight; deeper splits arc.
    let horizontal_dist = Vec3::new(start.x - end.x, 0.0, start.z - end.z).length();
    let height_factor = constants::ARC_HEIGHT_FACTOR + constants::ARC_HEIGHT_GROWTH * depth as f32;
    let peak_height = if depth == 0 {
        0.0
    } else {
        horizontal_dist * height_factor
    };

    let jitter_amplitude =
        constants::ARC_JITTER_BASE * constants::ARC_JITTER_DEPTH_FALLOFF.powi(depth as i32);
    let fork_count = if depth == 0 { 2 } else { 1 };

    let config = LightningBoltConfig {
        width: arc_width,
        lifetime: constants::ARC_LIFETIME,
        peak_height,
        jitter_amplitude,
        segments: constants::ARC_SEGMENTS,
        fork_count,
        fork_segments: 3,
        fork_length: arc_width * 4.0 + 12.0,
        afterimage_duration: constants::ARC_AFTERIMAGE_DURATION,
    };

    let bolt = spawn_lightning_bolt(
        commands,
        assets.unit_rect.clone(),
        assets.chain_lightning_arc.clone(),
        start,
        end,
        config,
    );

    commands
        .entity(bolt)
        .insert(ChainLightningArc { start, end });
}
