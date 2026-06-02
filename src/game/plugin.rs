use bevy::prelude::*;

use crate::state::{AppState, InGameState};

#[cfg(debug_assertions)]
use super::components::OnGameplayScreen;
use super::run_conditions::{is_gameplay_running, is_spell_effects_active};
#[cfg(debug_assertions)]
use super::units::components::Hitbox;

pub use super::sets::{MovementSystemSet, PostCombatSet, VelocitySystemSet};

use super::achievements::AchievementsPlugin;
use super::battlefield::BattlefieldPlugin;
use super::cauldron::CauldronPlugin;
use super::combat_systems;
use super::constants::ATTACK_CYCLE_DURATION;
use super::crt_effect::CrtEffectPlugin;
use super::debug_ui::DebugUiPlugin;
#[cfg(debug_assertions)]
use super::debug_ui::DebugUiVisible;
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

/// Global attack cycle timer resource.
///
/// Cycles from 0.0 to CYCLE_DURATION seconds. Units track which time offset
/// in the cycle they last attacked and can only attack again when the timer
/// cycles back to that offset. This naturally staggers attacks across all units.
#[derive(Resource)]
pub struct GlobalAttackCycle {
    /// Current time in the cycle (0.0 to CYCLE_DURATION)
    pub current_time: f32,
    /// Cycle time BEFORE the most recent `tick(delta)` call. `combat()`
    /// uses this as the `last_time` parameter to `can_attack` so the
    /// "did the cycle sweep past this unit's slot in the last frame?"
    /// window is exactly the time actually elapsed — not a constant
    /// `APPROX_FRAME_TIME` approximation that re-fires under fast frames
    /// (frame_delta < 0.016) and skips slots under slow frames
    /// (frame_delta > 0.016).
    pub previous_time: f32,
    /// Duration of one complete cycle in seconds
    pub cycle_duration: f32,
}

impl Default for GlobalAttackCycle {
    fn default() -> Self {
        Self {
            current_time: 0.0,
            previous_time: 0.0,
            cycle_duration: ATTACK_CYCLE_DURATION,
        }
    }
}

impl GlobalAttackCycle {
    /// Advances the cycle timer by delta time, wrapping back to 0 after cycle_duration.
    pub fn tick(&mut self, delta: f32) {
        self.previous_time = self.current_time;
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
                    apply_game_speed,
                ),
            )
            .add_systems(
                OnEnter(InGameState::ScoreScreen),
                shared_systems::stop_all_sfx,
            )
            .add_systems(
                Update,
                auto_pause_on_focus_loss
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
                    // Host-authoritative match clock. In MP the guest does NOT tick
                    // its own — it mirrors the host's value from the game snapshot
                    // (`apply_state_snapshot`), so both peers show the exact same time.
                    shared_systems::tick_elapsed_time,
                )
                    .run_if(is_gameplay_running),
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
                    movement_systems::enforce_wall_collision,
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
            .add_systems(
                Update,
                (
                    shared_systems::update_battle_ambience,
                    shared_systems::update_crowd_ambience,
                )
                    .run_if(is_gameplay_running),
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

        // Debug hitbox visualization — driven by the global DebugUiVisible
        // flag (toggled by F2 in `debug_ui.rs`). Debug builds only.
        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            (
                sync_debug_hitboxes_resource,
                update_debug_hitboxes.run_if(resource_exists::<DebugHitboxes>),
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

/// Sets the virtual time speed from the GameConfig game_speed setting.
fn apply_game_speed(config: Res<crate::config::GameConfig>, mut time: ResMut<Time<Virtual>>) {
    time.set_relative_speed_f64(config.game_speed as f64);
}

// --- Debug hitbox visualization ---

/// When present, debug hitbox cylinders are shown.
#[cfg(debug_assertions)]
#[derive(Resource)]
struct DebugHitboxes {
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
}

/// Marker linking a debug cylinder to its parent unit entity.
#[cfg(debug_assertions)]
#[derive(Component)]
struct DebugHitboxMarker(Entity);

#[cfg(debug_assertions)]
fn sync_debug_hitboxes_resource(
    mut commands: Commands,
    visible: Res<DebugUiVisible>,
    existing: Option<Res<DebugHitboxes>>,
    debug_cylinders: Query<Entity, With<DebugHitboxMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if visible.0 && existing.is_none() {
        let material = materials.add(StandardMaterial {
            base_color: Color::srgba(0.5, 0.5, 0.5, 0.3),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        });
        let mesh = meshes.add(Cylinder::new(1.0, 1.0));
        commands.insert_resource(DebugHitboxes { material, mesh });
    } else if !visible.0 && existing.is_some() {
        commands.remove_resource::<DebugHitboxes>();
        for entity in &debug_cylinders {
            commands.entity(entity).try_despawn();
        }
    }
}

#[cfg(debug_assertions)]
fn update_debug_hitboxes(
    mut commands: Commands,
    debug_res: Res<DebugHitboxes>,
    units: Query<(Entity, &Transform, &Hitbox)>,
    mut cylinders: Query<(&DebugHitboxMarker, &mut Transform), Without<Hitbox>>,
    cylinder_entities: Query<(Entity, &DebugHitboxMarker), Without<Hitbox>>,
) {
    // Track which units already have a debug cylinder
    let mut has_cylinder: std::collections::HashSet<Entity> = std::collections::HashSet::new();
    for (marker, mut cyl_transform) in &mut cylinders {
        if let Ok((_, unit_transform, hitbox)) = units.get(marker.0) {
            has_cylinder.insert(marker.0);
            // Position cylinder centered on the unit's position, scaled to hitbox.
            // Cylinder primitive has half_height=1 (total height=2), so Y scale = height/2.
            cyl_transform.translation = unit_transform.translation;
            cyl_transform.scale = Vec3::new(hitbox.radius, hitbox.height / 2.0, hitbox.radius);
        } else {
            // Unit despawned — will be cleaned up below
        }
    }

    // Remove orphaned debug cylinders (unit no longer exists)
    for (entity, marker) in &cylinder_entities {
        if units.get(marker.0).is_err() {
            commands.entity(entity).try_despawn();
        }
    }

    // Spawn debug cylinders for new units
    for (entity, _transform, _hitbox) in &units {
        if !has_cylinder.contains(&entity) {
            commands.spawn((
                Mesh3d(debug_res.mesh.clone()),
                MeshMaterial3d(debug_res.material.clone()),
                Transform::default(),
                DebugHitboxMarker(entity),
                OnGameplayScreen,
            ));
        }
    }
}

/// Pauses gameplay when the window loses focus.
fn auto_pause_on_focus_loss(
    mut focus_events: MessageReader<bevy::window::WindowFocused>,
    mut next_state: ResMut<NextState<InGameState>>,
) {
    for event in focus_events.read() {
        if !event.focused {
            next_state.set(InGameState::Paused);
        }
    }
}
