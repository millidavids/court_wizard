use super::super::super::components::{PrimedSpell, Spell};

/// Spell configuration for Raise The Dead
pub const PRIMED_RAISE_THE_DEAD: PrimedSpell = PrimedSpell {
    spell: Spell::RaiseTheDead,
    cast_time: 1.0, // 1 second cast time
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

/// Initial interval between resurrections (in seconds)
pub const INITIAL_CHANNEL_INTERVAL: f32 = 0.8;

/// Minimum interval between resurrections after ramp-up (in seconds)
pub const MIN_CHANNEL_INTERVAL: f32 = 0.3;

/// Time it takes to ramp from initial to minimum interval (in seconds)
pub const CHANNEL_RAMP_TIME: f32 = 4.0;

/// Mana cost per resurrected corpse
pub const MANA_COST_PER_CORPSE: f32 = 10.0;

/// Radius around cursor to search for corpses (in world units)
pub const RESURRECTION_RADIUS: f32 = 150.0;

// === Talent Constants ===

// Tier 1: Mass Graves
pub const MASS_GRAVES_RADIUS_MULT: f32 = 1.6;

// Tier 1: Efficient Necromancy
pub const EFFICIENT_NECROMANCY_MANA_MULT: f32 = 0.7;

// Tier 2: Empowered Undead
pub const EMPOWERED_UNDEAD_HP_MULT: f32 = 1.5;
pub const EMPOWERED_UNDEAD_DAMAGE_MULT: f32 = 1.25;

// Tier 2: Plague Bearer
pub const PLAGUE_BEARER_DPS: f32 = 3.0;
pub const PLAGUE_BEARER_RADIUS: f32 = 60.0;
pub const PLAGUE_BEARER_TICK_INTERVAL: f32 = 0.5;

// Tier 2: Corpse Magnet
pub const CORPSE_MAGNET_RADIUS: f32 = 400.0;
pub const CORPSE_MAGNET_PULL_SPEED: f32 = 200.0;

// Tier 3: Revenant Lord
pub const REVENANT_HP_MULT: f32 = 5.0;
pub const REVENANT_DAMAGE_MULT: f32 = 3.0;
pub const REVENANT_RAISE_RADIUS: f32 = 150.0;
pub const REVENANT_RAISE_INTERVAL: f32 = 2.0;

// Tier 3: Undead Detonation
pub const UNDEAD_DETONATION_DAMAGE: f32 = 50.0;
pub const UNDEAD_DETONATION_RADIUS: f32 = 80.0;

// Tier 3: Perpetual Unrest — proximity radius for auto-raise on nearby kills
pub const PERPETUAL_UNREST_RADIUS: f32 = 80.0;
