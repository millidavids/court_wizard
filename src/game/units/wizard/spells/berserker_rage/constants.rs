use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_BERSERKER_RAGE: PrimedSpell = PrimedSpell {
    spell: Spell::BerserkerRage,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 1.0;
pub const MANA_COST: f32 = 25.0;
pub const CIRCLE_RADIUS: f32 = 150.0;
pub const DAMAGE_BONUS: f32 = 0.8;
pub const DAMAGE_VULNERABILITY: f32 = 0.5;
pub const BUFF_DURATION: f32 = 8.0;

// Talent constants
// Tier 1
pub const BLOOD_FURY_DAMAGE_BONUS: f32 = 1.2;
pub const BLOOD_FURY_VULNERABILITY: f32 = 0.65;
pub const CONTROLLED_RAGE_DAMAGE_BONUS: f32 = 0.6;
pub const CONTROLLED_RAGE_VULNERABILITY: f32 = 0.3;
pub const PRIMAL_ROAR_RADIUS_MULT: f32 = 1.5;
// Tier 2
pub const BLOODLUST_HEAL_FRACTION: f32 = 0.15;
pub const UNDYING_FURY_DURATION: f32 = 2.0;
pub const FRENZY_ATTACK_SPEED_BONUS: f32 = 0.3;
pub const FRENZY_HP_THRESHOLD: f32 = 0.5;
// Tier 3
pub const WRATH_INCARNATE_DAMAGE_BONUS: f32 = 2.0;
pub const WRATH_INCARNATE_VULNERABILITY: f32 = 1.0;
pub const CONTAGIOUS_RAGE_EFFECTIVENESS_LOSS: f32 = 0.2;
pub const FINAL_STAND_DAMAGE_FRACTION: f32 = 0.5;
pub const FINAL_STAND_RADIUS: f32 = 80.0;
/// Duration of the Final Stand explosion visual effect.
pub const FINAL_STAND_VFX_LIFETIME: f32 = 0.4;
/// Number of fire sparks spawned by Final Stand explosion.
pub const FINAL_STAND_SPARK_COUNT: usize = 12;
