use bevy::prelude::*;

use crate::game::run_conditions::{
    any_exist, is_gameplay_running, is_local_wizard_active, is_meteorologist,
    is_spell_effects_active,
};
use crate::state::{InGameState, MultiplayerGameState};

use super::components::*;
use super::messages::*;
use super::networking::{
    is_meteorologist_participating, is_remote_meteorologist, receive_weather_message,
    send_weather_state,
};
use super::resources::WeatherState;
use super::systems::*;

/// Plugin for the Meteorologist wizard archetype.
pub(in crate::game) struct MeteorologistPlugin;

impl Plugin for MeteorologistPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherState>()
            .add_message::<WeatherChangedMessage>()
            // Reset state + spawn overlays on entering gameplay. Gated on
            // `participating` so BOTH multiplayer peers set up the weather
            // visuals when EITHER player is the Meteorologist.
            .add_systems(
                OnEnter(InGameState::Running),
                (
                    reset_weather_state,
                    spawn_weather_overlays,
                    spawn_ground_overlay,
                )
                    .run_if(is_meteorologist_participating),
            )
            .add_systems(
                OnEnter(MultiplayerGameState::Running),
                (
                    reset_weather_state,
                    spawn_weather_overlays,
                    spawn_ground_overlay,
                )
                    .run_if(is_meteorologist_participating),
            )
            // Weather input (Q/W/E keys) — the LOCAL Meteorologist, on either
            // peer (`is_local_wizard_active`), so the guest can change weather.
            .add_systems(
                Update,
                handle_weather_input
                    .run_if(is_local_wizard_active)
                    .run_if(is_meteorologist),
            )
            // Replicate the weather CHOICE: the local Meteorologist sends it,
            // the opponent applies it.
            .add_systems(
                Update,
                send_weather_state
                    .run_if(is_local_wizard_active)
                    .run_if(is_meteorologist),
            )
            .add_systems(
                Update,
                receive_weather_message
                    .run_if(is_spell_effects_active)
                    .run_if(is_remote_meteorologist),
            )
            // Intensity ramp runs on BOTH peers (drives local VFX). Only mutates
            // the per-peer `WeatherState`, so each ramps from the same formula.
            .add_systems(
                Update,
                tick_weather_timers
                    .run_if(is_spell_effects_active)
                    .run_if(is_meteorologist_participating),
            )
            // SIMULATION — applies status to units / deals damage. HOST-only, and
            // runs whenever EITHER peer is the Meteorologist.
            .add_systems(
                Update,
                (apply_weather_status, update_weather_intensity)
                    .chain()
                    .run_if(is_gameplay_running)
                    .run_if(is_meteorologist_participating),
            )
            // Weather synergy systems (host-authoritative)
            .add_systems(
                Update,
                (
                    spread_shock_to_wet.run_if(any_exist::<WetModifier>()),
                    storm_lightning,
                )
                    .run_if(is_gameplay_running)
                    .run_if(is_meteorologist_participating),
            )
            // Burning patch gameplay (host-authoritative)
            .add_systems(
                Update,
                (update_burning_patches, update_burning_patch_visuals)
                    .run_if(is_gameplay_running)
                    .run_if(any_exist::<BurningPatch>()),
            )
            // Lightning visual cleanup (both peers, where strikes exist)
            .add_systems(
                Update,
                update_lightning_visuals.run_if(any_exist::<LightningStrike>()),
            )
            // Weather SFX — both peers (reacts to WeatherChangedMessage)
            .add_systems(
                Update,
                update_weather_sfx
                    .run_if(is_spell_effects_active)
                    .run_if(is_meteorologist_participating),
            )
            // Weather VFX systems — both peers, so the opponent sees rain/storm.
            .add_systems(
                Update,
                (
                    update_weather_overlay,
                    update_ground_overlay,
                    spawn_weather_particles,
                    update_weather_particles,
                )
                    .run_if(is_spell_effects_active)
                    .run_if(is_meteorologist_participating),
            )
            // Cleanup on exit (SP + MP)
            .add_systems(
                OnExit(InGameState::Running),
                cleanup_weather.run_if(is_meteorologist_participating),
            )
            .add_systems(
                OnExit(MultiplayerGameState::Running),
                cleanup_weather.run_if(is_meteorologist_participating),
            );
    }
}
