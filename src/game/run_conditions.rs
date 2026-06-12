use bevy::prelude::*;

use crate::config::{GameConfig, WizardType};
use crate::game::game_mode::components::{ActiveToggles, ToggleModifier};
use crate::networking::resources::PeerRole;
use crate::networking::session::MultiplayerSession;
use crate::state::{InGameState, MultiplayerGameState};

/// Check if any entities with the specified component exist.
/// Used to avoid running systems when there are no relevant entities.
///
/// Example: `any_exist::<MagicMissile>()` will only return true if there are magic missiles in the world.
///
/// This is more efficient than running systems with empty queries every frame.
pub fn any_exist<T: Component>() -> impl Fn(Query<(), With<T>>) -> bool {
    |query: Query<(), With<T>>| !query.is_empty()
}

/// Returns true if the single-player / co-op-host simulation should be active in
/// the given `InGameState`.
///
/// `Running` is always active. Past that, single-player and the co-op host follow
/// distinct rules (kept in separate branches so neither silently entangles the
/// other), evaluated by [`sp_menu_keeps_running`] and [`coop_host_keeps_running`].
/// The versus host runs in `MultiplayerGameState` and is handled by
/// `is_gameplay_running`'s separate `mp_state` branch — untouched here.
fn is_sp_simulation_active(
    state: &InGameState,
    active_toggles: &Option<Res<ActiveToggles>>,
    session: &Option<Res<MultiplayerSession>>,
) -> bool {
    if *state == InGameState::Running {
        return true;
    }
    let urgent = active_toggles
        .as_ref()
        .is_some_and(|t| t.is_active(ToggleModifier::Urgent));
    if session.as_ref().is_some_and(|s| s.is_coop()) {
        coop_host_keeps_running(state, urgent)
    } else {
        sp_menu_keeps_running(state, urgent)
    }
}

/// Single-player non-Running gate: only the spell/cauldron menus keep the sim
/// alive, and only with the Urgent toggle. A normal pause always freezes.
fn sp_menu_keeps_running(state: &InGameState, urgent: bool) -> bool {
    matches!(state, InGameState::SpellBook | InGameState::CauldronMenu) && urgent
}

/// Co-op-host non-Running gate: the spell/cauldron menus must NEVER pause the
/// shared match, so they always keep running; a pause freezes the host unless
/// Urgent is on (Urgent keeps the co-op game running through the pause).
fn coop_host_keeps_running(state: &InGameState, urgent: bool) -> bool {
    match state {
        InGameState::SpellBook | InGameState::CauldronMenu => true,
        InGameState::Paused => urgent,
        _ => false,
    }
}

/// Returns true when gameplay simulation should be running.
///
/// This is the primary run condition for all gameplay systems (movement, combat,
/// spells, etc.). It returns true in two scenarios:
/// - Single-player: `InGameState::Running` is active, OR urgent mode is enabled
///   and the player is in SpellBook/CauldronMenu
/// - Multiplayer host: `MultiplayerGameState::Running` is active AND this peer is the host
///
/// This allows all gameplay plugins to share a single set of system registrations
/// that work for both single-player and multiplayer host modes.
/// True for the set of multiplayer states where gameplay/spell systems run:
/// `Running`, plus the overlays that don't pause the simulation (`Paused` is a
/// co-op sync-pause, `Settings`/`SpellBook`/`CauldronMenu` are in-game overlays).
fn mp_state_in_gameplay_set(s: &MultiplayerGameState) -> bool {
    matches!(
        s,
        MultiplayerGameState::Running
            | MultiplayerGameState::Paused
            | MultiplayerGameState::Settings
            | MultiplayerGameState::SpellBook
            | MultiplayerGameState::CauldronMenu
    )
}

pub fn is_gameplay_running(
    sp_state: Option<Res<State<InGameState>>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    session: Option<Res<MultiplayerSession>>,
    active_toggles: Option<Res<ActiveToggles>>,
) -> bool {
    // Single-player / co-op host
    if let Some(ref state) = sp_state
        && is_sp_simulation_active(state.get(), &active_toggles, &session)
    {
        return true;
    }
    // Multiplayer host: Running, Paused, or SpellBook (overlays don't pause gameplay)
    if mp_state.is_some_and(|s| mp_state_in_gameplay_set(s.get())) {
        return session.is_some_and(|s| s.role == PeerRole::Host);
    }
    false
}

/// Returns true when a game is active (any in-game state, not just Running).
///
/// Equivalent to `in_state(AppState::InGame)` but also covers multiplayer host.
/// Used for systems that should run across all in-game sub-states (e.g. during
/// pause or spell book screens).
pub fn is_gameplay_active(
    sp_state: Option<Res<State<InGameState>>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    session: Option<Res<MultiplayerSession>>,
) -> bool {
    // Single-player: any InGameState exists
    if sp_state.is_some() {
        return true;
    }
    // Multiplayer host: any MultiplayerGameState exists + host role
    if mp_state.is_some() {
        return session.is_some_and(|s| s.role == PeerRole::Host);
    }
    false
}

/// Returns true when any local wizard should process input.
///
/// Similar to `is_gameplay_running`, but also returns true for the multiplayer
/// guest. This allows both host and guest to prime spells locally and process
/// local wizard input (e.g., spell priming from action bar).
pub fn is_local_wizard_active(
    sp_state: Option<Res<State<InGameState>>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
) -> bool {
    // Single-player: InGameState::Running
    if sp_state.is_some_and(|s| *s.get() == InGameState::Running) {
        return true;
    }
    // Multiplayer: Running, Paused, or SpellBook — both host AND guest
    if mp_state.is_some_and(|s| mp_state_in_gameplay_set(s.get())) {
        return true;
    }
    false
}

/// Returns true when spell visual/lifecycle systems should run.
///
/// Similar to `is_local_wizard_active`, this returns true for both host AND guest
/// in multiplayer. Used as the run condition for spell plugins so that visual,
/// lifecycle, and movement systems run on the guest (where the simulation systems
/// are safe no-ops because their queries find no entities with Health/Team).
pub fn is_spell_effects_active(
    sp_state: Option<Res<State<InGameState>>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    active_toggles: Option<Res<ActiveToggles>>,
    session: Option<Res<MultiplayerSession>>,
) -> bool {
    // Single-player / co-op host
    if let Some(ref state) = sp_state
        && is_sp_simulation_active(state.get(), &active_toggles, &session)
    {
        return true;
    }
    // Multiplayer: Running, Paused, or SpellBook — both host AND guest
    mp_state.is_some_and(|s| mp_state_in_gameplay_set(s.get()))
}

/// Duration (seconds) of the multiplayer setup stage at the start of a match,
/// during which armies are frozen and units are immune to damage.
pub const MP_SETUP_DURATION: f32 = 10.0;

/// Returns true OUTSIDE the multiplayer setup stage. Used to gate movement
/// system sets so they pause during the first `MP_SETUP_DURATION` seconds of a
/// match. Driven by the host-authoritative match clock (`KillStats::elapsed_time`),
/// which the guest mirrors via snapshots, so both peers agree. Always true in
/// single-player (no `MultiplayerSession`).
pub fn is_not_mp_setup_phase(
    kill_stats: Res<crate::game::resources::KillStats>,
    session: Option<Res<MultiplayerSession>>,
) -> bool {
    // Co-op runs on the single-player endless/roguelite battlefield (whose
    // attacker waves pace themselves), so it has NO versus-style setup freeze.
    // Only a versus session freezes the armies for the opening seconds.
    let Some(session) = session else {
        return true;
    };
    if session.is_coop() {
        return true;
    }
    kill_stats.elapsed_time >= MP_SETUP_DURATION
}

/// Returns true if the active wizard type is RuneCaster.
pub fn is_rune_caster(config: Res<GameConfig>) -> bool {
    config.wizard_type == WizardType::RuneCaster
}

/// Returns true if the active wizard type is Randomancer.
pub fn is_randomancer(config: Res<GameConfig>) -> bool {
    config.wizard_type == WizardType::Randomancer
}

/// Returns true if the active wizard type is Arcanorouter.
pub fn is_arcanorouter(config: Res<GameConfig>) -> bool {
    config.wizard_type == WizardType::Arcanorouter
}

/// Returns true if the active wizard type is Warglock (gunslinger).
pub fn is_warglock(config: Res<GameConfig>) -> bool {
    config.wizard_type == WizardType::Warglock
}

/// Returns true if the active wizard type is NOT Warglock.
pub fn is_not_warglock(config: Res<GameConfig>) -> bool {
    config.wizard_type != WizardType::Warglock
}

/// Returns true if the active wizard type is Swordcerer.
pub fn is_swordcerer(config: Res<GameConfig>) -> bool {
    config.wizard_type == WizardType::Swordcerer
}

/// Returns true when EITHER player is the Swordcerer — used host-side for the
/// avatar systems that must run regardless of which peer owns the avatar (e.g.
/// cooldown ticking).
pub fn is_swordcerer_participant(
    config: Res<GameConfig>,
    session: Option<Res<MultiplayerSession>>,
) -> bool {
    if config.wizard_type == WizardType::Swordcerer {
        return true;
    }
    session.is_some_and(|s| {
        s.host_wizard == WizardType::Swordcerer || s.guest_wizard == WizardType::Swordcerer
    })
}

/// Returns true if the active wizard type is Meteorologist.
pub fn is_meteorologist(config: Res<GameConfig>) -> bool {
    config.wizard_type == WizardType::Meteorologist
}
