use bevy::prelude::*;

use crate::state::{AppState, InGameState};

use super::run_conditions::is_gameplay_running;

use super::achievements::AchievementsPlugin;
use super::battlefield::BattlefieldPlugin;
use super::cauldron::CauldronPlugin;
use super::crt_effect::CrtEffectPlugin;
use super::constants::ATTACK_CYCLE_DURATION;
use super::drops::DropsPlugin;
use super::input::InputPlugin;
use super::loading::LoadingPlugin;
use super::messages::{
    AchievementUnlockedMessage, IngredientCollectedMessage, SpellResearchedMessage,
    WaveSpawnedMessage,
};
use super::pathfinding::PathfindingPlugin;
use super::units::wizard::talents::TalentsPlugin;
use super::resources::{
    BattleInsightData, CurrentLevel, GameOutcome, KillStats, RetryTracker, WaveState,
};
use super::shared_systems;
use super::systems;
use super::units::UnitsPlugin;
use super::units::boss::hags::systems as hags_systems;
use super::wave_systems;
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

/// System set that runs after combat resolution (wall collision → combat → corpse conversion).
/// Used by systems that need to react to combat results (e.g., brute AOE splash).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostCombatSet;

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
            .init_resource::<RetryTracker>()
            .init_resource::<BattleInsightData>()
            .init_resource::<WaveState>()
            .insert_resource(GameOutcome::Victory)
            .add_message::<AchievementUnlockedMessage>()
            .add_message::<IngredientCollectedMessage>()
            .add_message::<SpellResearchedMessage>()
            .add_message::<WaveSpawnedMessage>()
            .add_plugins((
                InputPlugin,
                LoadingPlugin,
                BattlefieldPlugin,
                UnitsPlugin,
                CauldronPlugin,
                PathfindingPlugin,
                AchievementsPlugin,
                DropsPlugin,
                CrtEffectPlugin,
                TalentsPlugin,
            ))
            .add_systems(
                OnEnter(AppState::MetaGame),
                shared_systems::init_level_from_config,
            )
            .add_systems(
                OnEnter(AppState::InGame),
                (
                    shared_systems::init_level_from_config,
                    shared_systems::reset_resources_for_replay,
                ),
            )
            .add_systems(OnExit(AppState::InGame), shared_systems::cleanup_game)
            // Also clean up OnGameplayScreen entities when leaving MP
            // (HUD, action bar, and other shared UI use this marker)
            .add_systems(
                OnExit(AppState::MultiplayerGame),
                shared_systems::cleanup_game,
            )
            .add_systems(
                OnExit(InGameState::ScoreScreen),
                (
                    shared_systems::cleanup_for_replay,
                    shared_systems::reset_resources_for_replay,
                ),
            )
            .configure_sets(
                Update,
                (
                    VelocitySystemSet.run_if(is_gameplay_running),
                    MovementSystemSet
                        .run_if(is_gameplay_running)
                        .after(VelocitySystemSet),
                    PostCombatSet
                        .run_if(is_gameplay_running)
                        .after(MovementSystemSet),
                ),
            )
            .add_systems(
                Update,
                (
                    shared_systems::tick_attack_cycle,
                    shared_systems::tick_elapsed_time,
                    wave_systems::tick_wave_timer,
                )
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                (
                    // Check if any defender is near an enemy to activate all defenders
                    shared_systems::activate_defenders_on_proximity,
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
                    // Zero out targeting when a wall blocks line to target
                    // Runs after all targeting systems (VelocitySystemSet) so every
                    // unit — including the King — has its targeting suppressed.
                    shared_systems::suppress_targeting_through_walls,
                    // Calculate effectiveness based on nearby allies/enemies
                    shared_systems::calculate_effectiveness,
                    // Apply rough terrain slowdown before movement
                    shared_systems::apply_rough_terrain_slowdown,
                )
                    .chain()
                    .run_if(is_gameplay_running)
                    .after(VelocitySystemSet)
                    .before(MovementSystemSet),
            )
            .add_systems(
                Update,
                (
                    shared_systems::enforce_wall_collision,
                    shared_systems::combat,
                    shared_systems::enforce_invulnerability,
                    hags_systems::resurrect_eyed_hags,
                    shared_systems::convert_dead_to_corpses,
                )
                    .chain()
                    .in_set(PostCombatSet),
            )
            // Billboard rotation is a visual-only system that must run for both
            // SP host AND MP guest (ghost entities need billboard facing too).
            // Runs during any active game state, not just gameplay simulation.
            .add_systems(
                Update,
                systems::update_billboards
                    .run_if(in_state(AppState::InGame).or(in_state(AppState::MultiplayerGame))),
            )
            // SP-only win/lose check — runs after combat chain, gated to InGameState
            // (MP has its own check_mp_king_death registered in MultiplayerGamePlugin)
            .add_systems(
                Update,
                win_lose_systems::check_win_lose_conditions
                    .after(PostCombatSet)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
