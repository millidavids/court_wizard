use bevy::prelude::*;

/// Team component for all units.
///
/// Determines which side a unit is on. Units attack members of opposing teams.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Team {
    Defenders,
    Attackers,
    Undead,
}

impl Team {
    /// Returns true if units on these two teams are hostile to each other.
    /// Undead are hostile to everyone (including other Undead is false).
    pub fn is_enemy(&self, other: &Team) -> bool {
        match (self, other) {
            (Team::Undead, Team::Undead) => false,
            (Team::Undead, _) | (_, Team::Undead) => true,
            _ => self != other,
        }
    }
}
