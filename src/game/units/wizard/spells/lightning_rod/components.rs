//! Lightning Rod spell components.

use bevy::prelude::*;

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
}

impl LightningRod {
    /// Creates a new lightning rod at the specified position.
    pub fn new(position: Vec3, duration: f32, empowerment: f32) -> Self {
        Self {
            position,
            time_alive: 0.0,
            // Start ready to strike immediately.
            time_since_strike: f32::MAX,
            duration,
            empowerment,
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
}

impl LightningRodCircleIndicator {
    /// Creates a new circle indicator.
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            time_alive: 0.0,
        }
    }

    /// Returns the current scale factor for pulse animation.
    pub fn pulse_scale(&self) -> f32 {
        let pulse_freq = 2.0;
        let pulse_amplitude = 0.05;
        1.0 + (self.time_alive * pulse_freq * std::f32::consts::TAU).sin() * pulse_amplitude
    }
}
