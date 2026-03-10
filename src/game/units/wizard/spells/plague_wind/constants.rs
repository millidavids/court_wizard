use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_PLAGUE_WIND: PrimedSpell = PrimedSpell {
    spell: Spell::PlagueWind,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 1.5;
pub const MANA_COST: f32 = 35.0;
pub const CLOUD_RADIUS: f32 = 100.0;
pub const CLOUD_DURATION: f32 = 12.0;
pub const CLOUD_SPEED: f32 = 80.0;

/// Arrow indicator dimensions.
pub const ARROW_LENGTH: f32 = 80.0;
pub const ARROW_WIDTH: f32 = 24.0;
pub const DAMAGE_PER_TICK: f32 = 5.0;
pub const TICK_INTERVAL: f32 = 0.5;
pub const FADE_DURATION: f32 = 2.0;

pub const CLOUD_BASE_Y: f32 = 0.5;
// ===== Talent Constants =====

// -- Tier 1 --

/// T1-0 Virulent Strain: damage multiplier.
pub(super) const VIRULENT_STRAIN_DAMAGE_MULT: f32 = 1.6;

/// T1-1 Miasma: radius multiplier.
pub(super) const MIASMA_RADIUS_MULT: f32 = 1.5;

/// T1-1 Miasma: duration multiplier (reduced).
pub(super) const MIASMA_DURATION_MULT: f32 = 0.75;

/// T1-2 Lingering Fog: duration multiplier.
pub(super) const LINGERING_FOG_DURATION_MULT: f32 = 1.5;

/// T1-2 Lingering Fog: speed multiplier (reduced).
pub(super) const LINGERING_FOG_SPEED_MULT: f32 = 0.5;

// -- Tier 2 --

/// T2-0 Plague Carrier: fraction of tick damage applied as lingering DoT.
pub(super) const PLAGUE_CARRIER_DAMAGE_FRACTION: f32 = 0.5;

/// T2-0 Plague Carrier: duration of lingering DoT after leaving cloud (seconds).
pub(super) const PLAGUE_CARRIER_DURATION: f32 = 3.0;

/// T2-0 Plague Carrier: tick interval for lingering DoT (seconds).
pub(super) const PLAGUE_CARRIER_TICK_INTERVAL: f32 = 0.5;

/// T2-1 Toxic Weakness: extra damage taken multiplier while inside cloud.
pub(super) const TOXIC_WEAKNESS_VULNERABILITY: f32 = 0.25;

/// T2-2 Choking Gas: movement slow applied to units inside cloud.
pub(super) const CHOKING_GAS_SLOW: f32 = -0.4;

/// T2-2 Choking Gas: slow duration (refreshed each tick while in cloud).
pub(super) const CHOKING_GAS_SLOW_DURATION: f32 = 1.0;

// -- Tier 3 --

/// T3-0 Pandemic: duration of child clouds spawned on kill (seconds).
pub(super) const PANDEMIC_CHILD_DURATION: f32 = 8.0;

/// T3-0 Pandemic: radius multiplier for child clouds (relative to parent).
pub(super) const PANDEMIC_CHILD_RADIUS_MULT: f32 = 0.75;

/// T3-1 Twin Plumes: damage multiplier for each of the two clouds.
pub(crate) const TWIN_PLUMES_DAMAGE_MULT: f32 = 0.65;

/// T3-1 Twin Plumes: angular spread between the two clouds (radians).
pub(crate) const TWIN_PLUMES_ANGLE_SPREAD: f32 = std::f32::consts::FRAC_PI_4; // 45 degrees

/// T3-2 Necrotic Rot: fraction of poison damage that also reduces max health.
pub(super) const NECROTIC_ROT_MAX_HP_REDUCTION_FRACTION: f32 = 1.0;
