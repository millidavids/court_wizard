use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_BANISHMENT: PrimedSpell = PrimedSpell {
    spell: Spell::Banishment,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 1.0;
pub const MANA_COST: f32 = 40.0;
pub const BANISH_DURATION: f32 = 8.0;

// Talent constants
// Tier 1
/// Extended Exile: banish duration becomes 12 seconds.
pub const EXTENDED_EXILE_DURATION: f32 = 12.0;
/// Quick Dismissal: cast time multiplier (50% reduction).
pub const QUICK_DISMISSAL_CAST_TIME_MULT: f32 = 0.5;
/// Cheap Ticket: mana cost multiplier (30% reduction).
pub const CHEAP_TICKET_MANA_MULT: f32 = 0.7;

// Tier 2
/// Painful Return: damage dealt when banishment expires.
pub const PAINFUL_RETURN_DAMAGE: f32 = 120.0;
/// Dual Banishment: mana cost multiplier for the second target.
pub const DUAL_BANISHMENT_SECOND_MANA_MULT: f32 = 0.5;

// Tier 3
/// Dimensional Shunt: fraction of max HP the unit returns at.
pub const DIMENSIONAL_SHUNT_HP_FRACTION: f32 = 0.5;
/// Mass Banishment: radius for AoE banish.
pub const MASS_BANISHMENT_RADIUS: f32 = 100.0;
/// Mass Banishment: mana cost (very high).
pub const MASS_BANISHMENT_MANA_COST: f32 = 100.0;
/// Mass Banishment: shorter duration.
pub const MASS_BANISHMENT_DURATION: f32 = 4.0;
/// One-Way Trip: HP threshold (fraction) below which the unit doesn't return.
pub const ONE_WAY_TRIP_HP_THRESHOLD: f32 = 0.2;
/// Displacement: maximum random offset distance from original position.
pub const DISPLACEMENT_RADIUS: f32 = 800.0;

// VFX constants
/// Starting radius for the shrinking lensing sphere.
pub const VFX_START_RADIUS: f32 = 25.0;
/// Duration of the lensing shrink effect in seconds.
pub const VFX_LIFETIME: f32 = 0.5;
/// Number of spark particles per banishment.
pub const SPARK_COUNT: usize = 12;
