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
    /// Poison damage
    ///
    /// Dealt by: Plague Wind, Spike Growth
    Poison,
    /// Poop damage
    ///
    /// Dealt by: Excremage wizard type
    Poop,
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
            DamageType::Poison => "Poison",
            DamageType::Poop => "Poop",
        }
    }

    /// Compact ordinal for network serialisation. Paired with `from_u8`.
    pub const fn to_u8(self) -> u8 {
        match self {
            DamageType::Force => 0,
            DamageType::Fire => 1,
            DamageType::Electric => 2,
            DamageType::Frost => 3,
            DamageType::Necrotic => 4,
            DamageType::Nature => 5,
            DamageType::Poison => 6,
            DamageType::Poop => 7,
        }
    }

    /// Inverse of `to_u8`; out-of-range values fall back to `Force`.
    pub const fn from_u8(byte: u8) -> Self {
        match byte {
            0 => DamageType::Force,
            1 => DamageType::Fire,
            2 => DamageType::Electric,
            3 => DamageType::Frost,
            4 => DamageType::Necrotic,
            5 => DamageType::Nature,
            6 => DamageType::Poison,
            7 => DamageType::Poop,
            _ => DamageType::Force,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trip every variant through `to_u8` / `from_u8`. Catches the
    /// "added a variant but forgot to update both conversions" footgun —
    /// the network wire format would otherwise silently misroute.
    #[test]
    fn damage_type_u8_roundtrip_is_exhaustive() {
        let all = [
            DamageType::Force,
            DamageType::Fire,
            DamageType::Electric,
            DamageType::Frost,
            DamageType::Necrotic,
            DamageType::Nature,
            DamageType::Poison,
            DamageType::Poop,
        ];
        // If a new variant is added to the enum but missing from `all`,
        // this exhaustiveness check ensures the test will fail to compile
        // once `all` is updated — surfacing the need to also update
        // `to_u8` / `from_u8`.
        for v in all {
            assert_eq!(
                DamageType::from_u8(v.to_u8()),
                v,
                "DamageType::{:?} roundtrip failed",
                v
            );
            // Exhaustiveness: every variant must produce a distinct byte.
            let _ensure_match_is_exhaustive = match v {
                DamageType::Force
                | DamageType::Fire
                | DamageType::Electric
                | DamageType::Frost
                | DamageType::Necrotic
                | DamageType::Nature
                | DamageType::Poison
                | DamageType::Poop => (),
            };
        }
    }
}
