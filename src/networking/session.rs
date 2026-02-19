//! Multiplayer session resource and run conditions.

use bevy::prelude::*;

use super::resources::PeerRole;
use crate::config::WizardType;
use crate::game::units::wizard::components::Spell;

/// Tracks the active multiplayer session configuration.
///
/// Inserted when both players are ready to start a match.
/// Contains all data needed to set up the multiplayer battlefield.
#[derive(Resource)]
pub struct MultiplayerSession {
    /// Whether this peer is the host or guest.
    pub role: PeerRole,

    /// Wizard type chosen by the host.
    pub host_wizard: WizardType,

    /// Wizard type chosen by the guest.
    pub guest_wizard: WizardType,

    /// Spells available to the host (from their unlocked spells).
    pub host_spells: Vec<Spell>,

    /// Spells available to the guest (from their unlocked spells).
    pub guest_spells: Vec<Spell>,

    /// Host's action bar configuration.
    pub host_action_bar: [Option<Spell>; 5],

    /// Guest's action bar configuration.
    pub guest_action_bar: [Option<Spell>; 5],
}

/// Returns true when in a multiplayer game as the host.
pub fn is_multiplayer_host(session: Option<Res<MultiplayerSession>>) -> bool {
    session.is_some_and(|s| s.role == PeerRole::Host)
}

/// Returns true when in a multiplayer game as the guest.
pub fn is_multiplayer_guest(session: Option<Res<MultiplayerSession>>) -> bool {
    session.is_some_and(|s| s.role == PeerRole::Guest)
}
