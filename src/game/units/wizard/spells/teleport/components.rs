//! Components for the Teleport spell.

use bevy::prelude::*;

/// Marker component indicating the wizard is actively managing Teleport spell state.
///
/// Tracks the destination circle entity and whether we're in phase 1 or 2.
#[derive(Component)]
pub struct TeleportCaster {
    /// Entity ID of the destination circle (None if no destination placed).
    pub destination_circle: Option<Entity>,
    /// Position of the destination circle.
    pub destination_position: Option<Vec3>,
    /// Entity ID of the source circle during second cast (None otherwise).
    pub source_circle: Option<Entity>,
}

impl TeleportCaster {
    /// Creates a new TeleportCaster with no circles placed.
    pub const fn new() -> Self {
        Self {
            destination_circle: None,
            destination_position: None,
            source_circle: None,
        }
    }

    /// Returns true if a destination has been placed.
    pub const fn has_destination(&self) -> bool {
        self.destination_position.is_some()
    }
}

/// Visual indicator for the destination circle (persists between casts).
#[derive(Component)]
pub struct TeleportDestinationCircle {
    /// Time this indicator has been active (for animations).
    pub time_alive: f32,
}

impl TeleportDestinationCircle {
    /// Creates a new destination circle indicator.
    pub const fn new() -> Self {
        Self { time_alive: 0.0 }
    }

    /// Returns the current scale factor for pulse animation.
    ///
    /// Pulsates between 0.95 and 1.05.
    pub fn pulse_scale(&self) -> f32 {
        let pulse_freq = 2.0; // Hz
        let pulse_amplitude = 0.05;
        1.0 + (self.time_alive * pulse_freq * std::f32::consts::TAU).sin() * pulse_amplitude
    }
}

/// Visual indicator for the source circle (only during second cast).
#[derive(Component)]
pub struct TeleportSourceCircle {
    /// Position of the circle center.
    pub position: Vec3,
    /// Time this indicator has been active (for animations).
    pub time_alive: f32,
}

impl TeleportSourceCircle {
    /// Creates a new source circle indicator.
    pub const fn new(position: Vec3) -> Self {
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
