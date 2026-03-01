use bevy::prelude::*;

use crate::state::SplashState;

/// Marker component for splash screen entities scoped to the current substate.
#[derive(Component)]
pub(super) struct SplashEntity;

/// What the splash timer transitions to when finished.
#[derive(Component)]
pub(super) enum SplashTransition {
    /// Move to the next splash substate.
    NextSplash(SplashState),
    /// Exit the splash entirely and go to main menu.
    MainMenu,
}

/// Preloaded image handles for all splash screens.
///
/// Inserted during the Black substate so assets begin loading immediately
/// and are ready by the time each splash screen is displayed.
#[derive(Resource)]
pub(super) struct SplashAssets {
    pub rust_logo: Handle<Image>,
    pub bevy_logo: Handle<Image>,
    pub studio_image: Handle<Image>,
    /// CRT shader handle — Black substate waits until this is loaded.
    pub shader: Handle<Shader>,
}

/// Tracks how long a splash screen has been displayed.
#[derive(Component)]
pub(super) struct SplashTimer {
    pub elapsed: f32,
    pub duration: f32,
}

impl SplashTimer {
    pub fn new(duration: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}
