use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_BATTLE_HYMN: PrimedSpell = PrimedSpell {
    spell: Spell::BattleHymn,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 1.5;
pub const MANA_COST: f32 = 35.0;
pub const CIRCLE_RADIUS: f32 = 200.0;
pub const DAMAGE_BONUS: f32 = 0.4;
pub const ATTACK_SPEED_BONUS: f32 = 0.3;
pub const BUFF_DURATION: f32 = 10.0;

// Talent tuning
/// Wide Anthem (tier 1): radius multiplier for the aura circle.
pub const WIDE_ANTHEM_RADIUS_MULT: f32 = 1.4;
/// Inspiring Words (tier 1): duration multiplier.
pub const INSPIRING_WORDS_DURATION_MULT: f32 = 1.5;
/// War Drums (tier 1): damage bonus multiplier.
pub const WAR_DRUMS_DAMAGE_MULT: f32 = 1.5;
/// Hymn of Legends (tier 3): multiplier applied to both damage bonus and attack speed.
pub const HYMN_OF_LEGENDS_MULT: f32 = 2.0;
/// Anthem Resilience (tier 3): damage reduction amount.
pub const ANTHEM_RESILIENCE_REDUCTION: f32 = 0.3;
/// Fortifying Hymn (tier 2): temporary HP granted per cast.
pub const FORTIFYING_HYMN_TEMP_HP: f32 = 20.0;
/// Swift March (tier 2): movement speed bonus fraction.
pub const SWIFT_MARCH_SPEED_BONUS: f32 = 0.25;
