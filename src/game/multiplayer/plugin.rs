//! Multiplayer game plugin.
//!
//! Registers all multiplayer gameplay systems. This is completely independent
//! from `GamePlugin` — it reuses shared helper functions but has its own
//! system registrations under `AppState::MultiplayerGame`.

use bevy::prelude::*;

use crate::game::cauldron::resources::CauldronBuffs;
use crate::game::plugin::GlobalAttackCycle;
use crate::game::resources::{GameOutcome, KillStats};
use crate::game::run_conditions::any_exist;
use crate::game::shared_systems;
use crate::game::units::archer::components::{Archer, Arrow};
use crate::game::units::archer::systems as archer_systems;
use crate::game::units::components::{
    BattleHymnModifier, BerserkerRageModifier, FogEvasionModifier, GreaseSlipModifier,
    MarkedForDeathModifier, MesmerizedModifier, SleepModifier,
};
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::units::infantry::systems as infantry_systems;
use crate::game::units::infantry::Infantry;
use crate::game::units::king::components::King;
use crate::game::units::king::systems as king_systems;
use crate::game::units::movement;
use crate::game::units::systems as unit_systems;
use crate::networking::entity_map::EntityIdCounter;
use crate::networking::entity_map::NetworkEntityMap;
use crate::networking::session::{is_multiplayer_guest, is_multiplayer_host};
use crate::networking::snapshot::SnapshotTick;
use crate::state::{AppState, MultiplayerGameState};

use super::components::OnMultiplayerGameScreen;
use super::guest_systems;
use super::host_systems;
use super::loading;

/// Multiplayer-specific system sets for ordering host simulation.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MpSystemSet {
    /// Velocity/targeting calculations (parallel, immutable queries).
    Velocity,
    /// Movement application (after velocity).
    Movement,
}

/// Plugin that manages multiplayer gameplay.
pub struct MultiplayerGamePlugin;

impl Plugin for MultiplayerGamePlugin {
    fn build(&self, app: &mut App) {
        // ── Multiplayer Loading ──────────────────────────────────────
        app.add_systems(
            OnEnter(AppState::MultiplayerLoading),
            loading::init_mp_loading,
        )
        .add_systems(
            Update,
            loading::process_mp_spawn_queue.run_if(in_state(AppState::MultiplayerLoading)),
        )
        .add_systems(
            OnExit(AppState::MultiplayerLoading),
            loading::cleanup_mp_loading,
        );

        // ── Camera ───────────────────────────────────────────────────
        app.add_systems(
            OnEnter(AppState::MultiplayerGame),
            loading::setup_mp_camera,
        );
        app.add_systems(
            OnExit(AppState::MultiplayerGame),
            loading::restore_camera,
        );

        // ── Resource Init / Cleanup ──────────────────────────────────
        app.add_systems(OnEnter(AppState::MultiplayerGame), init_mp_game);
        app.add_systems(OnExit(AppState::MultiplayerGame), cleanup_mp_game);

        // ── System Set Configuration ─────────────────────────────────
        let mp_host = in_state(MultiplayerGameState::Running).and(is_multiplayer_host);

        app.configure_sets(
            Update,
            (
                MpSystemSet::Velocity.run_if(mp_host.clone()),
                MpSystemSet::Movement
                    .run_if(mp_host.clone())
                    .after(MpSystemSet::Velocity),
            ),
        );

        // ── Host Simulation: Core Game Loop ──────────────────────────
        // Tick timers
        app.add_systems(
            Update,
            (
                shared_systems::tick_attack_cycle,
                shared_systems::tick_elapsed_time,
            )
                .run_if(mp_host.clone()),
        );

        // Velocity set: activation, separation, wall avoidance
        app.add_systems(
            Update,
            (
                shared_systems::activate_defenders_on_proximity,
                shared_systems::apply_separation,
                shared_systems::apply_wall_avoidance,
            )
                .chain()
                .in_set(MpSystemSet::Velocity),
        );

        // Between velocity and movement: effectiveness, terrain slowdown
        app.add_systems(
            Update,
            (
                shared_systems::calculate_effectiveness,
                shared_systems::apply_rough_terrain_slowdown,
            )
                .chain()
                .run_if(mp_host.clone())
                .after(MpSystemSet::Velocity)
                .before(MpSystemSet::Movement),
        );

        // After movement: wall collision, combat, corpse conversion, billboards
        app.add_systems(
            Update,
            (
                shared_systems::enforce_wall_collision,
                shared_systems::combat,
                shared_systems::convert_dead_to_corpses,
            )
                .chain()
                .run_if(mp_host.clone())
                .after(MpSystemSet::Movement),
        );

        // ── Host Simulation: Unit Systems ────────────────────────────
        // Modifier tickers + damage effects
        app.add_systems(
            Update,
            (
                unit_systems::process_pending_damage_effects,
                unit_systems::update_temporary_hit_points,
                unit_systems::update_frost_slow_modifiers,
                unit_systems::update_rooted_modifiers,
                unit_systems::update_haste_modifiers,
                unit_systems::update_spike_growth_slow_modifiers,
                unit_systems::update_fire_dot,
                unit_systems::update_electric_charge,
                unit_systems::update_electric_arc_visuals,
                unit_systems::update_persistent_effect_visuals,
            )
                .run_if(mp_host.clone()),
        );

        // Conditional modifier tickers (only when components exist)
        app.add_systems(
            Update,
            (
                unit_systems::update_mark_of_death_modifiers
                    .run_if(any_with_component::<MarkedForDeathModifier>),
                unit_systems::update_mesmerized_modifiers
                    .run_if(any_with_component::<MesmerizedModifier>),
                unit_systems::update_sleep_modifiers
                    .run_if(any_with_component::<SleepModifier>),
                unit_systems::update_battle_hymn_modifiers
                    .run_if(any_with_component::<BattleHymnModifier>),
                unit_systems::update_berserker_rage_modifiers
                    .run_if(any_with_component::<BerserkerRageModifier>),
                unit_systems::update_fog_evasion_modifiers
                    .run_if(any_with_component::<FogEvasionModifier>),
                unit_systems::update_grease_slip_modifiers
                    .run_if(any_with_component::<GreaseSlipModifier>),
            )
                .run_if(mp_host.clone()),
        );

        // Unit movement application (after movement calculations)
        app.add_systems(
            Update,
            movement::apply_unit_movement
                .run_if(mp_host.clone())
                .after(MpSystemSet::Movement),
        );

        // ── Host Simulation: Infantry ────────────────────────────────
        app.add_systems(
            Update,
            (
                infantry_systems::check_defender_activation
                    .before(MpSystemSet::Velocity),
                infantry_systems::update_infantry_targeting
                    .in_set(MpSystemSet::Velocity),
                infantry_systems::infantry_movement
                    .in_set(MpSystemSet::Movement),
            )
                .run_if(any_exist::<Infantry>())
                .run_if(mp_host.clone()),
        );

        // ── Host Simulation: Archer ──────────────────────────────────
        app.add_systems(
            Update,
            (
                archer_systems::update_archer_targeting
                    .in_set(MpSystemSet::Velocity),
                archer_systems::archer_movement
                    .in_set(MpSystemSet::Movement),
                (
                    archer_systems::update_archer_movement_timers,
                    archer_systems::archer_melee_combat,
                    archer_systems::archer_ranged_combat,
                )
                    .chain(),
            )
                .run_if(any_exist::<Archer>())
                .run_if(mp_host.clone()),
        );

        app.add_systems(
            Update,
            (
                archer_systems::move_arrows,
                archer_systems::check_arrow_collisions,
            )
                .chain()
                .run_if(any_exist::<Arrow>())
                .run_if(mp_host.clone()),
        );

        // ── Host Simulation: King ────────────────────────────────────
        app.add_systems(
            Update,
            (
                king_systems::update_king_targeting
                    .in_set(MpSystemSet::Velocity),
                king_systems::king_movement
                    .in_set(MpSystemSet::Movement),
                king_systems::king_cohesion_force
                    .after(MpSystemSet::Velocity)
                    .before(MpSystemSet::Movement),
                king_systems::snap_kings_guard_to_king
                    .in_set(MpSystemSet::Movement),
            )
                .run_if(any_exist::<King>())
                .run_if(mp_host.clone()),
        );

        // ── Host Networking: ID Assignment + Snapshots ───────────────
        app.add_systems(
            Update,
            (
                host_systems::assign_network_ids,
                host_systems::send_state_snapshots,
            )
                .chain()
                .run_if(mp_host.clone()),
        );

        // ── Billboards (both host and guest) ─────────────────────────
        app.add_systems(
            Update,
            crate::game::systems::update_billboards
                .run_if(in_state(MultiplayerGameState::Running)),
        );

        // ── Guest: Snapshot Rendering ────────────────────────────────
        app.add_systems(
            Update,
            guest_systems::apply_state_snapshot
                .run_if(in_state(MultiplayerGameState::Running).and(is_multiplayer_guest)),
        );
    }
}

/// Initializes resources needed for multiplayer gameplay.
fn init_mp_game(mut commands: Commands) {
    commands.init_resource::<GlobalAttackCycle>();
    commands.init_resource::<KillStats>();
    commands.insert_resource(DefendersActivated { active: true });
    commands.init_resource::<EntityIdCounter>();
    commands.init_resource::<NetworkEntityMap>();
    commands.init_resource::<SnapshotTick>();
    commands.init_resource::<CauldronBuffs>();
    commands.insert_resource(GameOutcome::Victory);
}

/// Cleans up multiplayer game entities and resources.
fn cleanup_mp_game(
    mut commands: Commands,
    mp_entities: Query<Entity, With<OnMultiplayerGameScreen>>,
) {
    for entity in &mp_entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
    commands.remove_resource::<GlobalAttackCycle>();
    commands.remove_resource::<KillStats>();
    commands.remove_resource::<DefendersActivated>();
    commands.remove_resource::<EntityIdCounter>();
    commands.remove_resource::<NetworkEntityMap>();
    commands.remove_resource::<SnapshotTick>();
    commands.remove_resource::<CauldronBuffs>();
    commands.remove_resource::<GameOutcome>();
}
