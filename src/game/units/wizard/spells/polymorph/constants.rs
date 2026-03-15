use crate::game::units::wizard::components::{PrimedSpell, Spell};
use bevy::prelude::*;

pub const PRIMED_POLYMORPH: PrimedSpell = PrimedSpell {
    spell: Spell::Polymorph,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 0.8;
pub const MANA_COST: f32 = 35.0;
pub const POLYMORPH_DURATION: f32 = 10.0;
pub const SHEEP_HP: f32 = 20.0;
pub const TARGET_SEARCH_RADIUS: f32 = 80.0;
pub const SHEEP_COLOR: Color = Color::srgba(0.9, 0.9, 0.85, 1.0);

// --- Talent constants ---

// Tier 1
pub const EXTENDED_DURATION: f32 = 14.0;
pub const FRAGILE_SHEEP_HP: f32 = 5.0;
pub const QUICK_SHAPESHIFT_CAST_MULT: f32 = 0.6;

// Tier 2 - Explosive Sheep
pub const EXPLOSIVE_SHEEP_DAMAGE: f32 = 40.0;
pub const EXPLOSIVE_SHEEP_RADIUS: f32 = 30.0;

// Tier 2 - Contagious Baas (spreads on expiry, no constants needed)

// Tier 2 - Pig Form
pub const PIG_SPEED: f32 = 60.0;
pub const PIG_COLOR: Color = Color::srgba(0.85, 0.6, 0.5, 1.0);

// Tier 3 - Mass Polymorph
pub const MASS_POLYMORPH_RADIUS: f32 = 35.0;
pub const MASS_POLYMORPH_MANA_MULT: f32 = 3.0;

// Tier 3 - Dire Sheep
pub const DIRE_SHEEP_HP: f32 = 200.0;
pub const DIRE_SHEEP_DAMAGE: f32 = 15.0;
pub const DIRE_SHEEP_ATTACK_INTERVAL: f32 = 1.5;
pub const DIRE_SHEEP_ATTACK_RADIUS: f32 = 15.0;
pub const DIRE_SHEEP_MOVE_SPEED: f32 = 40.0;
pub const DIRE_SHEEP_COLOR: Color = Color::srgba(0.95, 0.95, 0.9, 1.0);
