use bevy::prelude::*;

use crate::networking::entity_map::NetworkEntityId;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::NetworkConnection;

/// Receives Dispel-driven messages from the guest: despawn a spell-effect
/// entity by network ID, or strip SpellShield from a unit.
pub fn receive_dispel_messages(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    spell_effects: Query<(
        Entity,
        &NetworkEntityId,
        &crate::game::multiplayer::components::NetworkedSpellEffect,
    )>,
    units_with_shield: Query<
        (Entity, &NetworkEntityId),
        With<crate::game::units::king::components::SpellShield>,
    >,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }
    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::DispelSpellEffect { target_network_id } => {
                // The guest forwards every host spell-effect ghost in the dispel
                // radius; the host is authoritative for *which* are dispellable.
                if let Some(entity) = spell_effects.iter().find_map(|(e, id, effect)| {
                    (id.0 == target_network_id
                        && crate::game::units::wizard::spells::dispel::bolt::is_dispellable(
                            effect.kind,
                        ))
                    .then_some(e)
                }) && let Ok(mut ec) = commands.get_entity(entity)
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
