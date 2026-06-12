use std::cmp::Ordering;

use super::super::super::super::components::Spell;
use super::super::components::*;
use super::super::constants;
use crate::game::constants::{UNIT_HEALTH, UNIT_MOVEMENT_SPEED};
use crate::game::units::components::{Corpse, Effectiveness, PermanentCorpse, Team};
use crate::game::units::infantry::constants::UNDEAD_SPRITE_TINT;
use crate::game::units::undead::resources::UndeadAssets;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use bevy::prelude::*;

/// Applies talent-specific components to a newly raised undead entity.
///
/// Shared between `resurrect_nearest_corpse` (direct casting) and
/// `handle_perpetual_unrest` (chain-raising on kill).
fn apply_talent_components(
    entity_cmds: &mut EntityCommands,
    talent_params: &RaiseTheDeadTalentParams,
    empowerment: f32,
) {
    // Compute damage bonus from empowerment + talents
    let mut damage_bonus = if empowerment > 1.0 { 0.25 } else { 0.0 };
    if talent_params.empowered_undead {
        damage_bonus += constants::EMPOWERED_UNDEAD_DAMAGE_MULT - 1.0;
    }
    if talent_params.revenant_lord {
        damage_bonus += constants::REVENANT_DAMAGE_MULT - 1.0;
    }
    if damage_bonus > 0.0 {
        let mut effectiveness = Effectiveness::new();
        effectiveness.spell_bonus = damage_bonus;
        entity_cmds.insert(effectiveness);
    }

    if talent_params.plague_bearer {
        entity_cmds.insert(PlagueBearerAura::new(
            constants::PLAGUE_BEARER_DPS,
            constants::PLAGUE_BEARER_RADIUS,
            constants::PLAGUE_BEARER_TICK_INTERVAL,
        ));
    }
    if talent_params.undead_detonation {
        entity_cmds.insert(UndeadDetonation {
            damage: constants::UNDEAD_DETONATION_DAMAGE,
            radius: constants::UNDEAD_DETONATION_RADIUS,
        });
    }
    if talent_params.perpetual_unrest {
        entity_cmds.insert(PerpetualUnrest {
            raise_radius: constants::PERPETUAL_UNREST_RADIUS,
        });
    }
    if talent_params.revenant_lord {
        entity_cmds.insert(RevenantLord {
            raise_radius: constants::REVENANT_RAISE_RADIUS,
            raise_interval: constants::REVENANT_RAISE_INTERVAL,
            raise_timer: 0.0,
        });
    }
}

/// Finds the nearest corpse to a position within a given radius.
pub(crate) fn find_nearest_corpse(
    corpse_query: &Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    target_pos: Vec3,
    radius: f32,
) -> Option<(Entity, Vec3)> {
    corpse_query
        .iter()
        .filter_map(|(entity, transform)| {
            let dist = target_pos.distance(transform.translation);
            (dist <= radius).then_some((entity, transform.translation, dist))
        })
        .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal))
        .map(|(entity, pos, _)| (entity, pos))
}

/// Raises a corpse entity as undead infantry with talent components.
///
/// Shared between direct casting, Perpetual Unrest, and Revenant Lord.
#[allow(clippy::too_many_arguments)]
pub(crate) fn raise_corpse_as_undead(
    commands: &mut Commands,
    corpse_entity: Entity,
    position: Vec3,
    health: f32,
    speed: f32,
    talent_params: &RaiseTheDeadTalentParams,
    empowerment: f32,
    undead_assets: &UndeadAssets,
    materials: &mut Assets<StandardMaterial>,
    talent_progress: Option<&mut BattleTalentProgress>,
) {
    crate::game::units::systems::resurrect_corpse_as_infantry(
        commands,
        corpse_entity,
        position,
        Team::Undead,
        health,
        speed,
        UNDEAD_SPRITE_TINT,
        undead_assets.sprite_texture.clone(),
        undead_assets.sprite_mesh.clone(),
        materials,
        Some(undead_assets.death_texture.clone()),
    );

    let mut entity_cmds = commands.entity(corpse_entity);
    entity_cmds.insert(RaisedUndead);
    apply_talent_components(&mut entity_cmds, talent_params, empowerment);

    if let Some(progress) = talent_progress {
        progress.increment(Spell::RaiseTheDead, 1);
    }
}

/// Resurrects the nearest corpse to the target position as undead infantry.
/// Returns true if a corpse was raised.
#[allow(clippy::too_many_arguments)]
fn resurrect_nearest_corpse(
    commands: &mut Commands,
    target_pos: Vec3,
    corpse_query: &Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    undead_assets: &UndeadAssets,
    materials: &mut Assets<StandardMaterial>,
    empowerment: f32,
    talent_params: &RaiseTheDeadTalentParams,
    talent_progress: Option<&mut BattleTalentProgress>,
) -> bool {
    let search_radius = constants::RESURRECTION_RADIUS * empowerment * talent_params.radius_mult;

    let Some((corpse_entity, position)) =
        find_nearest_corpse(corpse_query, target_pos, search_radius)
    else {
        return false;
    };

    // Compute HP with Empowered Undead and Revenant Lord modifiers
    let mut hp_mult = 1.0;
    if talent_params.empowered_undead {
        hp_mult *= constants::EMPOWERED_UNDEAD_HP_MULT;
    }
    if talent_params.revenant_lord {
        hp_mult *= constants::REVENANT_HP_MULT;
    }

    let health = UNIT_HEALTH * empowerment * hp_mult;
    let speed = UNIT_MOVEMENT_SPEED * 0.5 * empowerment;

    raise_corpse_as_undead(
        commands,
        corpse_entity,
        position,
        health,
        speed,
        talent_params,
        empowerment,
        undead_assets,
        materials,
        talent_progress,
    );

    true
}

/// Multiplayer dispatcher for the raise action. On the host (and SP) it just
/// calls `resurrect_nearest_corpse` locally; on the guest it looks up the
/// nearest GHOST-corpse's `NetworkEntityId` and ships a `RaiseCorpse` message
/// — the host then performs the authoritative raise via
/// `receive_raise_corpse_messages` and the new undead propagates back to the
/// guest in the regular unit snapshot. Talent flags are packed into the
/// message's `flags` u32 so Plague Bearer / Perpetual Unrest / Revenant Lord
/// / Undead Detonation get applied host-side.
#[allow(clippy::too_many_arguments)]
pub(crate) fn try_raise_or_forward(
    commands: &mut Commands,
    target_pos: Vec3,
    corpse_query: &Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    undead_assets: &UndeadAssets,
    materials: &mut Assets<StandardMaterial>,
    empowerment: f32,
    talent_params: &RaiseTheDeadTalentParams,
    talent_progress: Option<&mut BattleTalentProgress>,
    is_guest: bool,
    connection: Option<&mut crate::networking::resources::NetworkConnection>,
    corpse_ids: &Query<&crate::networking::entity_map::NetworkEntityId, With<Corpse>>,
) -> bool {
    if !is_guest {
        return resurrect_nearest_corpse(
            commands,
            target_pos,
            corpse_query,
            undead_assets,
            materials,
            empowerment,
            talent_params,
            talent_progress,
        );
    }

    let Some(connection) = connection else {
        return false;
    };
    let search_radius = constants::RESURRECTION_RADIUS * empowerment * talent_params.radius_mult;
    let Some((corpse_entity, _)) = find_nearest_corpse(corpse_query, target_pos, search_radius)
    else {
        return false;
    };
    let Ok(net_id) = corpse_ids.get(corpse_entity) else {
        // Corpse exists locally but has no network ID — likely a
        // late-spawn race; skip this tick and try again on the next.
        return false;
    };

    use crate::networking::protocol::status_flags as sf;
    let mut flags: u32 = 0;
    if talent_params.plague_bearer {
        flags |= sf::RAISE_PLAGUE_BEARER;
    }
    if talent_params.perpetual_unrest {
        flags |= sf::RAISE_PERPETUAL_UNREST;
    }
    if talent_params.revenant_lord {
        flags |= sf::RAISE_REVENANT_LORD;
    }
    if talent_params.undead_detonation {
        flags |= sf::RAISE_UNDEAD_DETONATION;
    }
    if talent_params.empowered_undead {
        flags |= sf::RAISE_EMPOWERED_UNDEAD;
    }

    connection
        .outgoing_messages
        .push(crate::networking::protocol::NetworkMessage::RaiseCorpse {
            target_network_id: net_id.0,
            flags,
            empowerment,
        });
    true
}
