//! Pathfinding plugin.

use bevy::prelude::*;

use super::debug::{self, DebugBallActive, DebugBallLogTimer, FlowFieldDebugMode};
use super::messages::ObstacleChanged;
use super::resources::PathfindingGrid;
use super::systems::*;
use crate::game::plugin::VelocitySystemSet;
use crate::game::run_conditions::is_gameplay_running;

/// Plugin that handles flow field pathfinding for units.
pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register message channels
            .add_message::<ObstacleChanged>()
            // Debug visualization
            .init_resource::<FlowFieldDebugMode>()
            // Wave staging timers (timeout-based force activation)
            .init_resource::<WaveStagingTimers>()
            // Flow field management: continuously rebuild all fields in parallel.
            .add_systems(
                Update,
                (
                    // Generate initial fields when Defender King spawns
                    generate_initial_fields,
                    // Track King's closest enemy target for defender field
                    update_king_target,
                    // Tick defender rally delay (no-enemies → spawn center)
                    tick_defender_rally_delay,
                    // Handle obstacle changes (updates base_costs)
                    handle_obstacle_events,
                    // Poll completed rebuilds and immediately spawn new ones
                    continuous_flow_field_rebuild,
                )
                    .chain()
                    .run_if(resource_exists::<PathfindingGrid>)
                    .run_if(is_gameplay_running),
            )
            // Wave activation: tag new attackers, check wave thresholds, manage speedup
            .add_systems(
                Update,
                (
                    tag_new_attackers,
                    check_wave_activation,
                    manage_staging_speedup,
                )
                    .chain()
                    .run_if(resource_exists::<PathfindingGrid>)
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                (
                    // Sample flow fields MUST run in VelocitySystemSet (before movement calculations)
                    // VelocitySystemSet is already gated by is_gameplay_running
                    sample_flow_fields.run_if(resource_exists::<PathfindingGrid>),
                    // Auto-insert StuckDetection on new flow field entities
                    init_stuck_detection,
                    // Detect and recover stuck units
                    detect_and_recover_stuck_units,
                )
                    .in_set(VelocitySystemSet),
            )
            // Debug visualization (F3 toggle + arrow rendering)
            .add_systems(
                Update,
                (
                    debug::toggle_flow_field_debug,
                    debug::update_debug_visualization.run_if(resource_exists::<PathfindingGrid>),
                )
                    .run_if(is_gameplay_running),
            )
            // Debug ball (F4 toggle + arrow key movement + position logging)
            .init_resource::<DebugBallActive>()
            .init_resource::<DebugBallLogTimer>()
            .add_systems(
                Update,
                (
                    debug::toggle_debug_ball,
                    debug::move_debug_ball,
                    debug::log_debug_ball_position,
                )
                    .run_if(is_gameplay_running),
            );
    }
}
