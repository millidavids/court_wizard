use bevy::prelude::*;

use crate::state::{AppState, SplashState};

use super::systems;

pub struct SplashScreenPlugin;

impl Plugin for SplashScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SplashState::Black), systems::setup_black)
            .add_systems(OnEnter(SplashState::Language), systems::setup_language)
            .add_systems(OnEnter(SplashState::Engine), systems::setup_engine)
            .add_systems(OnEnter(SplashState::Studio), systems::setup_studio)
            // Black substate: wait for shader to load, then skip or proceed
            .add_systems(
                Update,
                systems::tick_black
                    .run_if(in_state(SplashState::Black)),
            )
            // Language/Engine/Studio substates: timer-driven transitions
            .add_systems(
                Update,
                systems::tick
                    .run_if(in_state(AppState::Splash))
                    .run_if(not(in_state(SplashState::Black))),
            )
            .add_systems(OnExit(SplashState::Black), crate::ui::systems::cleanup_screen::<super::components::SplashEntity>)
            .add_systems(OnExit(SplashState::Language), crate::ui::systems::cleanup_screen::<super::components::SplashEntity>)
            .add_systems(OnExit(SplashState::Engine), crate::ui::systems::cleanup_screen::<super::components::SplashEntity>)
            .add_systems(
                OnExit(AppState::Splash),
                (crate::ui::systems::cleanup_screen::<super::components::SplashEntity>, systems::cleanup_assets),
            );
    }
}
