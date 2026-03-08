use bevy::prelude::*;

use super::components::WeatherBarRoot;
use super::systems;
use crate::game::run_conditions::{any_exist, is_gameplay_running, is_meteorologist};
use crate::state::InGameState;

/// Plugin for the weather bar UI displayed when playing as Meteorologist.
pub struct WeatherBarPlugin;

impl Plugin for WeatherBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameState::Running),
            systems::spawn_weather_bar.run_if(is_meteorologist),
        )
        .add_systems(
            OnExit(InGameState::Running),
            systems::cleanup_weather_bar,
        )
        .add_systems(
            Update,
            (
                systems::update_weather_buttons,
                systems::handle_weather_button_click,
            )
                .run_if(is_gameplay_running)
                .run_if(is_meteorologist)
                .run_if(any_exist::<WeatherBarRoot>()),
        );
    }
}
