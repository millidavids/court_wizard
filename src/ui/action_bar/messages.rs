use bevy::prelude::*;

use crate::game::units::wizard::components::Spell;

/// Message sent when a spell should be assigned to an action bar slot.
#[derive(Message, Debug, Clone, Copy)]
pub struct AssignSpellToSlot {
    /// The slot index to assign to.
    pub slot: u8,
    /// The spell to assign.
    pub spell: Spell,
}
