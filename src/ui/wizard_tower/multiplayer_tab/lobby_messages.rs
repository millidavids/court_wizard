//! Lobby message pump: drains `NetworkConnection.incoming_messages`,
//! advances `LobbyPhase`, and triggers `StartGame` when both peers are ready.
//!
//! Ported from the legacy multiplayer screen — the host-authoritative game-start
//! handshake is preserved verbatim.

use bevy::prelude::*;

use crate::config::WizardType;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{NetworkConnection, PeerRole};
use crate::networking::session::{MultiplayerSession, SessionMode};
use crate::state::AppState;

use super::state::{LobbyPhase, MultiplayerLobby, load_my_unlocked_content};

/// Coerces a wizard type to one allowed in multiplayer. Psychopath is disabled
/// in MP (its self-sabotage win condition doesn't map to a competitive match);
/// it's already filtered from the lobby grid, but if a peer still proposes it
/// (an old or modified client) fall back to Boring Ole Mage rather than entering
/// a match in an undefined state.
fn mp_safe_wizard(wt: WizardType) -> WizardType {
    if wt == WizardType::Psychopath {
        WizardType::BoringOleMage
    } else {
        wt
    }
}

/// Drains `NetworkConnection.incoming_messages` and handles
/// `PlayerInfo`/`WizardSelected`/`ReadyUp`/`Unready`/`StartGame`.
///
/// `PlayerInfo` is *sent* once by `sync_lobby_with_connection` at the moment the
/// lobby reaches `Handshake` — not here — so it is never re-sent per frame.
pub(crate) fn process_lobby_messages(
    mut connection: ResMut<NetworkConnection>,
    mut lobby: ResMut<MultiplayerLobby>,
    mut commands: Commands,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    {
        let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
        let mut unhandled = Vec::new();

        for msg in messages {
            match msg {
                NetworkMessage::HandshakeVersion { version } => {
                    let local_version = crate::networking::protocol::PROTOCOL_VERSION;
                    if version != local_version {
                        warn!(
                            "[MP Lobby] Version mismatch via HandshakeVersion: local v{local_version}, peer v{version}"
                        );
                        lobby.phase = LobbyPhase::Failed {
                            reason: format!(
                                "Multiplayer version mismatch: you are on v{local_version}, opponent is on v{version}. Both players must update to the same game version."
                            ),
                        };
                        return;
                    }
                    lobby.peer_protocol_version = Some(version);
                    info!("[MP Lobby] Received HandshakeVersion (v{version}) — match");
                }
                NetworkMessage::PlayerInfo {
                    wizard_types: opponent_wt,
                    spells: _,
                } => {
                    // Enforce the "HandshakeVersion-first" invariant. An old
                    // binary that doesn't know about `HandshakeVersion`
                    // sends `PlayerInfo` first with no preceding version
                    // exchange — we treat that as an incompatible peer.
                    if lobby.peer_protocol_version.is_none() {
                        let local_version = crate::networking::protocol::PROTOCOL_VERSION;
                        warn!(
                            "[MP Lobby] PlayerInfo arrived before HandshakeVersion — peer is on an incompatible older version"
                        );
                        lobby.phase = LobbyPhase::Failed {
                            reason: format!(
                                "Multiplayer version mismatch: opponent is on an older version that doesn't support the v{local_version} handshake. Both players must update to the same game version."
                            ),
                        };
                        return;
                    }
                    info!(
                        "[MP Lobby] Received PlayerInfo ({} wizard types, peer protocol v{:?})",
                        opponent_wt.len(),
                        lobby.peer_protocol_version
                    );
                    let (my_wt, _) = load_my_unlocked_content();
                    let initial = my_wt
                        .first()
                        .copied()
                        .unwrap_or(crate::config::WizardType::BoringOleMage);

                    lobby.phase = LobbyPhase::WizardSelect {
                        my_wizard_types: my_wt.clone(),
                        opponent_wizard_types: opponent_wt,
                        my_wizard: Some(initial),
                        opponent_wizard: None,
                        my_ready: false,
                        opponent_ready: false,
                    };
                    // Seed the shared wizard-card grid's selection so it
                    // highlights the correct card if the player opens it.
                    commands.insert_resource(
                        crate::ui::wizard_tower::wizard_cards::SelectedWizard(initial),
                    );
                    connection
                        .outgoing_messages
                        .push(NetworkMessage::WizardSelected(initial));
                }
                NetworkMessage::WizardSelected(wt) => {
                    if let LobbyPhase::WizardSelect {
                        opponent_wizard, ..
                    } = &mut lobby.phase
                    {
                        *opponent_wizard = Some(wt);
                    }
                }
                NetworkMessage::ReadyUp => {
                    if let LobbyPhase::WizardSelect { opponent_ready, .. } = &mut lobby.phase {
                        *opponent_ready = true;
                    }
                }
                NetworkMessage::Unready => {
                    if let LobbyPhase::WizardSelect { opponent_ready, .. } = &mut lobby.phase {
                        *opponent_ready = false;
                    }
                }
                NetworkMessage::StartGame {
                    seed,
                    coop_mode,
                    level,
                } => {
                    let mode = SessionMode::from_coop_wire(coop_mode);
                    info!("[MP Lobby] Received StartGame from host (seed {seed}, mode {mode:?})");
                    // Only start if we have a valid wizard-select state — the
                    // session and RNG must be in place before the transition,
                    // or `init_mp_loading` panics on the missing resource.
                    if let LobbyPhase::WizardSelect {
                        my_wizard: Some(my_wiz),
                        opponent_wizard: Some(opp_wiz),
                        ..
                    } = &lobby.phase
                    {
                        let (_, my_spells) = load_my_unlocked_content();
                        let session = MultiplayerSession {
                            role: PeerRole::Guest,
                            mode,
                            host_wizard: mp_safe_wizard(*opp_wiz),
                            guest_wizard: mp_safe_wizard(*my_wiz),
                            host_spells: Vec::new(),
                            guest_spells: my_spells,
                        };
                        commands.insert_resource(session);
                        // Seed the shared RNG from the host's seed so both
                        // peers produce identical randomness.
                        insert_game_rng(&mut commands, seed);
                        // Co-op: remember the host's level so `init_mp_loading`
                        // generates matching seeded terrain.
                        if mode != SessionMode::Versus {
                            commands.insert_resource(
                                crate::game::multiplayer::coop::CoopGuestLevel(level),
                            );
                        }
                        next_app_state.set(AppState::MultiplayerLoading);
                    } else {
                        warn!("[MP Lobby] Ignoring StartGame received outside wizard select");
                    }
                }
                other => unhandled.push(other),
            }
        }

        if !unhandled.is_empty() {
            connection.incoming_messages.extend(unhandled);
        }
    }
}

/// Host-only: if both wizards are selected and both players are ready, build
/// the `MultiplayerSession`, send `StartGame`, and transition to
/// `MultiplayerLoading`. Called by the host's explicit "Start Game" button.
/// No-ops if preconditions aren't met or the local peer isn't the host.
pub(super) fn commit_host_start(
    lobby: &MultiplayerLobby,
    connection: &mut NetworkConnection,
    commands: &mut Commands,
    next_app_state: &mut NextState<AppState>,
) {
    if let LobbyPhase::WizardSelect {
        my_wizard: Some(my_wiz),
        opponent_wizard: Some(opp_wiz),
        my_ready: true,
        opponent_ready: true,
        ..
    } = &lobby.phase
        && connection.role == Some(PeerRole::Host)
    {
        info!(
            "[MP Lobby] Both ready! Host: {:?}, Guest: {:?} — sending StartGame",
            my_wiz, opp_wiz
        );
        let (_, my_spells) = load_my_unlocked_content();
        // This button is the VERSUS start path only. Co-op matches are started
        // from the Endless/Roguelite tabs with a connection active (WS2), which
        // build a co-op `MultiplayerSession` on the single-player Loading path.
        let session = MultiplayerSession {
            role: PeerRole::Host,
            mode: SessionMode::Versus,
            host_wizard: mp_safe_wizard(*my_wiz),
            guest_wizard: mp_safe_wizard(*opp_wiz),
            host_spells: my_spells,
            guest_spells: Vec::new(),
        };
        commands.insert_resource(session);

        // Pick the shared run seed, seed our own RNG, and send it to the guest.
        let seed = rand::random::<u64>();
        insert_game_rng(commands, seed);
        connection
            .outgoing_messages
            .push(NetworkMessage::StartGame {
                seed,
                coop_mode: SessionMode::Versus.to_coop_wire(),
                level: 0,
            });
        next_app_state.set(AppState::MultiplayerLoading);
    }
}

/// Inserts the `GameSeed` and `GameRng` resources. Multiplayer doesn't go
/// through `AppState::Loading` (where single-player seeds its RNG), so the
/// lobby seeds it here from the host-chosen seed shared over the network.
fn insert_game_rng(commands: &mut Commands, seed: u64) {
    use crate::game::seeded_rng::resources::{GameRng, GameSeed};
    let game_seed = GameSeed(seed);
    commands.insert_resource(GameRng::new(&game_seed));
    commands.insert_resource(game_seed);
}
