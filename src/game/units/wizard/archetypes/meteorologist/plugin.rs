use bevy::prelude::*;

use crate::game::run_conditions::{any_exist, is_gameplay_running, is_meteorologist};
use crate::state::InGameState;

use super::components::*;
use super::messages::*;
use super::resources::WeatherState;
use super::systems::*;

/// Plugin for the Meteorologist wizard archetype.
pub(in crate::game) struct MeteorologistPlugin;

impl Plugin for MeteorologistPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherState>()
            .add_message::<WeatherChangedMessage>()
            // Reset state on entering gameplay
            .add_systems(
                OnEnter(InGameState::Running),
                (
                    reset_weather_state,
                    spawn_weather_overlays,
                    spawn_ground_overlay,
                )
                    .run_if(is_meteorologist),
            )
            // Weather input (Q/W/E keys)
            .add_systems(
                Update,
                handle_weather_input
                    .run_if(is_gameplay_running)
                    .run_if(is_meteorologist),
            )
            // Core weather systems
            .add_systems(
                Update,
                (
                    tick_weather_timers,
                    apply_weather_status,
                    update_weather_intensity,
                )
                    .chain()
                    .run_if(is_gameplay_running)
                    .run_if(is_meteorologist),
            )
            // Weather synergy systems
            .add_systems(
                Update,
                (
                    spread_shock_to_wet.run_if(any_exist::<WetModifier>()),
                    storm_lightning,
                )
                    .run_if(is_gameplay_running)
                    .run_if(is_meteorologist),
            )
            // Burning patch systems
            .add_systems(
                Update,
                (
                    update_burning_patches,
                    update_burning_patch_visuals,
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_exist::<BurningPatch>()),
            )
            // Lightning visual cleanup
            .add_systems(
                Update,
                update_lightning_visuals.run_if(any_exist::<LightningStrike>()),
            )
            // Weather SFX (reacts to WeatherChangedMessage)
            .add_systems(
                Update,
                update_weather_sfx
                    .run_if(is_gameplay_running)
                    .run_if(is_meteorologist),
            )
            // Weather VFX systems
            .add_systems(
                Update,
                (
                    update_weather_overlay,
                    update_ground_overlay,
                    spawn_weather_particles,
                    update_weather_particles,
                )
                    .run_if(is_gameplay_running)
                    .run_if(is_meteorologist),
            )
            // Cleanup on exit
            .add_systems(
                OnExit(InGameState::Running),
                cleanup_weather.run_if(is_meteorologist),
            );
    }
}
