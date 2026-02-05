use bevy::prelude::*;

/// Types of damage that can be dealt by spells and effects.
///
/// Damage types enable future implementations of resistances, vulnerabilities,
/// and elemental interactions between spells and effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
#[allow(dead_code)]
pub enum DamageType {
    /// Physical force damage
    ///
    /// Dealt by: Magic Missile, Disintegrate, Black Hole
    Force,
    /// Fire damage
    ///
    /// Dealt by: Fireball
    Fire,
    /// Electric damage
    ///
    /// Dealt by: Chain Lightning
    Electric,
    /// Frost damage
    ///
    /// Dealt by: Squall
    Frost,
    /// Necrotic damage (reserved for future spells)
    Necrotic,
}
