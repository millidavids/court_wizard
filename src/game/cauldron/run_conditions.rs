use bevy::prelude::*;

use super::components::{
    BrewBubble, Cauldron, CauldronBrewingEffects, CauldronDamageBonus, CauldronDamageResistance,
    CauldronSpeedModifier, CauldronState,
};
use super::resources::{CauldronBuffs, RemoteCauldronBuffs};
use crate::config::WizardType;
use crate::game::units::components::Team;
use crate::networking::session::MultiplayerSession;

/// True when the OPPONENT is an Alchemist — host-side gate for applying the
/// guest's replicated army buffs.
pub fn is_remote_alchemist(session: Option<Res<MultiplayerSession>>) -> bool {
    session.is_some_and(|s| s.remote_wizard() == WizardType::Alchemist)
}

/// Returns true if the cauldron is currently brewing.
pub fn cauldron_is_brewing(query: Query<&CauldronState, With<Cauldron>>) -> bool {
    query.iter().any(|state| state.is_brewing())
}

/// Returns true if any cauldron buffs are currently active.
pub fn has_active_buffs(cauldron_buffs: Res<CauldronBuffs>) -> bool {
    cauldron_buffs.has_active_buffs()
}

/// Returns true if any brew bubbles exist in the world.
pub fn has_brew_bubbles(query: Query<(), With<BrewBubble>>) -> bool {
    !query.is_empty()
}

/// Returns true if the HOST's own (Defender) cauldron buff components need
/// cleanup (no host buffs active but Defender components remain). Only considers
/// `Team::Defenders` so the guest's replicated Attacker buffs (managed
/// independently by `apply_guest_army_buffs`) never trigger the host's cleanup.
///
/// In CO-OP the guest Alchemist's buffs ALSO land on `Team::Defenders` (the
/// shared army), so cleanup must not fire while the guest's brew is still active
/// — otherwise it would strip the guest's live buffs every frame the host isn't
/// brewing.
pub fn needs_buff_cleanup(
    cauldron_buffs: Res<CauldronBuffs>,
    remote: Res<RemoteCauldronBuffs>,
    session: Option<Res<MultiplayerSession>>,
    damage_bonus: Query<&Team, With<CauldronDamageBonus>>,
    resistance: Query<&Team, With<CauldronDamageResistance>>,
    speed_mod: Query<&Team, With<CauldronSpeedModifier>>,
) -> bool {
    if cauldron_buffs.has_active_buffs() {
        return false;
    }
    // Co-op: the guest's replicated buffs keep the shared Defender army buffed
    // even when the host has no brew — don't clean those up.
    if session.is_some_and(|s| s.is_coop()) && remote.0.has_active() {
        return false;
    }
    damage_bonus.iter().any(|t| *t == Team::Defenders)
        || resistance.iter().any(|t| *t == Team::Defenders)
        || speed_mod.iter().any(|t| *t == Team::Defenders)
}

/// Returns true if the cauldron has brewing effects active.
pub fn has_brewing_effects(
    query: Query<(), (With<Cauldron>, With<CauldronBrewingEffects>)>,
) -> bool {
    !query.is_empty()
}
