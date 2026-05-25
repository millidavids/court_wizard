//! Pathfinding plugin.

use bevy::prelude::*;

#[cfg(debug_assertions)]
use super::debug::{self, DebugBallActive, DebugBallLogTimer, FlowFieldDebugMode};
use super::messages::ObstacleChanged;
use super::resources::PathfindingGrid;
use super::runtime::*;
use super::setup::*;
use super::staging::WaveStagingPlan;
use crate::game::plugin::VelocitySystemSet;
use crate::game::run_conditions::is_gameplay_running;

/// Plugin that handles flow field pathfinding for units.
pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register message channels
            .add_message::<ObstacleChanged>()
            // Wave staging timers (timeout-based force activation)
            .init_resource::<WaveStagingTimers>()
            // Wave staging plan (which staging points each wave uses)
            .init_resource::<WaveStagingPlan>()
            // Handle obstacle changes (updates base_costs) — runs during loading
            // so terrain spawned via the spawn queue is registered immediately.
            .add_systems(
                Update,
                handle_obstacle_events.run_if(resource_exists::<PathfindingGrid>),
            )
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
                    // Poll completed rebuilds and immediately spawn new ones
                    continuous_flow_field_rebuild,
                )
                    .chain()
                    .run_if(resource_exists::<PathfindingGrid>)
                    .run_if(is_gameplay_running),
            )
            // Wave activation: tag new attackers, check wave thresholds.
            // Single-player only — multiplayer has no staging phase, armies
            // are spawned in full at match start and run straight at their
            // targets without ever being tagged as `StagingAttacker`.
            .add_systems(
                Update,
                (tag_new_attackers, check_wave_activation)
                    .chain()
                    .run_if(resource_exists::<PathfindingGrid>)
                    .run_if(is_gameplay_running)
                    .run_if(in_state(crate::state::AppState::InGame)),
            )
            // Game-speed management runs whenever a game is in progress —
            // NOT gated on `is_gameplay_running` so it can actively DROP the
            // clock back to baseline when the player opens the spell book or
            // cauldron menu (which take us out of Running). Without this it
            // stayed stuck at 3x.
            .add_systems(
                Update,
                manage_staging_speedup.run_if(resource_exists::<PathfindingGrid>),
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
            );

        // Debug visualization (F3 flow-field, F4 debug ball) — debug builds only.
        #[cfg(debug_assertions)]
        app.init_resource::<FlowFieldDebugMode>()
            .init_resource::<DebugBallActive>()
            .init_resource::<DebugBallLogTimer>()
            .add_systems(
                Update,
                (
                    debug::toggle_flow_field_debug,
                    debug::update_debug_visualization.run_if(resource_exists::<PathfindingGrid>),
                )
                    .run_if(is_gameplay_running),
            )
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
