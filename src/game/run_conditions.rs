use bevy::prelude::*;

use crate::config::{GameConfig, WizardType};
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

/// Returns true when gameplay simulation should be running.
///
/// This is the primary run condition for all gameplay systems (movement, combat,
/// spells, etc.). It returns true in two scenarios:
/// - Single-player: `InGameState::Running` is active
/// - Multiplayer host: `MultiplayerGameState::Running` is active AND this peer is the host
///
/// This allows all gameplay plugins to share a single set of system registrations
/// that work for both single-player and multiplayer host modes.
pub fn is_gameplay_running(
    sp_state: Option<Res<State<InGameState>>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    session: Option<Res<MultiplayerSession>>,
) -> bool {
    // Single-player: InGameState::Running
    if sp_state.is_some_and(|s| *s.get() == InGameState::Running) {
        return true;
    }
    // Multiplayer host: Running or Paused (escape menu doesn't pause gameplay)
    if mp_state.is_some_and(|s| {
        matches!(
            *s.get(),
            MultiplayerGameState::Running | MultiplayerGameState::Paused
        )
    }) {
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
