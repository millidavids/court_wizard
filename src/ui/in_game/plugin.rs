//! In-game UI plugin.

use bevy::prelude::*;

use crate::game::run_conditions::{
    is_gameplay_running, is_local_wizard_active, is_spell_effects_active,
};
use crate::state::{AppState, InGameState};
use crate::ui::plugin::ButtonActionSet;

use super::components::{RetreatFlash, ShieldFellFlash};
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
                    systems::update_level_display,
                    systems::update_past_victory_display,
                    systems::spawn_retreat_flash,
                    systems::update_timed_flash::<RetreatFlash>,
                )
                    .run_if(is_gameplay_running),
            )
            // Gamepad HUD shortcuts (X = spell book, Y = cauldron) must run for
            // BOTH peers in MP — the guest opens its spell book too — so gate on
            // the local wizard being active, not host-only `is_gameplay_running`.
            .add_systems(
                Update,
                systems::gamepad_hud_shortcuts.run_if(is_local_wizard_active),
            )
            // The level/match clock updates on BOTH peers in MP (and in SP) via
            // is_spell_effects_active, so the guest's HUD clock advances too — not
            // just the host. SP behavior is unchanged (same states as is_gameplay_running).
            .add_systems(
                Update,
                systems::update_level_clock.run_if(is_spell_effects_active),
            )
            // Wave HUD: SP-only — the wave timer doesn't tick in MP
            // (MP spawns all attackers up-front, no staging waves) and the
            // MP HUD doesn't include a WaveDisplay node anyway.
            .add_systems(
                Update,
                (
                    systems::update_wave_display,
                    systems::update_wave_incoming_flash,
                )
                    .run_if(is_gameplay_running.and(in_state(AppState::InGame))),
            )
            // Cast/overlay bars: use is_spell_effects_active so they update during
            // urgent mode menus (SP) and for both host+guest (MP).
            .add_systems(
                Update,
                (systems::update_cast_bar, systems::update_overlay_text)
                    .run_if(is_spell_effects_active),
            )
            // "King's shield has fallen!" banner: is_spell_effects_active so it
            // shows for BOTH host and guest in MP. No-op in SP (SP kings never
            // carry SpellShield, so the RemovedComponents watcher never fires).
            .add_systems(
                Update,
                (
                    systems::spawn_shield_fell_flash,
                    systems::update_timed_flash::<ShieldFellFlash>,
                )
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
