//! Plugin that registers gamepad input translation on top of the existing
//! mouse/keyboard pipeline.

use bevy::ecs::schedule::common_conditions::on_message;
use bevy::input::gamepad::GamepadConnectionEvent;
use bevy::prelude::*;

use super::action_translation::{
    translate_activate_button, translate_arcanorouter, translate_roulette, translate_runes,
    translate_warglock, translate_weather,
};
use super::messages::{ControllerRumbleMessage, GamepadConfirmPressed, MenuBackPressed};
use super::resources::{
    ActiveInputDevice, GamepadAimSettings, RadialHoveredSlot, VirtualCursorPosition,
};
use super::rumble::rumble_on_score_screen;
use super::run_conditions::gamepad_active;
use super::systems::{
    LastActiveGamepad, detect_active_input_device, emit_ui_confirm_back_messages,
    fill_action_state_from_gilrs, pause_on_controller_unplug, sync_gamepad_settings,
    toggle_cursor_visibility, translate_triggers_to_mouse_messages, update_radial_hovered_slot,
    update_virtual_cursor,
};
use crate::game::run_conditions::{
    is_arcanorouter, is_local_wizard_active, is_meteorologist, is_randomancer, is_rune_caster,
    is_warglock,
};
use crate::state::InGameState;

pub(crate) struct GamepadInputPlugin;

impl Plugin for GamepadInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveInputDevice>()
            .init_resource::<super::arbiter::InputDeviceArbiter>()
            .init_resource::<LastActiveGamepad>()
            .init_resource::<VirtualCursorPosition>()
            .init_resource::<GamepadAimSettings>()
            .init_resource::<RadialHoveredSlot>()
            .add_message::<MenuBackPressed>()
            .add_message::<GamepadConfirmPressed>()
            .add_message::<ControllerRumbleMessage>()
            // Device detection, fallback action-state fill, and cursor toggle in
            // PreUpdate. `fill_action_state_from_gilrs` runs after the device
            // coordinator so it sees the current active device, and before any
            // `Update` consumer of `GamepadActionState`.
            .add_systems(
                PreUpdate,
                (
                    detect_active_input_device,
                    fill_action_state_from_gilrs,
                    toggle_cursor_visibility,
                )
                    .chain(),
            )
            // Virtual cursor runs AFTER barrel correction so it overrides that output
            // when gamepad is active.
            .add_systems(
                PreUpdate,
                update_virtual_cursor
                    .after(crate::game::crt_effect::correct_cursor_for_barrel_distortion)
                    .after(toggle_cursor_visibility),
            )
            // Core gameplay translation: triggers → mouse messages, stick → radial slot.
            .add_systems(
                Update,
                (
                    translate_triggers_to_mouse_messages,
                    update_radial_hovered_slot,
                )
                    .run_if(is_local_wizard_active),
            )
            // Pause + restore mouse/keyboard when the active controller unplugs.
            // Only runs when a connection event is actually waiting.
            .add_systems(
                Update,
                pause_on_controller_unplug.run_if(on_message::<GamepadConnectionEvent>),
            )
            // UI confirm/back runs only when gamepad is active.
            .add_systems(Update, emit_ui_confirm_back_messages.run_if(gamepad_active))
            // Sync user-facing GameConfig settings into GamepadAimSettings.
            .add_systems(
                Update,
                sync_gamepad_settings.run_if(resource_changed::<crate::config::GameConfig>),
            )
            // Universal "Activate" button (A = Space analog) runs when wizard is controllable.
            .add_systems(
                Update,
                translate_activate_button.run_if(is_local_wizard_active),
            )
            // Archetype-specific translation gated on the active archetype.
            .add_systems(
                Update,
                translate_runes
                    .run_if(is_local_wizard_active)
                    .run_if(is_rune_caster),
            )
            .add_systems(
                Update,
                translate_roulette
                    .run_if(is_local_wizard_active)
                    .run_if(is_randomancer),
            )
            .add_systems(
                Update,
                translate_weather
                    .run_if(is_local_wizard_active)
                    .run_if(is_meteorologist),
            )
            .add_systems(
                Update,
                translate_warglock
                    .run_if(is_local_wizard_active)
                    .run_if(is_warglock),
            )
            .add_systems(
                Update,
                translate_arcanorouter
                    .run_if(is_local_wizard_active)
                    .run_if(is_arcanorouter),
            )
            .add_systems(OnEnter(InGameState::ScoreScreen), rumble_on_score_screen);
    }
}
