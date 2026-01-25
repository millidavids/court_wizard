use bevy::prelude::*;

/// Marker component indicating the wizard is actively casting Guardian Circle.
///
/// Used to track the casting visual entity and differentiate from other spells.
/// The circle_entity is None after cast completes but before mouse release.
#[derive(Component)]
pub struct GuardianCircleCaster {
    /// Entity ID of the visual circle indicator (None if despawned).
    pub circle_entity: Option<Entity>,
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
}

impl GuardianCircleIndicator {
    /// Creates a new circle indicator.
    pub const fn new(position: Vec3) -> Self {
        Self {
            position,
            time_alive: 0.0,
        }
    }

    /// Returns the current scale factor for pulse animation.
    ///
    /// Pulsates between 0.95 and 1.05 during cast time.
    pub fn pulse_scale(&self) -> f32 {
        let pulse_freq = 2.0; // Hz
        let pulse_amplitude = 0.05;
        1.0 + (self.time_alive * pulse_freq * std::f32::consts::TAU).sin() * pulse_amplitude
    }
}
