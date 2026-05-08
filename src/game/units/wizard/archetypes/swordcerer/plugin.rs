use bevy::prelude::*;

use crate::game::run_conditions::{
    any_exist, is_gameplay_running, is_spell_effects_active, is_swordcerer,
};
use crate::state::{AppState, InGameState};

use super::components::*;
use super::messages::*;
use super::systems::*;

/// Plugin for the Swordcerer wizard archetype.
pub(in crate::game) struct SwordcererPlugin;

impl Plugin for SwordcererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, super::resources::preload_swordcerer_assets)
            .add_message::<RetreatMessage>()
            // Initialize state and spawn Enter the Fray button when the battle
            // begins. Must be `AppState::InGame`, NOT `InGameState::Running`:
            // opening the spell book, pause, or cauldron menu transitions out
            // of `Running` and back, which would otherwise reset the
            // swordcerer state to Idle while his avatar is still on the field
            // and re-spawn the Enter the Fray button.
            .add_systems(
                OnEnter(AppState::InGame),
                (reset_swordcerer_state, spawn_enter_fray_button).run_if(is_swordcerer),
            )
            .add_systems(
                OnEnter(InGameState::ScoreScreen),
                reset_swordcerer_state.run_if(is_swordcerer),
            )
            // Block normal spell casting while on field
            .add_systems(
                Update,
                block_spells_on_field
                    .run_if(is_spell_effects_active)
                    .run_if(is_swordcerer),
            )
            // Location click and retreat handling
            .add_systems(
                Update,
                (handle_location_click, handle_retreat)
                    .run_if(is_spell_effects_active)
                    .run_if(is_swordcerer),
            )
            // Player control systems
            .add_systems(
                Update,
                (
                    player_movement,
                    fire_missile,
                    sword_swing,
                    tick_cooldowns,
                    check_avatar_death,
                )
                    .run_if(is_gameplay_running)
                    .run_if(is_swordcerer)
                    .run_if(any_exist::<SwordcererAvatar>()),
            )
            // Sword arc collision and cleanup
            .add_systems(
                Update,
                update_sword_arcs
                    .run_if(any_exist::<SwordArc>())
                    .run_if(is_spell_effects_active),
            )
            // Health bar UI
            .add_systems(
                Update,
                (spawn_health_bar, update_health_bar, despawn_health_bar)
                    .run_if(is_spell_effects_active)
                    .run_if(is_swordcerer),
            )
            // Enter the Fray button
            .add_systems(
                Update,
                (
                    handle_enter_fray_click,
                    handle_enter_fray_hotkey,
                    update_enter_fray_visibility,
                )
                    .run_if(is_spell_effects_active)
                    .run_if(is_swordcerer)
                    .run_if(any_exist::<EnterFrayRoot>()),
            );
    }
}
