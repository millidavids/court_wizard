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
