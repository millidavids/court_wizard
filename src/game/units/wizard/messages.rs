use bevy::prelude::*;

use super::components::PrimedSpell;

/// Message sent to prime a spell for casting.
/// Used by UI systems to request spell changes without direct component access.
#[derive(Message, Debug, Clone, Copy)]
pub struct PrimeSpellMessage {
    pub spell: PrimedSpell,
}
