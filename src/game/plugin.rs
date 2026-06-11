use bevy::prelude::*;

use crate::state::{AppState, InGameState};

use super::run_conditions::{is_gameplay_running, is_not_mp_setup_phase, is_spell_effects_active};

pub use super::sets::{MovementSystemSet, PostCombatSet, VelocitySystemSet};

use super::achievements::AchievementsPlugin;
use super::attack_cycle::GlobalAttackCycle;
use super::battlefield::BattlefieldPlugin;
use super::cauldron::CauldronPlugin;
use super::combat_systems;
use super::crt_effect::CrtEffectPlugin;
use super::debug_ui::DebugUiPlugin;
use super::drops::DropsPlugin;
use super::game_mode::GameModePlugin;
use super::input::InputPlugin;
use super::loading::LoadingPlugin;
use super::messages::{
    AchievementUnlockedMessage, IngredientCollectedMessage, InsightBonusUpgradedMessage,
    RetreatMessage, SpellResearchedMessage, WaveSpawnedMessage,
};
use super::movement_systems;
use super::pathfinding::PathfindingPlugin;
use super::resources::{
    BattleInsightData, CurrentLevel, GameOutcome, KillStats, RetryTracker, WaveState,
};
use super::seeded_rng::SeededRngPlugin;
use super::shared_systems::{self, ShadowMaterial};
use super::systems;
use super::units::UnitsPlugin;
use super::units::boss::hags::systems as hags_systems;
use super::units::wizard::talents::TalentsPlugin;
use super::wave_systems;
use super::win_lose_systems;

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
        #[cfg(feature = "benchmarking")]
        app.add_plugins(super::benchmarking::BenchmarkingPlugin);

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
            .add_message::<InsightBonusUpgradedMessage>()
            .add_message::<WaveSpawnedMessage>()
            .add_message::<RetreatMessage>()
            .add_plugins((
                InputPlugin,
                LoadingPlugin,
                BattlefieldPlugin,
                UnitsPlugin,
                CauldronPlugin,
                PathfindingPlugin,
                AchievementsPlugin,
                DropsPlugin,
                GameModePlugin,
                CrtEffectPlugin,
                DebugUiPlugin,
                TalentsPlugin,
                SeededRngPlugin,
                MaterialPlugin::<ShadowMaterial>::default(),
            ))
            .add_systems(
                Startup,
                (
                    shared_systems::load_battle_ambience_assets,
                    shared_systems::preload_shadow_assets,
                ),
            )
            .add_systems(
                OnEnter(AppState::MetaGame),
                shared_systems::init_level_from_config,
            )
            .add_systems(
                OnEnter(AppState::InGame),
                (
                    shared_systems::init_level_from_config,
                    shared_systems::reset_resources_for_replay,
                    shared_systems::apply_game_speed,
                ),
            )
            .add_systems(
                OnEnter(InGameState::ScoreScreen),
                shared_systems::stop_all_sfx,
            )
            .add_systems(
                Update,
                shared_systems::auto_pause_on_focus_loss
                    .run_if(in_state(InGameState::Running))
                    .run_if(|config: Res<crate::config::GameConfig>| {
                        config.auto_pause_on_focus_loss
                    }),
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
                    VelocitySystemSet
                        .run_if(is_gameplay_running)
                        .run_if(is_not_mp_setup_phase),
                    MovementSystemSet
                        .run_if(is_gameplay_running)
                        .run_if(is_not_mp_setup_phase)
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
                    // Host-authoritative match clock. In MP the guest does NOT tick
                    // its own — it mirrors the host's value from the game snapshot
                    // (`apply_state_snapshot`), so both peers show the exact same time.
                    shared_systems::tick_elapsed_time,
                )
                    .run_if(is_gameplay_running),
            )
            // Drives the global setup-stage damage-immunity flag. Runs on BOTH
            // peers (guest mirrors the host's clock) so guest-cast spells also
            // deal no damage during setup. Gated on the session resource so it
            // is a no-op in single-player.
            .add_systems(
                Update,
                crate::game::units::components::sync_setup_immunity_flag
                    .run_if(resource_exists::<crate::networking::session::MultiplayerSession>)
                    // Write the immunity flag before any damage is resolved this
                    // frame, so there's no first-frame window where the flag is
                    // still false while combat runs.
                    .before(PostCombatSet),
            )
            // Wave staging is single-player only. Multiplayer's attacker
            // armies are spawned in full at match start (`init_mp_loading`);
            // there is no off-screen wave to stage in.
            .add_systems(
                Update,
                wave_systems::tick_wave_timer
                    .run_if(is_gameplay_running.and(in_state(AppState::InGame))),
            )
            .add_systems(
                Update,
                wave_systems::apply_wave_upgrades
                    .run_if(resource_exists::<super::resources::PendingWaveUpgrades>),
            )
            .add_systems(
                Update,
                (
                    // Check if any defender is near an enemy to activate all defenders
                    shared_systems::activate_defenders_on_proximity,
                    // Separation adds flocking forces (immutable queries)
                    // Unit-specific targeting systems registered in their respective plugins
                    movement_systems::apply_separation,
                    movement_systems::apply_wall_avoidance,
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
                    movement_systems::suppress_targeting_through_walls,
                    // Staging attackers must not target enemies — only follow staging flow field
                    crate::game::pathfinding::systems::suppress_staging_targeting,
                    // Calculate effectiveness based on nearby allies/enemies
                    shared_systems::calculate_effectiveness,
                    // Apply rough terrain slowdown before movement
                    movement_systems::apply_rough_terrain_slowdown,
                )
                    .chain()
                    .run_if(is_gameplay_running)
                    .after(VelocitySystemSet)
                    .before(MovementSystemSet),
            )
            .add_systems(
                Update,
                (
                    // Frozen during the MP setup stage so units aren't shoved out
                    // of player-built walls while the armies are meant to hold still.
                    // (The rest of PostCombatSet keeps running — snapshots, combat
                    // whose damage is already neutralized by the immunity flag, etc.)
                    movement_systems::enforce_wall_collision.run_if(is_not_mp_setup_phase),
                    combat_systems::combat,
                    combat_systems::enforce_invulnerability,
                    super::units::wizard::spells::berserker_rage::systems::undying_fury_trigger,
                    hags_systems::resurrect_eyed_hags,
                    combat_systems::convert_dead_to_corpses,
                )
                    .chain()
                    .in_set(PostCombatSet),
            )
            // Track wizard spell damage to enemies (for Pacifist achievement).
            // Runs after PostCombatSet, gated to stop once flagged.
            .add_systems(
                Update,
                shared_systems::track_wizard_enemy_damage
                    .after(PostCombatSet)
                    .run_if(shared_systems::wizard_has_not_damaged_enemies),
            )
            // Unit shadows — spawn and sync ground-level shadows under all
            // units. Runs on both MP peers so ghost units cast shadows. The
            // queries only need Team/Hitbox/Transform, all of which exist on
            // ghost units. No gameplay state is mutated. `OnGameplayScreen`
            // tagging on the shadow child is correct for MP too — both SP
            // and MP cleanup paths despawn `OnGameplayScreen` entities.
            .add_systems(
                Update,
                (
                    shared_systems::spawn_unit_shadows,
                    shared_systems::update_unit_shadows,
                )
                    .chain()
                    .run_if(is_spell_effects_active),
            )
            // Battle ambience — scales looping sword-clash sound with melee unit count
            // Crowd ambience — muffled crowd loop throughout battle
            // `is_spell_effects_active` (not `is_gameplay_running`) so the MP GUEST
            // also hears them: the host streams `InMelee` onto ghosts (via the
            // IN_MELEE snapshot flag) and unit counts come from the ghosts.
            .add_systems(
                Update,
                (
                    shared_systems::update_battle_ambience,
                    shared_systems::update_crowd_ambience,
                )
                    .run_if(is_spell_effects_active),
            )
            // Billboard rotation is a visual-only system that must run for both
            // SP host AND MP guest (ghost entities need billboard facing too).
            // Runs during any active game state, not just gameplay simulation.
            .add_systems(
                Update,
                systems::update_billboards
                    .run_if(in_state(AppState::InGame).or(in_state(AppState::MultiplayerGame))),
            )
            // SP-only win/lose check — runs after combat chain, gated to gameplay states
            // only. Must NOT run during InGameState::ScoreScreen: bevy_state 0.18's
            // NextState::set re-fires OnEnter on identity transitions, so re-detecting
            // the same victory/defeat each frame would rebuild the score screen and
            // re-accumulate kills every frame.
            // MP has its own `check_mp_king_death` registered in MultiplayerGamePlugin.
            // The `in_state(AppState::InGame)` guard is required: `is_gameplay_running`
            // returns true for the multiplayer host, which would let this SP system
            // race with `check_mp_king_death` and overwrite the host's
            // `GameOutcome::Victory` with `DefeatKingDied` (the SP system sees ANY
            // dead king without filtering by team), causing both peers to see Defeat.
            .add_systems(
                Update,
                win_lose_systems::check_win_lose_conditions
                    .after(PostCombatSet)
                    .run_if(is_gameplay_running)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
