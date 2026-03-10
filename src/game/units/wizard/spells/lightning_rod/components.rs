//! Lightning Rod spell components.

use bevy::prelude::*;

use super::constants::{ARC_RADIUS, CIRCLE_Y_POSITION};
use crate::game::units::wizard::spells::utils::CircleIndicator;

/// Pre-computed talent parameters for a lightning rod instance.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LightningRodTalentParams {
    /// Combined duration multiplier (T1-0 Taller Rod, T3-0 Storm Spire).
    pub duration_mult: f32,
    /// Multiplier applied to strike interval (T1-1 Rapid Strikes).
    pub strike_interval_mult: f32,
    /// Multiplier applied to arc radius (T1-2 Wider Arc).
    pub arc_radius_mult: f32,
    /// Extra targets per strike (T1-2 Wider Arc).
    pub extra_targets: usize,
    /// Multiplier applied to arc damage (T3-0 Storm Spire).
    pub damage_mult: f32,
    /// T2-0 Chain Reaction: chain to extra targets.
    pub chain_reaction: bool,
    /// T2-1 Magnetic Field: slow enemies hit by arcs.
    pub magnetic_field: bool,
    /// T2-2 Overcharge: every Nth strike is empowered.
    pub overcharge: bool,
    /// T3-0 Storm Spire: place 2 rods instead of 1.
    pub storm_spire: bool,
    /// T3-1 Tesla Coil: ramp damage per strike.
    pub tesla_coil: bool,
    /// T3-2 Lightning Nexus: kills trigger bonus strikes.
    pub lightning_nexus: bool,
}

impl Default for LightningRodTalentParams {
    fn default() -> Self {
        Self {
            duration_mult: 1.0,
            strike_interval_mult: 1.0,
            arc_radius_mult: 1.0,
            extra_targets: 0,
            damage_mult: 1.0,
            chain_reaction: false,
            magnetic_field: false,
            overcharge: false,
            storm_spire: false,
            tesla_coil: false,
            lightning_nexus: false,
        }
    }
}

/// Lightning rod tower placed on the battlefield.
///
/// Periodically attracts lightning strikes that arc to nearby units.
#[derive(Component)]
pub(crate) struct LightningRod {
    /// Center position of the rod in world space.
    pub position: Vec3,
    /// Time since the rod was placed (seconds).
    pub time_alive: f32,
    /// Time since the last lightning strike (seconds).
    pub time_since_strike: f32,
    /// Total lifetime before despawn (seconds).
    pub duration: f32,
    /// Empowerment multiplier for spell effectiveness.
    pub empowerment: f32,
    /// Number of strikes this rod has fired (for Overcharge / Tesla Coil).
    pub strike_count: u32,
    /// Cumulative damage ramp from Tesla Coil (additive, e.g. 0.30 = +30%).
    pub damage_ramp: f32,
    /// Talent parameters for this rod instance.
    pub talent_params: LightningRodTalentParams,
}

impl LightningRod {
    /// Creates a new lightning rod at the specified position.
    pub fn new(
        position: Vec3,
        duration: f32,
        empowerment: f32,
        talent_params: LightningRodTalentParams,
    ) -> Self {
        Self {
            position,
            time_alive: 0.0,
            // Start ready to strike immediately.
            time_since_strike: f32::MAX,
            duration,
            empowerment,
            strike_count: 0,
            damage_ramp: 0.0,
            talent_params,
        }
    }

    /// Returns true if the rod has expired.
    pub fn is_expired(&self) -> bool {
        self.time_alive >= self.duration
    }
}

/// Descending lightning bolt heading toward the rod.
#[derive(Component)]
pub(crate) struct LightningStrike {
    /// Position the bolt is heading toward (top of the rod).
    pub target_pos: Vec3,
    /// Downward speed (units/second).
    pub speed: f32,
    /// Damage dealt by arcs when the bolt reaches the rod.
    pub arc_damage: f32,
    /// Radius to search for arc targets.
    pub arc_radius: f32,
    /// Empowerment multiplier.
    pub empowerment: f32,
    /// Maximum targets this strike can hit.
    pub max_targets: usize,
    /// Lightning Nexus damage multiplier (compounds per bonus generation, e.g. 1.0 → 0.5 → 0.25).
    pub nexus_damage_mult: f32,
    /// Talent parameters inherited from the rod.
    pub talent_params: LightningRodTalentParams,
}

/// Visual lightning arc between the rod and a hit target.
#[derive(Component)]
pub(crate) struct LightningRodArc {
    /// Time remaining before arc despawns (seconds).
    pub lifetime: f32,
    /// Time since arc was created (for animation).
    pub time_alive: f32,
}

impl LightningRodArc {
    /// Creates a new arc visual.
    pub fn new(lifetime: f32) -> Self {
        Self {
            lifetime,
            time_alive: 0.0,
        }
    }
}

/// Circle indicator shown during casting to preview the arc radius.
#[derive(Component)]
pub(super) struct LightningRodCircleIndicator {
    /// Position of the circle center.
    pub position: Vec3,
    /// Time this indicator has been active (for animations).
    pub time_alive: f32,
    /// Empowerment multiplier (for scaling unit-sized mesh).
    pub empowerment: f32,
}

impl LightningRodCircleIndicator {
    /// Creates a new circle indicator.
    pub fn new(position: Vec3, empowerment: f32) -> Self {
        Self {
            position,
            time_alive: 0.0,
            empowerment,
        }
    }

    /// Returns the current scale factor for pulse animation.
    pub fn pulse_scale(&self) -> f32 {
        let pulse_freq = 2.0;
        let pulse_amplitude = 0.05;
        1.0 + (self.time_alive * pulse_freq * std::f32::consts::TAU).sin() * pulse_amplitude
    }
}

impl CircleIndicator for LightningRodCircleIndicator {
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
        ARC_RADIUS * self.empowerment
    }
    fn circle_y_position(&self) -> f32 {
        CIRCLE_Y_POSITION
    }
    fn pulse_scale(&self) -> f32 {
        self.pulse_scale()
    }
}
