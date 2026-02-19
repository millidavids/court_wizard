//! Multiplayer game plugin.
//!
//! Registers all multiplayer gameplay systems. This is completely independent
//! from `GamePlugin` — it reuses shared helper functions but has its own
//! system registrations under `AppState::MultiplayerGame`.

use bevy::prelude::*;

/// Plugin that manages multiplayer gameplay.
///
/// Placeholder — systems will be registered as milestones 3 and 4 are implemented.
pub struct MultiplayerGamePlugin;

impl Plugin for MultiplayerGamePlugin {
    fn build(&self, _app: &mut App) {
        // TODO: Milestone 3 — multiplayer loading systems
        // TODO: Milestone 4 — host simulation + guest rendering systems
    }
}
