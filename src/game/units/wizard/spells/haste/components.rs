//! Haste spell components.

use bevy::prelude::*;

use crate::game::units::components::TimedModifier;

/// Talent parameters computed at cast time from active talent selections.
pub(crate) struct HasteTalentParams {
    // Tier 1: numeric modifiers
    /// Alacrity: multiplier on speed bonus.
    pub speed_mult: f32,
    /// Extended Rush: multiplier on duration.
    pub duration_mult: f32,
    /// Quick Cast: multiplier on cast time.
    pub cast_time_mult: f32,
    // Tier 2: behavioral flags
    /// Adrenaline Surge: grant attack speed bonus.
    pub adrenaline_surge: bool,
    /// Momentum: grant post-expiry damage bonus.
    pub momentum: bool,
    /// Fleet Feet: grant dodge on first attack.
    pub fleet_feet: bool,
    // Tier 3: transformative flags
    /// Time Warp: double speed+attack bonuses, halve duration.
    pub time_warp: bool,
    /// Slow Zone: leave a slow field at cast position.
    pub slow_zone: bool,
    /// Chain Haste: buff jumps to nearest un-hasted ally on expiry.
    pub chain_haste: bool,
}

impl Default for HasteTalentParams {
    fn default() -> Self {
        Self {
            speed_mult: 1.0,
            duration_mult: 1.0,
            cast_time_mult: 1.0,
            adrenaline_surge: false,
            momentum: false,
            fleet_feet: false,
            time_warp: false,
            slow_zone: false,
            chain_haste: false,
        }
    }
}

/// Tier 2: Momentum — post-expiry damage bonus applied when HasteModifier expires.
#[derive(Component)]
pub(crate) struct MomentumBuff {
    /// Damage multiplier bonus (e.g., 0.25 = +25%).
    pub damage_mult: f32,
    /// Time remaining before the momentum bonus expires.
    pub time_remaining: f32,
}

impl MomentumBuff {
    pub const fn new(damage_mult: f32, duration: f32) -> Self {
        Self {
            damage_mult,
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

impl TimedModifier for MomentumBuff {
    fn tick(&mut self, delta: f32) -> bool {
        self.update(delta)
    }
}

/// Tier 2: Fleet Feet — dodge the first attack while hasted.
#[derive(Component)]
pub(crate) struct FleetFeet {
    /// Number of dodges remaining.
    pub dodges_remaining: u32,
}

impl FleetFeet {
    pub const fn new(dodges: u32) -> Self {
        Self {
            dodges_remaining: dodges,
        }
    }
}

/// Tier 3: Slow Zone — persistent ground effect that slows enemies.
#[derive(Component)]
pub(crate) struct HasteSlowZone {
    /// Center position of the slow zone.
    pub position: Vec3,
    /// Radius of the slow zone.
    pub radius: f32,
    /// Time remaining before the zone expires.
    pub time_remaining: f32,
    /// Slow amount (negative value, e.g., -0.3 = -30% speed).
    pub slow_amount: f32,
}

/// Tier 2: Momentum pending — marker inserted at cast time so we know to apply
/// MomentumBuff when HasteModifier expires.
#[derive(Component)]
pub(crate) struct MomentumPending;

/// Tier 3: Chain Haste — tracks remaining hops on a hasted unit.
/// When the HasteModifier expires, the buff jumps to the nearest un-hasted ally.
#[derive(Component)]
pub(crate) struct ChainHasteSource {
    /// Number of hops remaining.
    pub hops_remaining: u32,
    /// Current effectiveness multiplier (decreases per hop).
    pub effectiveness: f32,
    /// The attack_speed value to carry forward.
    pub attack_speed: f32,
}
