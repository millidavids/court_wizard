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
            .add_systems(OnExit(SplashState::Black), systems::cleanup_substate)
            .add_systems(OnExit(SplashState::Language), systems::cleanup_substate)
            .add_systems(OnExit(SplashState::Engine), systems::cleanup_substate)
            .add_systems(
                OnExit(AppState::Splash),
                (systems::cleanup_substate, systems::cleanup_assets),
            );
    }
}
