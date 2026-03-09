use bevy::prelude::*;

use super::constants;
use crate::game::units::wizard::spells::utils::{CircleIndicator, indicator_pulse_scale};

/// Marker component for units that received a Guardian Circle shield.
///
/// Tracks which talent effects are active on this shielded unit for
/// Tier 2 and Tier 3 talent reactions (retaliation, martyrdom, chain ward).
#[derive(Component, Clone)]
pub(crate) struct GuardianCircleShielded {
    /// T2-0: Retaliating Wards — burst damage when temp HP fully breaks.
    pub retaliating_damage: f32,
    /// T2-0: Retaliating Wards — burst radius.
    pub retaliating_radius: f32,
    /// T2-1: Fortified Resolve — bonus damage multiplier while shielded.
    pub fortified_damage_bonus: f32,
    /// T3-0: Sanctuary — damage reduction while shielded.
    pub sanctuary_reduction: f32,
    /// T3-1: Martyrdom — explosion damage on death (stored at grant time).
    pub martyrdom_damage: f32,
    /// T3-1: Martyrdom — explosion radius.
    pub martyrdom_radius: f32,
    /// T3-2: Chain Ward — remaining hop count.
    pub chain_ward_hops: u32,
    /// T3-2: Chain Ward — temp HP amount to pass along.
    pub chain_ward_amount: f32,
    /// T3-2: Chain Ward — temp HP duration to pass along.
    pub chain_ward_duration: f32,
}

impl Default for GuardianCircleShielded {
    fn default() -> Self {
        Self {
            retaliating_damage: 0.0,
            retaliating_radius: 0.0,
            fortified_damage_bonus: 0.0,
            sanctuary_reduction: 0.0,
            martyrdom_damage: 0.0,
            martyrdom_radius: 0.0,
            chain_ward_hops: 0,
            chain_ward_amount: 0.0,
            chain_ward_duration: 0.0,
        }
    }
}

/// Visual indicator for the Guardian Circle area during casting.
///
/// Shows the area of effect that will receive temporary hit points.
#[derive(Component)]
pub struct GuardianCircleIndicator {
    /// Position of the circle center.
    pub position: Vec3,
    /// Time this indicator has been active (for animations).
    pub time_alive: f32,
    /// Whether this circle is empowered.
    pub empowerment: f32,
    /// Talent-based radius multiplier (e.g. Expansive Aegis).
    pub talent_radius_mult: f32,
}

impl GuardianCircleIndicator {
    /// Creates a new circle indicator.
    pub const fn new(position: Vec3, empowerment: f32) -> Self {
        Self {
            position,
            time_alive: 0.0,
            empowerment,
            talent_radius_mult: 1.0,
        }
    }

    /// Returns the current scale factor for pulse animation.
    ///
    /// Pulsates between 0.95 and 1.05 during cast time.
    pub fn pulse_scale(&self) -> f32 {
        indicator_pulse_scale(self.time_alive)
    }
}

impl CircleIndicator for GuardianCircleIndicator {
    fn position(&self) -> Vec3 {
        self.position
    }
    fn time_alive(&self) -> f32 {
        self.time_alive
    }
    fn set_time_alive(&mut self, time: f32) {
        self.time_alive = time;
    }
    fn base_radius(&self) -> f32 {
        constants::CIRCLE_RADIUS * self.empowerment * self.talent_radius_mult
    }
    fn circle_y_position(&self) -> f32 {
        constants::CIRCLE_Y_POSITION
    }
    fn pulse_scale(&self) -> f32 {
        self.pulse_scale()
    }
}
