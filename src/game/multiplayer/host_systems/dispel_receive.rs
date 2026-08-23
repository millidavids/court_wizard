use bevy::prelude::*;

use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::messages::announce_area_cast;
use crate::networking::entity_map::NetworkEntityId;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::SpellEffectKind;

/// Receives Dispel-driven messages from the remote peer: despawn one of *our*
/// spell effects by network id, or strip a SpellShield from a unit.
///
/// Runs on both peers. Dispel has to work in both directions — each peer's own
/// impact only despawns effects it owns, so the other peer's are unreachable
/// without this hand-off.
///
/// `NetworkEntityId` is optional because only the host assigns them
/// (`assign_network_ids` is host-gated). A guest's own effects have none, and
/// `collect_spell_effect_snapshots` falls back to the raw entity index when
/// building the snapshot — so the id we are handed must be resolved the same
/// way here or the guest would never find the effect the host is pointing at.
pub fn receive_dispel_messages(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    spell_effects: Query<(
        Entity,
        Option<&NetworkEntityId>,
        &crate::game::multiplayer::components::NetworkedSpellEffect,
        &Transform,
    )>,
    units_with_shield: Query<
        (Entity, &NetworkEntityId),
        With<crate::game::units::king::components::SpellShield>,
    >,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }
    let messages: Vec<NetworkMessage> = std::mem::take(&mut connection.incoming_messages);
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::DispelSpellEffect { target_network_id } => {
                // The guest forwards every host spell-effect ghost in the dispel
                // radius; the host is authoritative for *which* are dispellable.
                let Some((entity, kind, position)) =
                    spell_effects.iter().find_map(|(e, id, effect, transform)| {
                        let resolved = id.map_or(e.index_u32(), |n| n.0);
                        (resolved == target_network_id).then_some((
                            e,
                            effect.kind,
                            transform.translation,
                        ))
                    })
                else {
                    continue;
                };

                // Arcane crystals are excluded from `is_dispellable` because they
                // detonate instead of vanishing. Re-announce the dispel locally so
                // the host's own shatter path runs — that keeps the burst, the
                // Guardian Circle ward, and the owned-entity teardown identical to
                // a host-cast dispel. Without this a guest simply could not dispel
                // a host crystal at all.
                if kind == SpellEffectKind::ArcaneCrystal {
                    announce_area_cast(&mut commands, Spell::Dispel, position, 0.0, 1.0);
                    continue;
                }

                if crate::game::units::wizard::spells::dispel::bolt::is_dispellable(kind)
                    && let Ok(mut ec) = commands.get_entity(entity)
                {
                    ec.try_despawn();
                }
            }
            NetworkMessage::DispelShield { target_network_id } => {
                if let Some(entity) = units_with_shield
                    .iter()
                    .find_map(|(e, id)| (id.0 == target_network_id).then_some(e))
                    && let Ok(mut ec) = commands.get_entity(entity)
                {
                    ec.remove::<crate::game::units::king::components::SpellShield>();
                }
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}
