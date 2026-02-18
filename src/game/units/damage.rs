use bevy::prelude::*;

/// Types of damage that can be dealt by spells and effects.
///
/// Damage types enable future implementations of resistances, vulnerabilities,
/// and elemental interactions between spells and effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
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
    /// Nature damage
    ///
    /// Dealt by: Entangle, Spike Growth
    Nature,
}

impl DamageType {
    /// Returns a human-readable display name for this damage type.
    pub const fn display_name(&self) -> &'static str {
        match self {
            DamageType::Force => "Force",
            DamageType::Fire => "Fire",
            DamageType::Electric => "Electric",
            DamageType::Frost => "Frost",
            DamageType::Necrotic => "Necrotic",
            DamageType::Nature => "Nature",
        }
    }
}
