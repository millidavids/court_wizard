use bevy::prelude::*;

use crate::game::run_conditions::{
    any_exist, is_gameplay_running, is_local_wizard_active, is_spell_effects_active, is_swordcerer,
    is_swordcerer_participant,
};
use crate::networking::session::is_multiplayer_guest;
use crate::state::{AppState, InGameState, MultiplayerGameState};

use super::components::*;
use super::messages::*;
use super::networking;
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
            // State reset + Enter-the-Fray button run for BOTH peers when the
            // local player is the Swordcerer. The avatar itself is always
            // host-authoritative: SP/host spawn it locally, the guest asks the
            // host to spawn it and then streams control input.
            .add_systems(
                OnEnter(AppState::InGame),
                (reset_swordcerer_state, spawn_enter_fray_button).run_if(is_swordcerer),
            )
            .add_systems(
                OnEnter(AppState::MultiplayerGame),
                (reset_swordcerer_state, spawn_enter_fray_button)
                    .run_if(is_swordcerer)
                    // Must run AFTER the local wizard type is synced from the
                    // session, or `is_swordcerer` may read a stale SP value.
                    .after(crate::game::multiplayer::sync_wizard_type_from_session),
            )
            .add_systems(
                OnEnter(InGameState::ScoreScreen),
                reset_swordcerer_state.run_if(is_swordcerer),
            )
            .add_systems(
                OnEnter(MultiplayerGameState::ScoreScreen),
                reset_swordcerer_state.run_if(is_swordcerer),
            )
            // Block normal spell casting while on field
            .add_systems(
                Update,
                block_spells_on_field
                    .run_if(is_spell_effects_active)
                    .run_if(is_swordcerer),
            )
            // Location click (deploy) + retreat run for both peers' local
            // Swordcerer. `handle_location_click` spawns locally for SP/host and
            // sends a spawn request for the guest; `handle_retreat` restores the
            // wizard + state on whichever peer owns it.
            .add_systems(
                Update,
                (handle_location_click, handle_retreat)
                    .run_if(is_spell_effects_active)
                    .run_if(is_swordcerer),
            )
            // Player control of the LOCAL (host/SP) avatar from local input.
            .add_systems(
                Update,
                (
                    player_movement,
                    fire_missile,
                    sword_swing,
                    check_avatar_death,
                )
                    .run_if(is_gameplay_running)
                    .run_if(is_swordcerer)
                    .run_if(any_exist::<SwordcererAvatar>()),
            )
            // Cooldowns tick on the host for WHICHEVER avatar exists (host's own
            // or the guest's), so the guest's avatar isn't stuck after one shot.
            .add_systems(
                Update,
                tick_cooldowns
                    .run_if(is_gameplay_running)
                    .run_if(is_swordcerer_participant)
                    .run_if(any_exist::<SwordcererAvatar>()),
            )
            // ── Guest-controlled avatar (host-authoritative) ─────────────
            // Host: spawn the guest's avatar, drive it from streamed input, and
            // report its death.
            .add_systems(
                Update,
                (
                    networking::receive_swordcerer_spawn,
                    networking::apply_guest_avatar_input,
                    networking::check_guest_avatar_death,
                )
                    .chain()
                    .run_if(is_gameplay_running)
                    .run_if(networking::is_remote_swordcerer),
            )
            // Guest: stream input + react to the host's death notification.
            .add_systems(
                Update,
                (
                    networking::send_swordcerer_avatar_input,
                    networking::receive_swordcerer_death,
                )
                    .run_if(is_local_wizard_active)
                    .run_if(is_swordcerer)
                    .run_if(is_multiplayer_guest),
            )
            // Sword arc collision and cleanup
            .add_systems(
                Update,
                update_sword_arcs
                    .run_if(any_exist::<SwordArc>())
                    .run_if(is_spell_effects_active),
            )
            // Health bar UI for the local Swordcerer (both peers): the host reads
            // its own avatar, the guest reads its mirrored ghost avatar.
            .add_systems(
                Update,
                (spawn_health_bar, update_health_bar, despawn_health_bar)
                    .run_if(is_spell_effects_active)
                    .run_if(is_swordcerer),
            )
            // Enter the Fray button (both peers' local Swordcerer)
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
