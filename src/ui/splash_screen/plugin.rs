use bevy::prelude::*;

use crate::state::AppState;

use super::systems;

pub struct SplashScreenPlugin;

impl Plugin for SplashScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Splash), systems::setup)
            .add_systems(Update, systems::tick.run_if(in_state(AppState::Splash)))
            .add_systems(OnExit(AppState::Splash), systems::cleanup);
    }
}
