use bevy::prelude::*;

use crate::game::run_conditions::{
    any_exist, is_battlemage, is_gameplay_running, is_spell_effects_active,
};
use crate::state::InGameState;

use super::components::*;
use super::messages::*;
use super::systems::*;

/// Plugin for the Battlemage wizard archetype.
pub(in crate::game) struct BattlemagePlugin;

impl Plugin for BattlemagePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RetreatMessage>()
            // Initialize state and spawn Enter the Fray button on entering gameplay
            .add_systems(
                OnEnter(InGameState::Running),
                (reset_battlemage_state, spawn_enter_fray_button).run_if(is_battlemage),
            )
            .add_systems(
                OnEnter(InGameState::ScoreScreen),
                reset_battlemage_state.run_if(is_battlemage),
            )
            // Block normal spell casting while on field
            .add_systems(
                Update,
                block_spells_on_field
                    .run_if(is_spell_effects_active)
                    .run_if(is_battlemage),
            )
            // Location click and retreat handling
            .add_systems(
                Update,
                (handle_location_click, handle_retreat)
                    .run_if(is_spell_effects_active)
                    .run_if(is_battlemage),
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
                    .run_if(is_battlemage)
                    .run_if(any_exist::<BattlemageAvatar>()),
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
                    .run_if(is_battlemage),
            )
            // Enter the Fray button
            .add_systems(
                Update,
                (
                    handle_enter_fray_click,
                    update_enter_fray_visibility,
                )
                    .run_if(is_spell_effects_active)
                    .run_if(is_battlemage)
                    .run_if(any_exist::<EnterFrayRoot>()),
            );
    }
}
