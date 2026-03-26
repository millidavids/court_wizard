use super::resources::Rune;
use crate::game::units::wizard::components::Spell;

/// Maximum number of runes in a sequence.
pub const MAX_RUNE_SEQUENCE_LENGTH: usize = 4;

/// Timeout duration in seconds before sequence auto-clears.
pub const SEQUENCE_TIMEOUT_DURATION: f32 = 2.0;

/// Maps rune sequences to spells based on spell web distance from center.
///
/// Each branch starts with its root rune:
/// - Q = Offense (Magic Missile)
/// - W = Control (Entangle)
/// - E = Support (Guardian Circle)
/// - R = Utility (Telekinesis)
///
/// Combo length = ring distance on the spell web (1–4).
/// Each rune in the combo traces the path through the prerequisite tree.
pub fn sequence_to_spell(runes: &[Rune]) -> Option<Spell> {
    use Rune::*;

    match runes {
        // Ring 1
        [Q] => Some(Spell::MagicMissile),
        [W] => Some(Spell::Entangle),
        [E] => Some(Spell::GuardianCircle),
        [R] => Some(Spell::Telekinesis),

        // Ring 2
        [Q, Q] => Some(Spell::PlagueWind),
        [Q, W] => Some(Spell::Disintegrate),
        [Q, E] => Some(Spell::ChainLightning),
        [Q, R] => Some(Spell::FingerOfDeath),
        [W, Q] => Some(Spell::Grease),
        [W, W] => Some(Spell::SpikeGrowth),
        [W, E] => Some(Spell::Sleep),
        [E, Q] => Some(Spell::BattleHymn),
        [E, W] => Some(Spell::FogCloud),
        [E, E] => Some(Spell::BerserkerRage),
        [R, Q] => Some(Spell::Dispel),
        [R, W] => Some(Spell::Banishment),
        [R, E] => Some(Spell::ArcaneCrystal),

        // Ring 3
        [Q, W, Q] => Some(Spell::Fireball),
        [Q, E, Q] => Some(Spell::LightningRod),
        [Q, R, Q] => Some(Spell::MarkOfDeath),
        [W, Q, W] => Some(Spell::WallOfFire),
        [W, W, Q] => Some(Spell::WallOfStone),
        [W, W, E] => Some(Spell::Squall),
        [W, E, Q] => Some(Spell::MindControl),
        [W, E, W] => Some(Spell::Polymorph),
        [E, Q, W] => Some(Spell::HealingPlume),
        [E, Q, E] => Some(Spell::Haste),
        [E, W, Q] => Some(Spell::Teleport),
        [E, E, Q] => Some(Spell::RaiseTheDead),

        // Ring 4
        [Q, W, Q, W] => Some(Spell::MeteorFall),
        [W, E, W, Q] => Some(Spell::BlackHole),

        _ => None,
    }
}
