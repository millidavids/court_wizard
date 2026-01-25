use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// PrimedSpell constant for Guardian Circle.
pub const PRIMED_GUARDIAN_CIRCLE: PrimedSpell = PrimedSpell {
    spell: Spell::GuardianCircle,
    cast_time: CAST_TIME,
};

/// Cast time for Guardian Circle in seconds.
pub const CAST_TIME: f32 = 2.0;

/// Mana cost for casting Guardian Circle.
pub const MANA_COST: f32 = 50.0;

/// Radius of the protective circle in units.
pub const CIRCLE_RADIUS: f32 = 150.0;

/// Amount of temporary HP granted to units in the circle.
pub const TEMP_HP_AMOUNT: f32 = 30.0;

/// Duration of temporary HP in seconds.
pub const TEMP_HP_DURATION: f32 = 10.0;

/// Y position of the circle indicator (slightly above ground).
pub const CIRCLE_Y_POSITION: f32 = 1.0;
