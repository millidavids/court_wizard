//! In-game UI plugin.

use bevy::prelude::*;

use crate::game::run_conditions::{
    is_gameplay_running, is_local_wizard_active, is_spell_effects_active,
};
use crate::state::{AppState, InGameState};
use crate::ui::plugin::ButtonActionSet;

use super::systems;

/// Plugin that manages in-game UI and input handling.
///
/// Registers systems for:
/// - HUD spawning on entering InGame state (SP) or MultiplayerGame (MP)
/// - Keyboard input during active gameplay (e.g., pause on Escape)
/// - Mana bar and cast bar updates
#[derive(Default)]
pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app
            // SP HUD spawn
            .add_systems(OnEnter(AppState::InGame), systems::spawn_hud)
            // MP HUD spawn (no cauldron, no level display).
            // Use AppState::MultiplayerGame so it spawns once, not on every
            // Running re-entry from SpellBook/Paused.
            .add_systems(OnEnter(AppState::MultiplayerGame), systems::spawn_mp_hud)
            .add_systems(
                Update,
                systems::hud_button_action
                    .in_set(ButtonActionSet)
                    .run_if(is_local_wizard_active),
            )
            .add_systems(
                Update,
                systems::block_spell_input_on_button_interaction.run_if(is_local_wizard_active),
            )
            .add_systems(
                Update,
                (
                    systems::keyboard_input,
                    systems::gamepad_hud_shortcuts,
                    systems::update_level_display,
                    systems::update_past_victory_display,
                    systems::update_level_clock,
                    systems::update_wave_display,
                    systems::update_wave_incoming_flash,
                    systems::spawn_retreat_flash,
                    systems::update_retreat_flash,
                )
                    .run_if(is_gameplay_running),
            )
            // Cast/overlay bars: use is_spell_effects_active so they update during
            // urgent mode menus (SP) and for both host+guest (MP).
            .add_systems(
                Update,
                (systems::update_cast_bar, systems::update_overlay_text)
                    .run_if(is_spell_effects_active),
            )
            // Mana/king/ammo bars: use is_local_wizard_active so guest can see their bars too
            .add_systems(
                Update,
                (
                    systems::update_mana_bar,
                    systems::update_king_health_bar,
                    systems::update_ammo_display,
                )
                    .run_if(is_local_wizard_active),
            )
            // Boss health bar: spawn when boss appears, update each frame
            .add_systems(
                Update,
                (
                    systems::spawn_boss_health_bar,
                    systems::update_boss_health_bar,
                )
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                systems::update_ray_eye_health_bar
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<crate::game::units::boss::ray::RayEye>),
            )
            // Buff tracker: SP only (CauldronBuffs only exists in SP)
            .add_systems(
                Update,
                (
                    systems::update_buff_tracker,
                    systems::update_buff_timers,
                    systems::show_buff_tooltip,
                )
                    .run_if(in_state(InGameState::Running)),
            );
    }
}
