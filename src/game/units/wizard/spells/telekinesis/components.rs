use bevy::prelude::*;

/// Visual indicator ring around a targeted ingredient drop during Telekinesis casting.
#[derive(Component)]
pub(super) struct TelekinesisIndicator {
    /// Entity of the drop being targeted.
    pub target_drop: Entity,
    /// Time this indicator has been alive (for pulse animation).
    pub time_alive: f32,
}

impl TelekinesisIndicator {
    pub const fn new(target_drop: Entity) -> Self {
        Self {
            target_drop,
            time_alive: 0.0,
        }
    }

    /// Returns the current scale factor for pulse animation.
    pub fn pulse_scale(&self) -> f32 {
        let pulse_freq = 3.0;
        let pulse_amplitude = 0.1;
        1.0 + (self.time_alive * pulse_freq * std::f32::consts::TAU).sin() * pulse_amplitude
    }
}
