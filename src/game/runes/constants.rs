use super::resources::Rune;
use crate::game::units::wizard::components::Spell;

/// Maximum number of runes in a sequence.
pub const MAX_RUNE_SEQUENCE_LENGTH: usize = 3;

/// Timeout duration in seconds before sequence auto-clears.
pub const SEQUENCE_TIMEOUT_DURATION: f32 = 2.0;

/// Maps rune sequences to spells.
///
/// Returns None for invalid combinations.
pub fn sequence_to_spell(runes: &[Rune]) -> Option<Spell> {
    match runes {
        // Single runes
        [Rune::Q] => Some(Spell::MagicMissile),
        [Rune::W] => Some(Spell::Fireball),
        [Rune::E] => Some(Spell::Teleport),
        [Rune::R] => Some(Spell::GuardianCircle),

        // Two-rune combinations
        [Rune::Q, Rune::W] => Some(Spell::Disintegrate),
        [Rune::Q, Rune::E] => Some(Spell::ChainLightning),
        [Rune::W, Rune::E] => Some(Spell::WallOfStone),
        [Rune::W, Rune::R] => Some(Spell::RaiseTheDead),
        [Rune::E, Rune::R] => Some(Spell::FingerOfDeath),

        // Invalid or unsupported combinations
        _ => None,
    }
}

/// Returns true if the sequence is valid (maps to a spell).
pub fn is_valid_sequence(runes: &[Rune]) -> bool {
    sequence_to_spell(runes).is_some()
}
