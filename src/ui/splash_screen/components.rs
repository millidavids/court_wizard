use bevy::prelude::*;

use crate::state::SplashState;

/// Marker component for splash screen entities scoped to the current substate.
#[derive(Component)]
pub(super) struct SplashEntity;

/// Marker for elements that fade at full opacity (logos, text).
#[derive(Component)]
pub(super) struct SplashFadeImage;

/// Marker for the background color node that fades (e.g. gray circle).
#[derive(Component)]
pub(super) struct SplashFadeBackground {
    pub color: Color,
}

/// Marker for text elements that fade with a specific color.
#[derive(Component)]
pub(super) struct SplashFadeText {
    pub color: Color,
}

/// Marker for images that fade to a capped max opacity (e.g. studio watermark).
#[derive(Component)]
pub(super) struct SplashFadeImageCapped {
    pub max_opacity: f32,
}

/// What the splash timer transitions to when finished.
#[derive(Component)]
pub(super) enum SplashTransition {
    /// Move to the next splash substate.
    NextSplash(SplashState),
    /// Exit the splash entirely and go to main menu.
    MainMenu,
}

/// Tracks the four phases of a splash screen animation: delay, fade-in, hold, fade-out.
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
            0.0
        } else if self.elapsed <= self.delay + self.fade_in {
            let fade_elapsed = self.elapsed - self.delay;
            (fade_elapsed / self.fade_in).clamp(0.0, 1.0)
        } else if self.elapsed <= self.delay + self.fade_in + self.hold {
            1.0
        } else {
            let fade_out_elapsed = self.elapsed - self.delay - self.fade_in - self.hold;
            (1.0 - fade_out_elapsed / self.fade_out).clamp(0.0, 1.0)
        }
    }

    /// Returns true when the entire sequence is complete.
    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.total_duration()
    }
}
