//! Synchronized co-op pause.
//!
//! In a co-op match the HOST runs the simulation in `AppState::InGame`
//! (`InGameState`) while the GUEST spectates in `AppState::MultiplayerGame`
//! (`MultiplayerGameState`). Their pause states are otherwise independent, so this
//! module ties them together: either player pausing pauses BOTH, the other side's
//! pause menu shows its resume button disabled ("Waiting for other player"), and
//! only the player who initiated the pause can resume it.
//!
//! The authoritative state lives in [`CoopPauseState`] on each peer and is synced
//! with a single idempotent `NetworkMessage::CoopPauseSync`. The initiator re-sends
//! it as a low-rate heartbeat (and briefly after resuming) so a dropped packet
//! can't strand the other player.
//!
//! Disabled entirely when the Urgent roguelite toggle is active
//! (`MultiplayerSession.coop_urgent`): there pause stays local and the sim keeps
//! running, matching Urgent's "game keeps running while browsing menus" semantics.

use std::time::Duration;

use bevy::prelude::*;

use crate::game::input::action_state::{GamepadAction, GamepadActionState};
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{NetworkConnection, PeerRole};
use crate::networking::session::MultiplayerSession;
use crate::state::{InGameState, MultiplayerGameState};

/// How often the initiator re-broadcasts the authoritative pause state.
const HEARTBEAT_INTERVAL: f32 = 0.4;
/// How long after resuming the initiator keeps re-sending `paused:false` so a
/// dropped resume self-heals.
const GRACE_DURATION: f32 = 1.5;

/// The authoritative co-op pause state, mirrored on both peers.
#[derive(Resource)]
pub struct CoopPauseState {
    /// Whether the match is currently sync-paused.
    pub active: bool,
    /// Which peer owns the pause (only they may resume). Retained briefly after a
    /// resume so the heartbeat's grace window knows it was the local initiator.
    pub initiator: Option<PeerRole>,
    /// Drives the periodic re-broadcast of the authoritative state.
    heartbeat: Timer,
    /// Runs after a resume; while it is unfinished the initiator re-sends
    /// `paused:false`. Defaults to *finished* so a fresh state never re-sends.
    grace: Timer,
}

impl Default for CoopPauseState {
    fn default() -> Self {
        let mut grace = Timer::from_seconds(GRACE_DURATION, TimerMode::Once);
        // Start finished so the heartbeat's grace branch is inert until a resume.
        grace.tick(Duration::from_secs(3600));
        Self {
            active: false,
            initiator: None,
            heartbeat: Timer::from_seconds(HEARTBEAT_INTERVAL, TimerMode::Repeating),
            grace,
        }
    }
}

impl CoopPauseState {
    fn clear(&mut self) {
        self.active = false;
        self.initiator = None;
    }
}

// ── Run conditions ──────────────────────────────────────────────────

/// True for a co-op match where synchronized pause applies (co-op, non-Urgent).
pub(crate) fn coop_sync_pause_enabled(session: Option<Res<MultiplayerSession>>) -> bool {
    session.is_some_and(|s| s.coop_pause_synced())
}

/// Core check: may the local peer resume the current pause? True in single-player
/// and versus (no co-op sync-pause), true when no pause is active, and — during an
/// active co-op sync-pause — true only for the initiator. The single source of
/// truth shared by the run condition and the button/escape handlers.
pub(crate) fn local_can_resume(
    session: Option<&MultiplayerSession>,
    pause: Option<&CoopPauseState>,
) -> bool {
    let (Some(session), Some(pause)) = (session, pause) else {
        return true;
    };
    if !session.coop_pause_synced() || !pause.active {
        return true;
    }
    pause.initiator == Some(session.role)
}

/// Run-condition wrapper around [`local_can_resume`]. Gates the resume inputs
/// (Continue / Resume / Escape) so only the initiator can unpause.
pub(crate) fn coop_local_is_pause_controller(
    session: Option<Res<MultiplayerSession>>,
    pause: Option<Res<CoopPauseState>>,
) -> bool {
    local_can_resume(session.as_deref(), pause.as_deref())
}

// ── Non-initiator "waiting" relabel (shared by both pause menus) ────

/// Label shown on the non-initiator's resume button during a co-op sync-pause.
pub(crate) const COOP_WAITING_LABEL: &str = "Waiting for other player";
/// Muted colour for that disabled resume label.
pub(crate) const COOP_WAITING_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);

/// Relabels a button's text leaf to the "waiting" message + muted colour. Handles
/// both the pre-3D-conversion layout (text is a direct child) and the converted
/// layout (text reparented into the `ButtonFront` child). Shared by the SP and MP
/// pause-menu relabel systems so the traversal and constants live in one place.
pub(crate) fn set_waiting_label(
    children: &Children,
    front_q: &Query<&Children, With<crate::ui::components::ButtonFront>>,
    texts: &mut Query<(&mut Text, &mut TextColor)>,
) {
    let set = |entity: Entity, texts: &mut Query<(&mut Text, &mut TextColor)>| -> bool {
        if let Ok((mut text, mut color)) = texts.get_mut(entity) {
            *text = Text::new(COOP_WAITING_LABEL);
            *color = TextColor(COOP_WAITING_COLOR);
            true
        } else {
            false
        }
    };
    for child in children.iter() {
        if set(child, texts) {
            continue;
        }
        if let Ok(front_children) = front_q.get(child) {
            for gc in front_children.iter() {
                set(gc, texts);
            }
        }
    }
}

// ── Reset on match enter ────────────────────────────────────────────

/// Clears the pause state at the start of each co-op level so a stale `active`
/// never carries over (endless/roguelite loop through the tower between levels).
pub(crate) fn reset_coop_pause_state(mut pause: ResMut<CoopPauseState>) {
    *pause = CoopPauseState::default();
}

// ── Local input (owns the Running↔Paused toggle in co-op) ───────────

/// Classification of the local gameplay state for the active peer.
pub(crate) enum LocalState {
    Running,
    Paused,
    Other,
}

pub(crate) fn local_state_host(state: Option<&State<InGameState>>) -> LocalState {
    match state.map(|s| *s.get()) {
        Some(InGameState::Running) => LocalState::Running,
        Some(InGameState::Paused) => LocalState::Paused,
        _ => LocalState::Other,
    }
}

pub(crate) fn local_state_guest(state: Option<&State<MultiplayerGameState>>) -> LocalState {
    match state.map(|s| *s.get()) {
        Some(MultiplayerGameState::Running) => LocalState::Running,
        Some(MultiplayerGameState::Paused) => LocalState::Paused,
        _ => LocalState::Other,
    }
}

/// Reads Escape / gamepad-Start and drives the co-op sync-pause:
/// - Running & not paused → initiate (pause both, become the initiator).
/// - Paused & I am the initiator → resume (unpause both).
/// - Paused & I am NOT the initiator → ignored (only the initiator resumes).
///
/// The default SP `keyboard_input` and the guest `mp_escape_key_handler`'s
/// Running/Paused arms are gated off in co-op so this is the sole pause owner.
#[allow(clippy::too_many_arguments)]
pub(crate) fn coop_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    active_device: Res<ActiveInputDevice>,
    action_state: Res<GamepadActionState>,
    session: Res<MultiplayerSession>,
    mut pause: ResMut<CoopPauseState>,
    mut connection: ResMut<NetworkConnection>,
    sp_state: Option<Res<State<InGameState>>>,
    mut next_sp: Option<ResMut<NextState<InGameState>>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    mut next_mp: Option<ResMut<NextState<MultiplayerGameState>>>,
) {
    let gamepad_start =
        active_device.is_gamepad() && action_state.just_pressed(GamepadAction::Pause);
    if !keyboard.just_pressed(KeyCode::Escape) && !gamepad_start {
        return;
    }

    let local = session.role;
    let is_host = local == PeerRole::Host;
    let local_state = if is_host {
        local_state_host(sp_state.as_deref())
    } else {
        local_state_guest(mp_state.as_deref())
    };

    let set_running =
        |next_sp: &mut Option<ResMut<NextState<InGameState>>>,
         next_mp: &mut Option<ResMut<NextState<MultiplayerGameState>>>| {
            if is_host {
                if let Some(n) = next_sp {
                    n.set(InGameState::Running);
                }
            } else if let Some(n) = next_mp {
                n.set(MultiplayerGameState::Running);
            }
        };

    match local_state {
        LocalState::Running if !pause.active => {
            do_initiate_pause(
                is_host,
                local,
                &mut pause,
                &mut connection,
                &mut next_sp,
                &mut next_mp,
            );
        }
        LocalState::Paused if pause.active && pause.initiator == Some(local) => {
            pause.active = false;
            pause.grace.reset();
            set_running(&mut next_sp, &mut next_mp);
            connection
                .outgoing_messages
                .push(NetworkMessage::CoopPauseSync {
                    paused: false,
                    initiator_is_host: is_host,
                });
        }
        _ => {}
    }
}

/// Performs the local side-effects of initiating a co-op sync-pause: marks
/// [`CoopPauseState`], transitions the local peer's state to `Paused`, and
/// broadcasts the authoritative `CoopPauseSync`. Shared by the manual
/// Escape/Start handler ([`coop_pause_input`]) and the auto-pause request
/// consumer (`pause_request::apply_pause_requests`) so the co-op initiation
/// logic lives in exactly one place. Callers must ensure no pause is already
/// active and that the local peer is in its `Running` state.
pub(crate) fn do_initiate_pause(
    is_host: bool,
    local: PeerRole,
    pause: &mut CoopPauseState,
    connection: &mut NetworkConnection,
    next_sp: &mut Option<ResMut<NextState<InGameState>>>,
    next_mp: &mut Option<ResMut<NextState<MultiplayerGameState>>>,
) {
    pause.active = true;
    pause.initiator = Some(local);
    pause.heartbeat.reset();
    if is_host {
        if let Some(n) = next_sp {
            n.set(InGameState::Paused);
        }
    } else if let Some(n) = next_mp {
        n.set(MultiplayerGameState::Paused);
    }
    connection
        .outgoing_messages
        .push(NetworkMessage::CoopPauseSync {
            paused: true,
            initiator_is_host: is_host,
        });
}

// ── Receive (apply the remote peer's authoritative pause state) ─────

/// Co-op HOST: apply incoming `CoopPauseSync` to the host's `InGameState`.
pub(crate) fn coop_pause_receive_host(
    mut connection: ResMut<NetworkConnection>,
    mut pause: ResMut<CoopPauseState>,
    state: Res<State<InGameState>>,
    mut next: ResMut<NextState<InGameState>>,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }
    let msgs: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();
    for m in msgs {
        if let NetworkMessage::CoopPauseSync {
            paused,
            initiator_is_host,
        } = m
        {
            apply_remote_pause_host(
                paused,
                initiator_is_host,
                &mut pause,
                state.get(),
                &mut next,
            );
        } else {
            unhandled.push(m);
        }
    }
    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Co-op GUEST: apply incoming `CoopPauseSync` to the guest's `MultiplayerGameState`.
pub(crate) fn coop_pause_receive_guest(
    mut connection: ResMut<NetworkConnection>,
    mut pause: ResMut<CoopPauseState>,
    state: Res<State<MultiplayerGameState>>,
    mut next: ResMut<NextState<MultiplayerGameState>>,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }
    let msgs: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();
    for m in msgs {
        if let NetworkMessage::CoopPauseSync {
            paused,
            initiator_is_host,
        } = m
        {
            apply_remote_pause_guest(
                paused,
                initiator_is_host,
                &mut pause,
                state.get(),
                &mut next,
            );
        } else {
            unhandled.push(m);
        }
    }
    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

fn apply_remote_pause_host(
    paused: bool,
    initiator_is_host: bool,
    pause: &mut CoopPauseState,
    state: &InGameState,
    next: &mut NextState<InGameState>,
) {
    let incoming = if initiator_is_host {
        PeerRole::Host
    } else {
        PeerRole::Guest
    };
    if paused {
        // Tiebreak: a simultaneously-initiated pause resolves in the host's favor —
        // ignore the guest's pause if we already own an active pause as host.
        if pause.active && pause.initiator == Some(PeerRole::Host) && !initiator_is_host {
            return;
        }
        pause.active = true;
        pause.initiator = Some(incoming);
        if *state != InGameState::Paused {
            next.set(InGameState::Paused);
        }
    } else {
        // Honor a remote resume only for the pause it owns (guards against a stale
        // resume from a prior cycle clearing a freshly host-initiated pause).
        if pause.initiator == Some(incoming) || pause.initiator.is_none() {
            pause.clear();
            if *state == InGameState::Paused {
                next.set(InGameState::Running);
            }
        }
    }
}

fn apply_remote_pause_guest(
    paused: bool,
    initiator_is_host: bool,
    pause: &mut CoopPauseState,
    state: &MultiplayerGameState,
    next: &mut NextState<MultiplayerGameState>,
) {
    let incoming = if initiator_is_host {
        PeerRole::Host
    } else {
        PeerRole::Guest
    };
    if paused {
        // The host always wins a simultaneous tie, so the guest always yields to an
        // incoming host-initiated pause (downgrading itself to non-initiator).
        pause.active = true;
        pause.initiator = Some(incoming);
        if *state != MultiplayerGameState::Paused {
            next.set(MultiplayerGameState::Paused);
        }
    } else if pause.initiator == Some(incoming) || pause.initiator.is_none() {
        pause.clear();
        if *state == MultiplayerGameState::Paused {
            next.set(MultiplayerGameState::Running);
        }
    }
}

// ── Heartbeat (initiator self-heals dropped pause/resume packets) ───

/// The pause initiator re-broadcasts the authoritative state on a timer: while
/// paused it re-sends `paused:true`; for a short grace window after resuming it
/// re-sends `paused:false`, so a dropped resume can't leave the other peer stuck.
pub(crate) fn coop_pause_heartbeat(
    time: Res<Time>,
    session: Res<MultiplayerSession>,
    mut pause: ResMut<CoopPauseState>,
    mut connection: ResMut<NetworkConnection>,
) {
    let local = session.role;
    if pause.initiator != Some(local) {
        return;
    }
    let is_host = local == PeerRole::Host;
    if pause.active {
        pause.heartbeat.tick(time.delta());
        if pause.heartbeat.just_finished() {
            connection
                .outgoing_messages
                .push(NetworkMessage::CoopPauseSync {
                    paused: true,
                    initiator_is_host: is_host,
                });
        }
    } else if !pause.grace.is_finished() {
        let delta = time.delta();
        pause.grace.tick(delta);
        pause.heartbeat.tick(delta);
        if pause.heartbeat.just_finished() {
            connection
                .outgoing_messages
                .push(NetworkMessage::CoopPauseSync {
                    paused: false,
                    initiator_is_host: is_host,
                });
        }
        // Once the grace window closes, drop ownership so a later remote pause is
        // adopted cleanly.
        if pause.grace.just_finished() {
            pause.initiator = None;
        }
    }
}
