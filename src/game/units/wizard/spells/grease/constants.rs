use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_GREASE: PrimedSpell = PrimedSpell {
    spell: Spell::Grease,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 1.5;
pub const MANA_COST: f32 = 25.0;
pub const CIRCLE_RADIUS: f32 = 150.0;
pub const SLOW_MODIFIER: f32 = -0.4;
pub const SLOW_DURATION: f32 = 1.0;
pub const TICK_INTERVAL: f32 = 0.5;
pub const ZONE_DURATION: f32 = 20.0;
pub const IGNITE_DAMAGE: f32 = 0.0;
pub const IGNITE_BURN_DAMAGE: f32 = 1.0;
pub const IGNITE_BURN_TICK: f32 = 0.5;
pub const CIRCLE_Y_POSITION: f32 = 1.0;
pub const FADE_DURATION: f32 = 2.0;
/// Max Y height a fire source can be at to ignite grease (filters out aerial spells)
pub const IGNITION_HEIGHT_THRESHOLD: f32 = 15.0;
/// Time in seconds for fire to spread across the full grease radius
pub const FIRE_SPREAD_DURATION: f32 = 1.0;
/// Y position of fire overlay (slightly above grease mesh)
pub const FIRE_OVERLAY_Y_POSITION: f32 = 1.5;
/// Fraction of zone radius for initial burst damage at ignition point
pub const IGNITION_BURST_RADIUS_FRACTION: f32 = 0.3;
