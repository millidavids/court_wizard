//! Host → guest mirror of the host's currently-selected game mode.
//!
//! The host browses Endless/Roguelite/VS to pick what to start while the guest
//! waits on the Multiplayer tab with nothing to look at. This ships a
//! pre-formatted descriptor of the host's selection so the guest's left panel can
//! render it without needing any of the host-only game-mode resources.

use bevy::prelude::*;

use crate::config::GameConfig;
use crate::game::game_mode::components::{RogueliteModifiers, RogueliteRunState};
use crate::networking::protocol::{HostMode, NetworkMessage};
use crate::networking::resources::{ConnectionState, NetworkConnection, PeerRole};
use crate::ui::wizard_tower::layout::WizardTowerTab;
use crate::ui::wizard_tower::roguelite_tab::{PendingToggles, roguelite_summary_lines};

use super::state::{CoopHostSelection, LobbyPhase, MultiplayerLobby};

/// Host → guest: broadcast the host's currently-selected game mode so the guest's
/// Multiplayer-tab left panel mirrors what's about to start. Runs on ANY tab (the
/// host sits on Endless/Roguelite/VS while the guest waits on Multiplayer).
///
/// Dedups on the `CoopHostSelection` descriptor (it derives `PartialEq`), and ALSO
/// re-emits when the guest first arrives (`opponent_wizard` `None→Some`) so a late
/// or reconnecting guest receives the current selection even if it hasn't changed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn broadcast_host_mode_to_guest(
    tab: Res<WizardTowerTab>,
    config: Res<GameConfig>,
    lobby: Res<MultiplayerLobby>,
    roguelite_run: Option<Res<RogueliteRunState>>,
    roguelite_modifiers: Option<Res<RogueliteModifiers>>,
    pending_toggles: Option<Res<PendingToggles>>,
    mut connection: ResMut<NetworkConnection>,
    mut last_sent: Local<Option<CoopHostSelection>>,
    mut had_guest_wizard: Local<bool>,
) {
    // Host only, while connected and in wizard-select. Reset the dedup state when
    // out of this window so a fresh lobby re-broadcasts from scratch.
    if connection.role != Some(PeerRole::Host) || connection.state != ConnectionState::Connected {
        *last_sent = None;
        *had_guest_wizard = false;
        return;
    }
    let LobbyPhase::WizardSelect {
        my_wizard,
        opponent_wizard,
        ..
    } = &lobby.phase
    else {
        *last_sent = None;
        *had_guest_wizard = false;
        return;
    };

    // Re-send when the guest first appears (`opponent_wizard` None→Some), even if
    // the descriptor is unchanged from a prior guest.
    let guest_present = opponent_wizard.is_some();
    let guest_just_arrived = guest_present && !*had_guest_wizard;
    *had_guest_wizard = guest_present;
    let host_wizard = *my_wizard;

    // Cheap scalar descriptor for the current tab. `detail_lines` are built lazily
    // below: for every mode EXCEPT the roguelite no-run tab they're fully determined
    // by these scalars, so an unchanged scalar set with no new guest is a definitive
    // no-op — skip building the (allocating) detail lines and the descriptor.
    let (mode, level, is_continue) = match *tab {
        WizardTowerTab::Endless => (
            HostMode::Endless,
            config.current_level,
            config.highest_level_achieved > 1,
        ),
        WizardTowerTab::Roguelite => match roguelite_run.as_deref() {
            Some(run) => (HostMode::Roguelite, run.level_stats.len() as u32 + 1, true),
            None => (HostMode::Roguelite, 1, false),
        },
        WizardTowerTab::Vs => (HostMode::Versus, 0, false),
        // Multiplayer / Study / anything else: not a startable mode.
        _ => (HostMode::Browsing, 0, false),
    };
    let is_roguelite_no_run = mode == HostMode::Roguelite && !is_continue;
    let scalars_match = last_sent.as_ref().is_some_and(|s| {
        s.mode == mode
            && s.level == level
            && s.is_continue == is_continue
            && s.host_wizard == host_wizard
    });
    // Only the roguelite no-run summary can change without the scalars changing
    // (the host dragging modifier sliders), so always rebuild there; otherwise an
    // unchanged scalar set means nothing to do.
    if !guest_just_arrived && scalars_match && !is_roguelite_no_run {
        return;
    }

    let detail_lines = match (mode, roguelite_run.as_deref()) {
        (HostMode::Endless, _) => vec![format!("Level {level}")],
        (HostMode::Roguelite, Some(_)) => vec![format!("Level {level} (next)")],
        (HostMode::Roguelite, None) => {
            match (roguelite_modifiers.as_deref(), pending_toggles.as_deref()) {
                (Some(m), Some(t)) => roguelite_summary_lines(m, t),
                _ => vec!["New run".to_string()],
            }
        }
        (HostMode::Versus, _) => Vec::new(),
        (HostMode::Browsing, _) => vec!["Host is choosing a mode...".to_string()],
    };

    let descriptor = CoopHostSelection {
        mode,
        host_wizard,
        level,
        is_continue,
        detail_lines,
    };

    if guest_just_arrived || last_sent.as_ref() != Some(&descriptor) {
        connection
            .outgoing_messages
            .push(NetworkMessage::HostModeSelection {
                mode: descriptor.mode,
                host_wizard: descriptor
                    .host_wizard
                    .unwrap_or(crate::config::WizardType::BoringOleMage),
                level: descriptor.level,
                is_continue: descriptor.is_continue,
                detail_lines: descriptor.detail_lines.clone(),
            });
    }
    *last_sent = Some(descriptor);
}
