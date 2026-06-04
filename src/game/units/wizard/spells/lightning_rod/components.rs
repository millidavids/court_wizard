//! Lightning Rod spell components.

use bevy::prelude::*;

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
    /// T3-0 Storm Spire: turn the rod into a concentration (channeled) spell.
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

/// Snapshot-only marker on a `LightningBolt` parent for multiplayer serialization.
#[derive(Component)]
pub(crate) struct LightningRodArc {
    pub start: Vec3,
    pub end: Vec3,
}
