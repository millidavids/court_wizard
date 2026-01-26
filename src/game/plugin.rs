use bevy::prelude::*;

use crate::state::{AppState, InGameState};

use super::battlefield::BattlefieldPlugin;
use super::constants::ATTACK_CYCLE_DURATION;
use super::input::InputPlugin;
use super::shared_systems;
use super::units::UnitsPlugin;

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
            .add_plugins((InputPlugin, BattlefieldPlugin, UnitsPlugin))
            .add_systems(OnExit(AppState::InGame), shared_systems::cleanup_game)
            .add_systems(
                Update,
                (
                    shared_systems::tick_attack_cycle,
                    // Separation runs after targeting but before movement
                    shared_systems::apply_separation,
                    // Apply rough terrain slowdown before movement
                    shared_systems::apply_rough_terrain_slowdown,
                    shared_systems::move_units,
                    shared_systems::combat,
                    shared_systems::convert_dead_to_corpses,
                )
                    .chain()
                    .run_if(in_state(InGameState::Running)),
            );
    }
}
