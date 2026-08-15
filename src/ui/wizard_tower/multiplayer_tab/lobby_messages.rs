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

use super::state::{CoopHostSelection, LobbyPhase, MultiplayerLobby, load_my_unlocked_content};
use crate::ui::wizard_tower::layout::WizardTowerTab;

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
#[allow(clippy::too_many_arguments)]
pub(crate) fn process_lobby_messages(
    mut connection: ResMut<NetworkConnection>,
    mut lobby: ResMut<MultiplayerLobby>,
    mut commands: Commands,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut tab: Option<ResMut<WizardTowerTab>>,
    // Only used to tear the session down on a version mismatch — see below.
    mut host_selection: ResMut<CoopHostSelection>,
    transport: Option<Res<crate::networking::transport::TransportHandle>>,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    mut steam_lobby: Option<ResMut<crate::steam::multiplayer::SteamLobbyState>>,
    mut steam_socket: Option<ResMut<crate::steam::multiplayer::SteamP2pSocket>>,
    session: Option<Res<MultiplayerSession>>,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    // Set by either version-mismatch branch. The teardown happens after the drain
    // loop so both branches share one call site.
    let mut version_mismatch: Option<String> = None;

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
                        version_mismatch = Some(format!(
                            "Multiplayer version mismatch: you are on v{local_version}, opponent is on v{version}. Both players must update to the same game version."
                        ));
                        break;
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
                        version_mismatch = Some(format!(
                            "Multiplayer version mismatch: opponent is on an older version that doesn't support the v{local_version} handshake. Both players must update to the same game version."
                        ));
                        break;
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
                    // The guest does everything from the Multiplayer tab — pull it
                    // there if the player was browsing another tab when the
                    // connection completed (the other tabs lock for a connected
                    // guest, but force the view so they land on the lobby screen).
                    if connection.role == Some(PeerRole::Guest)
                        && let Some(tab) = tab.as_deref_mut()
                    {
                        *tab = WizardTowerTab::Multiplayer;
                    }
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
                    urgent,
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
                            // Co-op roguelite Urgent disables pause-sync; the guest
                            // has no `ActiveToggles`, so it learns the flag here.
                            coop_urgent: urgent && mode == SessionMode::CoopRoguelite,
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
                NetworkMessage::HostModeSelection {
                    mode,
                    host_wizard,
                    level,
                    is_continue,
                    detail_lines,
                } => {
                    // Guest-only: mirror the host's selected mode into the
                    // resource that drives the guest's Multiplayer-tab left panel.
                    if connection.role == Some(PeerRole::Guest) {
                        commands.insert_resource(CoopHostSelection {
                            mode,
                            host_wizard: Some(host_wizard),
                            level,
                            is_continue,
                            detail_lines,
                        });
                    }
                }
                other => unhandled.push(other),
            }
        }

        if !unhandled.is_empty() {
            connection.incoming_messages.extend(unhandled);
        }
    }

    // A version mismatch used to set `LobbyPhase::Failed` and return, leaving the
    // connection `Connected`, the socket live and the Steam lobby joined behind a
    // terminal panel that `budget_for(PhaseKind::Failed, _)` never times out. The
    // player's only escape was Try Again / Back; anything else carried the whole dead
    // session forward. Tear it down properly, then paint the reason — the reset clears
    // `lobby.phase`, so the order matters.
    if let Some(reason) = version_mismatch {
        crate::game::multiplayer::session_reset::reset_multiplayer_to_baseline(
            "multiplayer version mismatch",
            &mut commands,
            &mut connection,
            &mut lobby,
            &mut host_selection,
            transport.as_deref(),
            steam_client.as_deref(),
            steam_lobby.as_deref_mut(),
            steam_socket.as_deref_mut(),
            session.is_some(),
        );
        lobby.phase = LobbyPhase::Failed { reason };
    }
}

/// Host-only: if both wizards are selected and the GUEST is ready, build the
/// `MultiplayerSession`, send `StartGame`, and transition to `MultiplayerLoading`.
/// Called by the host's explicit "Start Game" button (VS tab). The host no longer
/// readies — only the guest does — so this gates on `opponent_ready` alone.
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
        opponent_ready: true,
        ..
    } = &lobby.phase
        && connection.role == Some(PeerRole::Host)
    {
        info!(
            "[MP Lobby] Guest ready! Host: {:?}, Guest: {:?} — sending StartGame",
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
            coop_urgent: false,
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
                urgent: false,
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
