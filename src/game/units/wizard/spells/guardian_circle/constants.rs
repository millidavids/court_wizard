use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// PrimedSpell constant for Guardian Circle.
pub const PRIMED_GUARDIAN_CIRCLE: PrimedSpell = PrimedSpell {
    spell: Spell::GuardianCircle,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

/// Cast time for Guardian Circle in seconds.
pub const CAST_TIME: f32 = 2.0;

/// Mana cost for casting Guardian Circle.
pub const MANA_COST: f32 = 12.5;

/// Radius of the protective circle in units.
pub const CIRCLE_RADIUS: f32 = 150.0;

/// Amount of temporary HP granted to units in the circle.
pub const TEMP_HP_AMOUNT: f32 = 50.0;

/// Duration of temporary HP in seconds.
pub const TEMP_HP_DURATION: f32 = 20.0;

// === Talent Constants ===

/// Tier 1, Choice 0: Reinforced Wards — temp HP multiplier.
pub(super) const REINFORCED_WARDS_MULT: f32 = 1.4;

/// Tier 1, Choice 1: Enduring Protection — duration multiplier.
pub(super) const ENDURING_PROTECTION_MULT: f32 = 1.6;

/// Tier 1, Choice 2: Expansive Aegis — radius multiplier.
pub(super) const EXPANSIVE_AEGIS_RADIUS_MULT: f32 = 1.5;

/// Tier 1, Choice 2: Expansive Aegis — temp HP reduction.
pub(super) const EXPANSIVE_AEGIS_HP_MULT: f32 = 0.85;

/// Tier 2, Choice 0: Retaliating Wards — burst damage when temp HP breaks.
pub(super) const RETALIATING_WARDS_DAMAGE: f32 = 30.0;

/// Tier 2, Choice 0: Retaliating Wards — burst damage radius.
pub(super) const RETALIATING_WARDS_RADIUS: f32 = 80.0;

/// Tier 2, Choice 1: Fortified Resolve — damage bonus while shielded.
pub(super) const FORTIFIED_RESOLVE_DAMAGE_MULT: f32 = 0.2;

/// Tier 2, Choice 2: Rapid Deployment — cast time multiplier.
pub(super) const RAPID_DEPLOYMENT_CAST_MULT: f32 = 0.5;

/// Tier 3, Choice 0: Sanctuary — damage reduction while shielded.
pub(super) const SANCTUARY_DAMAGE_REDUCTION: f32 = 0.3;

/// Tier 3, Choice 1: Martyrdom — explosion radius on death.
pub(super) const MARTYRDOM_DAMAGE_RADIUS: f32 = 100.0;

/// Tier 3, Choice 2: Chain Ward — max number of hops.
pub(super) const CHAIN_WARD_MAX_HOPS: u32 = 3;
