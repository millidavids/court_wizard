//! Plugin that registers gamepad input translation on top of the existing
//! mouse/keyboard pipeline.

use bevy::prelude::*;

use super::action_translation::{
    translate_activate_button, translate_arcanorouter, translate_roulette, translate_runes,
    translate_warglock, translate_weather,
};
use super::messages::{GamepadConfirmPressed, MenuBackPressed};
use super::resources::{
    ActiveInputDevice, GamepadAimSettings, RadialHoveredSlot, VirtualCursorPosition,
};
use super::rumble::rumble_on_score_screen;
use super::systems::{
    detect_active_input_device, emit_ui_confirm_back_messages, sync_gamepad_settings,
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
            .init_resource::<VirtualCursorPosition>()
            .init_resource::<GamepadAimSettings>()
            .init_resource::<RadialHoveredSlot>()
            .add_message::<MenuBackPressed>()
            .add_message::<GamepadConfirmPressed>()
            // Device detection and cursor toggle in PreUpdate.
            .add_systems(
                PreUpdate,
                (detect_active_input_device, toggle_cursor_visibility).chain(),
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
            // UI confirm/back runs always when gamepad is active.
            .add_systems(Update, emit_ui_confirm_back_messages)
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
