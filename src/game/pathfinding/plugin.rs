//! Pathfinding plugin.

use bevy::prelude::*;

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
                    // Handle obstacle changes
                    handle_obstacle_events,
                    // Apply completed async rebuilds
                    apply_completed_rebuilds,
                )
                    .chain()
                    .run_if(resource_exists::<PathfindingGrid>)
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                // Sample flow fields MUST run in VelocitySystemSet (before movement calculations)
                // VelocitySystemSet is already gated by is_gameplay_running
                sample_flow_fields
                    .in_set(VelocitySystemSet)
                    .run_if(resource_exists::<PathfindingGrid>),
            );
    }
}
