//! Steam Input plugin registration.
//!
//! Registered only inside `SteamPlugin`'s `Ok` arm, so it is entirely absent when
//! the game runs without Steam — in which case the gilrs fallback path
//! (`game::input::gamepad`) covers controllers exactly as before.

use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::*;

use super::action_sets::activate_steam_action_sets;
use super::glyphs::refresh_steam_glyphs;
use super::handles::{SteamInputHandles, resolve_steam_input_handles};
use super::haptics::{
    SteamRumbleStop, apply_steam_rumble, open_binding_panel_on_click, tick_steam_rumble_stop,
};
use super::init::init_steam_input;
use super::poll::poll_steam_input;
use super::run_frame::steam_input_run_frame;
use crate::game::input::gamepad::messages::ControllerRumbleMessage;
use crate::game::input::gamepad::systems::{
    detect_active_input_device, fill_action_state_from_gilrs,
};
use crate::game::input::messages::MouseClicked;

pub(crate) struct SteamInputPlugin;

impl Plugin for SteamInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SteamInputHandles>();

        // Initialize Steam Input at startup. This captures supported controllers,
        // which is safe only because every action-state reader and the poll
        // producer below are already in place.
        app.add_systems(Startup, init_steam_input);

        // Controller read chain in PreUpdate, slotted between the device
        // coordinator and the gilrs fallback fill: pump → resolve handles →
        // activate the context's action set → poll into GamepadActionState +
        // claim SteamInputPad on activity. The gilrs adapter (which runs right
        // after) then yields whenever a Steam pad is active.
        app.add_systems(
            PreUpdate,
            (
                steam_input_run_frame,
                resolve_steam_input_handles,
                activate_steam_action_sets,
                poll_steam_input,
            )
                .chain()
                .after(detect_active_input_device)
                .before(fill_action_state_from_gilrs),
        );

        // Haptics (TriggerVibration FFI) + the "Configure Controller" binding
        // panel. Each only does work when there's something to react to.
        app.add_systems(
            Update,
            (
                // Only once the action manifest has loaded (handles resolved).
                refresh_steam_glyphs.run_if(|h: Res<SteamInputHandles>| h.resolved),
                apply_steam_rumble.run_if(on_message::<ControllerRumbleMessage>),
                tick_steam_rumble_stop.run_if(resource_exists::<SteamRumbleStop>),
                open_binding_panel_on_click.run_if(on_message::<MouseClicked>),
            ),
        );
    }
}
