use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::*;

use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};
use crate::networking::session::is_multiplayer_guest;
use crate::state::AppState;
use crate::state::MultiplayerGameState;

use crate::game::messages::ComboDiscoveredMessage;

use super::messages::*;
use super::resources::{CauldronBuffs, RemoteCauldronBuffs};
use super::run_conditions::{
    cauldron_is_brewing, has_active_buffs, has_brew_bubbles, has_brewing_effects,
    is_remote_alchemist, needs_buff_cleanup,
};
use super::systems;

/// Plugin managing the cauldron brewing system.
pub struct CauldronPlugin;

impl Plugin for CauldronPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CauldronBuffs>()
            .init_resource::<super::resources::PhilosophersStoneUsed>()
            .init_resource::<RemoteCauldronBuffs>()
            .add_message::<StartBrewMessage>()
            .add_message::<BrewCompleteMessage>()
            .add_message::<CancelBrewMessage>()
            .add_message::<ComboDiscoveredMessage>()
            // Message handlers run across all InGame states (SP) and the full
            // MultiplayerGame state tree so messages sent from CauldronMenu
            // aren't lost during the state transition back to Running.
            .add_systems(
                Update,
                (
                    systems::handle_start_brew.run_if(on_message::<StartBrewMessage>),
                    systems::handle_cancel_brew.run_if(on_message::<CancelBrewMessage>),
                )
                    .run_if(
                        in_state(AppState::InGame)
                            .or_else(in_state(MultiplayerGameState::CauldronMenu))
                            .or_else(in_state(MultiplayerGameState::Running)),
                    ),
            )
            // LOCAL brew loop — brew timer, completion, the local wizard's mana
            // buff, and all cauldron visuals. These only touch the per-peer
            // `CauldronBuffs` resource, the local wizard, and the cauldron's own
            // visual entity, so they run on BOTH peers (`is_spell_effects_active`,
            // which preserves SP simulation-pause semantics). Without this the
            // GUEST Alchemist's `update_brew_timer` never ticks (it was gated
            // host-only) and the brew never completes.
            .add_systems(
                Update,
                (
                    systems::start_brewing_effects,
                    systems::update_brewing_timer.run_if(has_brewing_effects),
                    systems::update_cauldron_animation,
                    systems::update_brewing_effects.run_if(has_brewing_effects),
                    systems::update_brew_timer.run_if(cauldron_is_brewing),
                    systems::handle_brew_complete.run_if(on_message::<BrewCompleteMessage>),
                    systems::tick_active_buffs.run_if(has_active_buffs),
                    // NOT gated on `has_active_buffs`: it must also run with no
                    // buffs active to reset the local wizard's mana cap to base
                    // when a max-mana buff expires (the host-only buff cleanup
                    // never runs on the guest). It self-guards and is idempotent.
                    systems::apply_max_mana_buff,
                    systems::block_spells_during_brewing.run_if(cauldron_is_brewing),
                    systems::update_brew_bubble.run_if(has_brew_bubbles),
                )
                    .chain()
                    .run_if(is_spell_effects_active),
            )
            // HOST-authoritative army buffs — these mutate the host-simulated
            // defender army (heal / damage / resistance / speed / shield /
            // effectiveness) and must stay host-only (`is_gameplay_running`).
            .add_systems(
                Update,
                (
                    systems::heal_defenders.run_if(has_active_buffs),
                    systems::buff_defender_damage.run_if(has_active_buffs),
                    systems::buff_defender_resistance.run_if(has_active_buffs),
                    // Also runs when the GUEST is an Alchemist (no host buffs
                    // needed): it is the single owner of CauldronSpeedModifier and
                    // must apply the guest's replicated Meadowsweet to the guest's
                    // Attacker army even when the host isn't brewing.
                    systems::apply_cauldron_speed_modifiers
                        .run_if(has_active_buffs.or_else(is_remote_alchemist)),
                    systems::shield_defenders.run_if(has_active_buffs),
                    // NOT gated on `has_active_buffs`: it writes the dedicated
                    // `Effectiveness.cauldron_spell_bonus` field every frame so it
                    // self-resets to 0 when the brew lapses (a pure-effectiveness
                    // brew leaves no component for `cleanup` to key off of). The
                    // internal change-guard prevents per-frame write spam.
                    systems::buff_defender_effectiveness,
                    systems::cleanup_cauldron_buff_components.run_if(needs_buff_cleanup),
                )
                    .chain()
                    // Run after the local bucket's `tick_active_buffs` so the army
                    // buffs apply against the freshly-ticked buff state (preserving
                    // the old single-chain ordering across the split).
                    .after(systems::tick_active_buffs)
                    .run_if(is_gameplay_running),
            )
            // ── Multiplayer: replicate the GUEST Alchemist's army buffs ──────
            // The guest sends its buff scalars; the host applies them to the
            // guest's army (Attackers), since the army-buff systems above are
            // host-authoritative and only buff the host's own Defenders.
            .add_systems(
                OnEnter(AppState::InGame),
                systems::reset_remote_cauldron_buffs,
            )
            .add_systems(
                OnEnter(AppState::MultiplayerGame),
                systems::reset_remote_cauldron_buffs,
            )
            .add_systems(
                Update,
                systems::send_cauldron_buffs_to_host
                    .run_if(is_spell_effects_active)
                    .run_if(is_multiplayer_guest),
            )
            .add_systems(
                Update,
                (
                    systems::receive_cauldron_buffs,
                    systems::apply_guest_army_buffs,
                )
                    .chain()
                    // Both this and `buff_defender_effectiveness` hold
                    // `&mut Effectiveness` (Attackers vs Defenders, runtime-
                    // disjoint). Order them explicitly so the schedule is
                    // unambiguous.
                    .after(systems::buff_defender_effectiveness)
                    .run_if(is_gameplay_running)
                    // Only when the GUEST is an Alchemist — otherwise these would
                    // needlessly scan every unit each frame.
                    .run_if(is_remote_alchemist),
            );
    }
}
