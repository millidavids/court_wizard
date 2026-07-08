//! Auto-pause when the Steam overlay is opened.
//!
//! Steam recommends single-player / local games pause while the overlay is up.
//! We listen for the `GameOverlayActivated` callback on the existing
//! `SteamworksEvent` bus and request a pause through the shared
//! [`RequestGamePauseMessage`] consumer, which handles every mode: single-player
//! and synchronized co-op freeze, while versus and Urgent co-op pop the local
//! peer's non-freezing pause menu (the real-time sim keeps running).

use bevy::prelude::*;
use bevy_steamworks::{CallbackResult, SteamworksEvent};

use crate::config::GameConfig;
use crate::game::multiplayer::pause_request::RequestGamePauseMessage;

/// Requests a pause when the Steam overlay opens (`active == true`). Does NOT
/// auto-resume on close — the player resumes manually, matching the
/// window-focus-loss behavior.
pub(super) fn pause_on_steam_overlay(
    mut events: MessageReader<SteamworksEvent>,
    config: Res<GameConfig>,
    mut pause_writer: MessageWriter<RequestGamePauseMessage>,
) {
    if !config.pause_on_steam_overlay {
        return;
    }
    for evt in events.read() {
        if let SteamworksEvent::CallbackResult(CallbackResult::GameOverlayActivated(overlay)) = evt
            && overlay.active
        {
            pause_writer.write(RequestGamePauseMessage);
        }
    }
}
