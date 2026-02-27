use bevy::prelude::*;

/// Marker component for all splash screen entities (for cleanup on exit).
#[derive(Component)]
pub(super) struct OnSplashScreen;

/// Marker for the splash background image node.
#[derive(Component)]
pub(super) struct SplashImage;

/// Tracks the four phases of the splash screen animation: delay, fade-in, hold, fade-out.
#[derive(Component)]
pub(super) struct SplashTimer {
    pub elapsed: f32,
    pub delay: f32,
    pub fade_in: f32,
    pub hold: f32,
    pub fade_out: f32,
}

impl SplashTimer {
    pub fn new(delay: f32, fade_in: f32, hold: f32, fade_out: f32) -> Self {
        Self {
            elapsed: 0.0,
            delay,
            fade_in,
            hold,
            fade_out,
        }
    }

    /// Total duration of the entire splash sequence.
    fn total_duration(&self) -> f32 {
        self.delay + self.fade_in + self.hold + self.fade_out
    }

    /// Returns the current opacity based on elapsed time.
    pub fn opacity(&self) -> f32 {
        if self.elapsed <= self.delay {
            // Delay: nothing visible yet
            0.0
        } else if self.elapsed <= self.delay + self.fade_in {
            // Fade in: 0 → 1
            let fade_elapsed = self.elapsed - self.delay;
            (fade_elapsed / self.fade_in).clamp(0.0, 1.0)
        } else if self.elapsed <= self.delay + self.fade_in + self.hold {
            // Hold: fully visible
            1.0
        } else {
            // Fade out: 1 → 0
            let fade_out_elapsed = self.elapsed - self.delay - self.fade_in - self.hold;
            (1.0 - fade_out_elapsed / self.fade_out).clamp(0.0, 1.0)
        }
    }

    /// Returns true when the entire sequence is complete.
    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.total_duration()
    }
}
