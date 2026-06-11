//! Flow field debug visualization.
//!
//! Press F3 to cycle: Off → Attacker → Defender → Off.
//! Renders white arrows above the battlefield showing flow field directions.
//!
//! Press F4 to toggle a moveable debug ball for pinpointing world positions.
//! Arrow keys move it in 5-unit increments on the XZ plane.
//! Position is logged every 5 seconds while active.

mod gizmos;
mod overlay;

pub(crate) use gizmos::FlowFieldDebugMode;
pub(crate) use gizmos::{toggle_flow_field_debug, update_debug_visualization};
pub(crate) use overlay::{
    DebugBallActive, DebugBallLogTimer, log_debug_ball_position, move_debug_ball, toggle_debug_ball,
};
