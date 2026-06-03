use bevy::prelude::*;

use super::components::WeatherBarRoot;
use super::systems;
use crate::game::run_conditions::{any_exist, is_local_wizard_active, is_meteorologist};
use crate::state::{InGameState, MultiplayerGameState};

/// Plugin for the weather bar UI displayed when playing as Meteorologist.
pub struct WeatherBarPlugin;

impl Plugin for WeatherBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameState::Running),
            systems::spawn_weather_bar.run_if(is_meteorologist),
        )
        .add_systems(OnExit(InGameState::Running), systems::cleanup_weather_bar)
        // Multiplayer: the weather bar belongs to the local Meteorologist on
        // either peer. `OnEnter(MultiplayerGame)` syncs `config.wizard_type`
        // before this substate's `OnEnter`, so `is_meteorologist` is accurate.
        .add_systems(
            OnEnter(MultiplayerGameState::Running),
            systems::spawn_weather_bar.run_if(is_meteorologist),
        )
        .add_systems(
            OnExit(MultiplayerGameState::Running),
            systems::cleanup_weather_bar,
        )
        // `is_local_wizard_active` (not `is_gameplay_running`) so the GUEST
        // Meteorologist's button clicks register too.
        .add_systems(
            Update,
            (
                systems::update_weather_buttons,
                systems::handle_weather_button_click,
            )
                .run_if(is_local_wizard_active)
                .run_if(is_meteorologist)
                .run_if(any_exist::<WeatherBarRoot>()),
        );
    }
}
