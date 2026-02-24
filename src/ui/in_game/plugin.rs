//! In-game UI plugin.

use bevy::prelude::*;

use crate::game::run_conditions::{is_gameplay_running, is_local_wizard_active};
use crate::state::AppState;
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
            .add_systems(
                OnEnter(AppState::MultiplayerGame),
                systems::spawn_mp_hud,
            )
            .add_systems(
                Update,
                systems::hud_button_action
                    .in_set(ButtonActionSet)
                    .run_if(is_local_wizard_active),
            )
            .add_systems(
                Update,
                systems::block_spell_input_on_button_interaction
                    .run_if(is_local_wizard_active),
            )
            .add_systems(
                Update,
                (
                    systems::keyboard_input,
                    systems::update_level_display,
                    systems::update_past_victory_display,
                )
                    .run_if(is_gameplay_running),
            )
            // Mana/cast bar: use is_local_wizard_active so guest can see their bars too
            .add_systems(
                Update,
                (systems::update_mana_bar, systems::update_cast_bar)
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
            );
    }
}
