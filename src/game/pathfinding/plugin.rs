//! Pathfinding plugin.

use bevy::prelude::*;

use super::debug::{self, FlowFieldDebugMode};
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
            // Note: initialize_pathfinding is now called via the loading spawn queue
            // Flow field management: only the host needs to generate/rebuild fields.
            // Gated by is_gameplay_running so they run for both SP and MP host.
            .add_systems(
                Update,
                (
                    // Generate initial fields when Defender King spawns
                    generate_initial_fields,
                    // Track King movement for attacker field
                    update_king_position,
                    // Update King's target for defender field
                    update_king_target,
                    // Tick defender rally delay (no-enemies → spawn center)
                    tick_defender_rally_delay,
                    // Handle obstacle changes (updates costs, starts debounce timer)
                    handle_obstacle_events,
                    // Flush debounced rebuilds after the batching window expires
                    flush_debounced_rebuilds,
                    // Process rebuild queue (one rebuild per frame max)
                    process_rebuild_queue,
                    // Apply completed async rebuilds
                    apply_completed_rebuilds,
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
                    debug::update_debug_visualization
                        .run_if(resource_exists::<PathfindingGrid>),
                )
                    .run_if(is_gameplay_running),
            );
    }
}
