use bevy::prelude::*;

use crate::state::{AppState, InGameState};

use super::battlefield::BattlefieldPlugin;
use super::constants::ATTACK_CYCLE_DURATION;
use super::input::InputPlugin;
use super::resources::{CurrentLevel, GameOutcome, KillStats};
use super::shared_systems;
use super::systems;
use super::units::UnitsPlugin;
use super::win_lose_systems;

/// Global attack cycle timer resource.
///
/// Cycles from 0.0 to CYCLE_DURATION seconds. Units track which time offset
/// in the cycle they last attacked and can only attack again when the timer
/// cycles back to that offset. This naturally staggers attacks across all units.
#[derive(Resource)]
pub struct GlobalAttackCycle {
    /// Current time in the cycle (0.0 to CYCLE_DURATION)
    pub current_time: f32,
    /// Duration of one complete cycle in seconds
    pub cycle_duration: f32,
}

impl Default for GlobalAttackCycle {
    fn default() -> Self {
        Self {
            current_time: 0.0,
            cycle_duration: ATTACK_CYCLE_DURATION,
        }
    }
}

impl GlobalAttackCycle {
    /// Advances the cycle timer by delta time, wrapping back to 0 after cycle_duration.
    pub fn tick(&mut self, delta: f32) {
        self.current_time = (self.current_time + delta) % self.cycle_duration;
    }
}

/// System set for velocity calculation systems.
///
/// These systems use immutable queries to calculate velocities and accelerations:
/// - Targeting: Sets TargetingVelocity based on nearest enemy
/// - Flocking/Separation: Adds forces to Acceleration
///
/// All systems in this set can run in parallel since they only read Transform.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct VelocitySystemSet;

/// System set for unit movement systems.
///
/// Movement systems query their specific unit type (mutable Transform) and apply velocities.
/// This set runs after velocity calculations to ensure all velocities are computed first.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovementSystemSet;

/// Main game plugin that coordinates all gameplay sub-plugins.
///
/// Registers sub-plugins for:
/// - Input handling (InputPlugin)
/// - Battlefield and castle setup (BattlefieldPlugin)
/// - All units: wizard, defenders, attackers (UnitsPlugin)
/// - Shared movement and cleanup systems
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalAttackCycle>()
            .init_resource::<KillStats>()
            .init_resource::<CurrentLevel>()
            .insert_resource(GameOutcome::Victory)
            .add_plugins((InputPlugin, BattlefieldPlugin, UnitsPlugin))
            .add_systems(
                OnEnter(AppState::InGame),
                shared_systems::init_level_from_config,
            )
            .add_systems(OnExit(AppState::InGame), shared_systems::cleanup_game)
            .add_systems(
                OnExit(InGameState::GameOver),
                (
                    shared_systems::cleanup_for_replay,
                    shared_systems::reset_resources_for_replay,
                ),
            )
            .configure_sets(
                Update,
                (
                    VelocitySystemSet.run_if(in_state(InGameState::Running)),
                    MovementSystemSet
                        .run_if(in_state(InGameState::Running))
                        .after(VelocitySystemSet),
                ),
            )
            .add_systems(
                Update,
                shared_systems::tick_attack_cycle.run_if(in_state(InGameState::Running)),
            )
            .add_systems(
                Update,
                (
                    // Separation adds flocking forces (immutable queries)
                    // Unit-specific targeting systems registered in their respective plugins
                    shared_systems::apply_separation,
                    shared_systems::apply_wall_avoidance,
                )
                    .chain()
                    .in_set(VelocitySystemSet),
            )
            .add_systems(
                Update,
                (
                    // Calculate effectiveness based on nearby allies/enemies
                    shared_systems::calculate_effectiveness,
                    // Apply rough terrain slowdown before movement
                    shared_systems::apply_rough_terrain_slowdown,
                )
                    .chain()
                    .run_if(in_state(InGameState::Running))
                    .after(VelocitySystemSet)
                    .before(MovementSystemSet),
            )
            .add_systems(
                Update,
                (
                    // Unit-specific movement systems run in parallel as a set
                    // (infantry_movement and archer_movement registered in their respective plugins)
                    // They read from TargetingVelocity set by update_targeting
                    shared_systems::enforce_wall_collision,
                    shared_systems::combat,
                    shared_systems::convert_dead_to_corpses,
                    // Update billboards to face camera
                    systems::update_billboards,
                    // Check win/lose conditions
                    win_lose_systems::check_win_lose_conditions,
                )
                    .chain()
                    .run_if(in_state(InGameState::Running))
                    .after(MovementSystemSet),
            );
    }
}
