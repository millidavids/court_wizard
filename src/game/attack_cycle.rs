//! The global attack-cycle timer that staggers unit attacks.

use bevy::prelude::*;

use super::constants::ATTACK_CYCLE_DURATION;

/// Global attack cycle timer resource.
///
/// Cycles from 0.0 to CYCLE_DURATION seconds. Units track which time offset
/// in the cycle they last attacked and can only attack again when the timer
/// cycles back to that offset. This naturally staggers attacks across all units.
#[derive(Resource)]
pub struct GlobalAttackCycle {
    /// Current time in the cycle (0.0 to CYCLE_DURATION)
    pub current_time: f32,
    /// Cycle time BEFORE the most recent `tick(delta)` call. `combat()`
    /// uses this as the `last_time` parameter to `can_attack` so the
    /// "did the cycle sweep past this unit's slot in the last frame?"
    /// window is exactly the time actually elapsed — not a constant
    /// `APPROX_FRAME_TIME` approximation that re-fires under fast frames
    /// (frame_delta < 0.016) and skips slots under slow frames
    /// (frame_delta > 0.016).
    pub previous_time: f32,
    /// Duration of one complete cycle in seconds
    pub cycle_duration: f32,
}

impl Default for GlobalAttackCycle {
    fn default() -> Self {
        Self {
            current_time: 0.0,
            previous_time: 0.0,
            cycle_duration: ATTACK_CYCLE_DURATION,
        }
    }
}

impl GlobalAttackCycle {
    /// Advances the cycle timer by delta time, wrapping back to 0 after cycle_duration.
    pub fn tick(&mut self, delta: f32) {
        self.previous_time = self.current_time;
        self.current_time = (self.current_time + delta) % self.cycle_duration;
    }
}
