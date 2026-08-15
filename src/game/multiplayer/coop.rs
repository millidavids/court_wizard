//! Co-op multiplayer: a guest fights alongside the host on the single-player
//! endless/roguelite battlefield.
//!
//! Unlike versus (both peers in `AppState::MultiplayerGame`), the co-op HOST
//! runs the authoritative simulation natively in `AppState::InGame` and the
//! guest spectates + casts from `AppState::MultiplayerGame`. The host therefore
//! never runs `init_mp_game` (which is `OnEnter(MultiplayerGame)`), so this
//! module stands up the same networking resources for the InGame host and tears
//! them down on exit — the mirror of `systems::init_mp_game` / `cleanup_mp_game`
//! for the host's single-player state path.

use bevy::prelude::*;

use crate::config::WizardType;
use crate::game::units::components::{Corpse, Effectiveness, Team};
use crate::game::units::wizard::components::Spell;
use crate::networking::crdt::PeerId;
use crate::networking::entity_map::{EntityIdCounter, NetworkEntityMap};
use crate::networking::resources::PeerRole;
use crate::networking::session::{MultiplayerSession, SessionMode};
use crate::networking::snapshot::SnapshotTick;

use super::spell_sync::PendingCastEvents;

/// Set by the wizard-tower start flow when the host launches an endless/roguelite
/// game with a guest connected. Consumed on the single-player loading path
/// (`init_loading_progress`), which turns it into a co-op `MultiplayerSession`
/// for the host before the battlefield is built.
#[derive(Resource)]
pub struct CoopPendingSession {
    pub guest_wizard: WizardType,
    pub guest_spells: Vec<Spell>,
    pub mode: SessionMode,
}

/// True while a co-op guest is connected during the host's InGame match. Drives
/// the +30% attacker-effectiveness buff (read by `apply_coop_difficulty_buff`)
/// and is cleared on guest disconnect (`detect_coop_guest_disconnect`) so the
/// buff lifts immediately. Only present during a co-op host match.
#[derive(Resource, Default)]
pub struct CoopGuestConnected(pub bool);

/// The host's endless/roguelite level, received by the guest in `StartGame` and
/// used by `init_mp_loading` so the guest generates the SAME seeded terrain the
/// host's single-player loader produced. Guest-side only.
#[derive(Resource)]
pub struct CoopGuestLevel(pub u32);

/// The connected co-op partner's display name (Steam persona name when on Steam,
/// else `None` → a generic "MP connected"). Populated while a connection is live
/// and read by the wizard-tower header indicator and the save-tagging path.
#[derive(Resource, Default)]
pub struct CoopPeerInfo {
    pub name: Option<String>,
}

/// Host-side "both peers loaded" handshake for a co-op level. The co-op host
/// loads on the single-player `AppState::Loading` path (which normally does no
/// networking), so it must exchange `GameLoaded` with the guest (who loads in
/// `MultiplayerLoading`) before entering the match. Present only while the co-op
/// host is loading.
#[derive(Resource, Default)]
pub struct CoopLoadingSync {
    pub my_loaded: bool,
    pub peer_loaded: bool,
}

/// Returns true when this peer is the co-op HOST (co-op session, host role).
pub fn is_coop_host(session: Option<Res<MultiplayerSession>>) -> bool {
    session.is_some_and(|s| s.is_coop() && s.role == PeerRole::Host)
}

/// Sets up a co-op match start on the HOST (called from the Endless/Roguelite
/// tabs when a guest is connected): picks the shared seed into `config.seed`
/// (deterministic terrain), inserts `CoopPendingSession`, and sends `StartGame`
/// to pull the guest in. The caller still transitions to `AppState::Loading`.
pub fn start_coop_host(
    commands: &mut Commands,
    connection: &mut crate::networking::resources::NetworkConnection,
    config: &mut crate::config::GameConfig,
    guest_wizard: WizardType,
    mode: SessionMode,
    urgent: bool,
) {
    use crate::networking::protocol::NetworkMessage;
    let seed = rand::random::<u64>();
    config.seed = Some(seed);
    commands.insert_resource(CoopPendingSession {
        guest_wizard,
        guest_spells: Vec::new(),
        mode,
    });
    connection
        .outgoing_messages
        .push(NetworkMessage::StartGame {
            seed,
            coop_mode: mode.to_coop_wire(),
            level: config.current_level,
            urgent,
        });
}

/// Inserts the networking resources the InGame co-op host needs to snapshot the
/// battlefield to the guest. Mirrors the resource set `init_mp_game` inserts for
/// the guest (`systems.rs`) — keep the two in sync.
fn insert_host_networking_resources(commands: &mut Commands) {
    commands.insert_resource(PeerId(PeerId::HOST));
    commands.init_resource::<EntityIdCounter>();
    commands.init_resource::<NetworkEntityMap>();
    commands.init_resource::<SnapshotTick>();
    commands.init_resource::<crate::networking::snapshot::SpellSnapshotData>();
    commands.init_resource::<super::components::SpellEffectEntityMap>();
    commands.init_resource::<super::spell_sync::LatestSpellSnapshot>();
    commands.init_resource::<super::score_stats::LocalWizardStats>();
}

/// `OnEnter(AppState::InGame)`: for a co-op host match, stand up the networking
/// resources and activate cast-event replication so the host's spells ship to
/// the guest. No-op for single-player and for the guest (different state path).
///
/// Runs every level of a multi-level co-op run. The networking resources are torn
/// down by `cleanup_coop_host` on each `OnExit(InGame)` and re-created here from
/// `Default` on the next level, so the `EntityIdCounter` restarts at 0 every level.
/// That's safe because the guest also fully resets its ghosts + `NetworkEntityMap`
/// between levels (`cleanup_mp_game` on `OnExit(MultiplayerGame)`), so a restarted
/// counter can't alias a stale ghost ID. `CoopGuestConnected` is set from the live
/// connection so the +30% buff reflects whether the guest is still present after a
/// between-level reconnect/disconnect.
pub(in crate::game) fn init_coop_host(
    mut commands: Commands,
    session: Option<Res<MultiplayerSession>>,
    connection: Res<crate::networking::resources::NetworkConnection>,
    mut pending_events: ResMut<PendingCastEvents>,
) {
    let Some(session) = session else {
        return;
    };
    if !(session.is_coop() && session.role == PeerRole::Host) {
        return;
    }
    insert_host_networking_resources(&mut commands);
    let connected = connection.state == crate::networking::resources::ConnectionState::Connected;
    commands.insert_resource(CoopGuestConnected(connected));
    // `mark_pending_events_mp_active` only fires on `OnEnter(MultiplayerGame)`,
    // which the co-op host never enters — activate the cast-event drain here so
    // the host's one-shot cast VFX reach the guest.
    pending_events.mp_active = true;
}

/// Applies the +30% attacker effectiveness while a co-op guest is connected,
/// and clears it the instant they disconnect. Runs only on the co-op host (which
/// is always in an endless/roguelite InGame match), so the bonus is scoped to
/// exactly "co-op endless/roguelite AND a guest connected". Writes the dedicated
/// `coop_difficulty_bonus` field every frame so it self-clears.
pub(in crate::game) fn apply_coop_difficulty_buff(
    coop_connected: Option<Res<CoopGuestConnected>>,
    mut attackers: Query<(&mut Effectiveness, &Team), Without<Corpse>>,
) {
    let bonus = if coop_connected.is_some_and(|c| c.0) {
        crate::game::constants::COOP_ATTACKER_EFFECTIVENESS_BONUS
    } else {
        0.0
    };
    for (mut eff, team) in &mut attackers {
        if *team == Team::Attackers && (eff.coop_difficulty_bonus - bonus).abs() > f32::EPSILON {
            eff.coop_difficulty_bonus = bonus;
        }
    }
}

/// `OnEnter(InGameState::ScoreScreen)` (co-op host): tell the guest the level
/// ended and how, so it returns to the wizard tower (where it's pulled into the
/// NEXT level when the host clicks Continue — that's how multi-level co-op loops)
/// and (WS9) records its own progression. Host spell stats are sent so the guest
/// can attribute shared activity.
pub(in crate::game) fn send_coop_level_over(
    mut connection: ResMut<crate::networking::resources::NetworkConnection>,
    game_outcome: Res<crate::game::resources::GameOutcome>,
    kill_stats: Res<crate::game::resources::KillStats>,
    current_level: Res<crate::game::resources::CurrentLevel>,
) {
    use crate::networking::protocol::NetworkMessage;
    let victory = matches!(*game_outcome, crate::game::resources::GameOutcome::Victory);
    connection
        .outgoing_messages
        .push(NetworkMessage::CoopLevelOver {
            victory,
            level: current_level.0,
            defenders_killed: kill_stats.defenders_killed,
            attackers_killed: kill_stats.attackers_killed,
            undead_killed: kill_stats.undead_killed,
            elapsed_time: kill_stats.elapsed_time,
            // WS9 fills these from the host's `LocalWizardStats`.
            host_spell_damage: 0.0,
            host_spell_healing: 0.0,
        });
}

/// Tell an attached co-op guest that the host's run is over, so it returns to the
/// tower on purpose rather than by noticing the socket died.
///
/// `NetworkMessage::CoopRunEnded` has been handled by the guest since it was
/// introduced (`receive_coop_lifecycle` below) but had no sender anywhere in the
/// codebase — the guest only ever found out when the transport dropped underneath it.
///
/// Best-effort by nature: the message has to survive one frame in
/// `outgoing_messages` before a transport pump picks it up, and the quit paths that
/// call this go on to reset the connection. Both pumps are ordered to run before that
/// reset (`transport_bridge_system` in `PreUpdate`; the Steam pump is explicitly
/// `.after(ButtonActionSet)` so it ships what the click produced in the same frame),
/// so it lands in practice. If it ever loses the race, the guest still recovers
/// through its own disconnect detector — it just sees "The host disconnected."
/// instead of a clean return.
pub(crate) fn notify_guest_run_ended(
    connection: &mut crate::networking::resources::NetworkConnection,
    victory: bool,
) {
    if !connection.has_connected_guest() {
        return;
    }
    connection
        .outgoing_messages
        .push(crate::networking::protocol::NetworkMessage::CoopRunEnded { victory });
}

/// Co-op host: detect a guest disconnect during the match. The host keeps
/// playing solo — we just clear `CoopGuestConnected`, which lifts the +30%
/// attacker buff next frame. The guest's wizard proxy is left in place (inert)
/// so a future reconnect can reclaim it. (The host's own disconnect overlay is
/// versus-only — `detect_mp_disconnect` is `MultiplayerGame`-gated — so the
/// co-op host never gets kicked to a menu by a guest drop.)
pub(in crate::game) fn detect_coop_guest_disconnect(
    connection: Res<crate::networking::resources::NetworkConnection>,
    coop_connected: Option<ResMut<CoopGuestConnected>>,
    mut coop_pause: ResMut<super::coop_pause::CoopPauseState>,
    in_game_state: Option<Res<State<crate::state::InGameState>>>,
    mut next_in_game: ResMut<NextState<crate::state::InGameState>>,
) {
    use crate::networking::resources::ConnectionState;
    if let Some(mut cc) = coop_connected
        && cc.0
        && matches!(
            connection.state,
            ConnectionState::Failed | ConnectionState::Disconnected
        )
    {
        cc.0 = false;
        info!("[Co-op] Guest disconnected — host continues solo; +30% attacker buff removed.");
        // Don't strand the now-solo host on a sync-pause: whoever initiated it,
        // the resume can never arrive (a host-initiated resume goes to no one; a
        // guest-initiated pause's initiator just left), so force-resume.
        if coop_pause.active {
            coop_pause.active = false;
            coop_pause.initiator = None;
            if in_game_state.is_some_and(|s| *s.get() == crate::state::InGameState::Paused) {
                next_in_game.set(crate::state::InGameState::Running);
            }
        }
    }
}

/// Co-op guest: on a level end (or run end) return to the wizard tower. The
/// connection is preserved across this transition (`cleanup_mp_game` doesn't
/// reset it, and `reset_lobby_on_exit` skips co-op), so the guest sits in its
/// tower lobby and is pulled straight into the NEXT level when the host clicks
/// Continue and re-sends `StartGame` — that's how multi-level endless co-op
/// loops. (WS9 will record the guest's own progression here from the
/// `CoopLevelOver` payload before returning.)
pub(in crate::game) fn receive_coop_lifecycle(
    mut connection: ResMut<crate::networking::resources::NetworkConnection>,
    mut next_app_state: ResMut<NextState<crate::state::AppState>>,
    session: Option<Res<MultiplayerSession>>,
    coop_peer: Option<Res<CoopPeerInfo>>,
) {
    use crate::networking::protocol::NetworkMessage;
    let mut return_to_tower = false;
    let mut level_result: Option<CoopLevelResult> = None;
    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();
    for m in messages {
        match m {
            NetworkMessage::CoopLevelOver {
                victory,
                level,
                defenders_killed,
                attackers_killed,
                undead_killed,
                elapsed_time,
                ..
            } => {
                level_result = Some(CoopLevelResult {
                    victory,
                    level,
                    defenders_killed,
                    attackers_killed,
                    undead_killed,
                    elapsed_time,
                });
                return_to_tower = true;
            }
            NetworkMessage::CoopRunEnded { .. } => return_to_tower = true,
            other => unhandled.push(other),
        }
    }
    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }

    // Record the guest's OWN progression for this co-op level in a single save
    // load+save: lifetime counters (games/kills always, levels on victory) count
    // for both players, and an endless victory also updates this wizard's endless
    // best-stats with the contiguity rule. The full achievement-message replay for
    // the guest is a documented follow-up.
    if let (Some(r), Some(session)) = (level_result, session.as_deref()) {
        let total_defenders = (crate::game::constants::INITIAL_DEFENDER_COUNT
            + crate::game::units::archer::constants::INITIAL_ARCHER_DEFENDER_COUNT)
            as f32;
        let efficiency = 1.0 - (r.defenders_killed as f32 / total_defenders);
        crate::config::save_data::record_coop_guest_level_end(
            session.guest_wizard,
            session.mode == SessionMode::CoopEndless,
            r.victory,
            r.level,
            r.defenders_killed,
            r.attackers_killed,
            r.undead_killed,
            efficiency,
            r.elapsed_time,
            coop_peer.and_then(|p| p.name.clone()),
        );
    }

    if return_to_tower {
        next_app_state.set(crate::state::AppState::MetaGame);
    }
}

/// Outcome of a co-op level as received by the guest in a `CoopLevelOver`.
#[derive(Clone, Copy)]
struct CoopLevelResult {
    victory: bool,
    level: u32,
    defenders_killed: u32,
    attackers_killed: u32,
    undead_killed: u32,
    elapsed_time: f32,
}

/// `OnExit(AppState::InGame)`: tear down the co-op host's per-level networking
/// resources + session. Endless/roguelite loop THROUGH the wizard tower between
/// levels (win → score screen → tower → "Continue" → next level), so each co-op
/// level is an independent match: a fresh session is built on the next level's
/// load. The CONNECTION is preserved across this (no `connection.reset()` here,
/// and `reset_lobby_on_exit` skips co-op), so the host's tower lobby stays
/// connected and re-launches the next level. Versus uses `cleanup_mp_game`
/// (`OnExit(MultiplayerGame)`); this mirror is for the host's single-player path.
pub(in crate::game) fn cleanup_coop_host(
    mut commands: Commands,
    session: Option<Res<MultiplayerSession>>,
    mut pending_events: ResMut<PendingCastEvents>,
    guest_wizards: Query<Entity, With<crate::game::units::wizard::components::GuestWizard>>,
) {
    if !session
        .as_deref()
        .is_some_and(|s| s.is_coop() && s.role == PeerRole::Host)
    {
        return;
    }
    // The host's GuestWizard proxy is spawned via `spawn_mp_wizard`, which tags
    // it `OnMultiplayerGameScreen` — but the co-op host lives in `AppState::InGame`,
    // whose SP `cleanup_game` only despawns `OnGameplayScreen`. Despawn the proxy
    // here so it doesn't leak (one orphan per level, each running wizard systems).
    for entity in &guest_wizards {
        commands.entity(entity).try_despawn();
    }
    commands.remove_resource::<MultiplayerSession>();
    commands.remove_resource::<PeerId>();
    commands.remove_resource::<EntityIdCounter>();
    commands.remove_resource::<NetworkEntityMap>();
    commands.remove_resource::<SnapshotTick>();
    commands.remove_resource::<crate::networking::snapshot::SpellSnapshotData>();
    commands.remove_resource::<super::components::SpellEffectEntityMap>();
    commands.remove_resource::<super::spell_sync::LatestSpellSnapshot>();
    commands.remove_resource::<super::score_stats::LocalWizardStats>();
    commands.remove_resource::<CoopGuestConnected>();
    // Clear the queue as well as the flag. The versus path does both
    // (`spell_sync::mark_pending_events_mp_inactive`); dropping only the flag here
    // left any event emitted after the last drain of level N queued, and it then
    // shipped in level N+1's very first snapshot.
    pending_events.mp_active = false;
    pending_events.events.clear();
}
