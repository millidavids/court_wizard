use bevy::prelude::*;

use crate::networking::resources::NetworkConnection;

/// Guest-side forwarder: when a spell on the guest applies damage to a
/// ghost unit, SP's `apply_spell_damage` inserts a `PendingDamageEffect`
/// on the target. We **poll** for any such component (deliberately not
/// `Added<>`) each frame because `apply_spell_damage` queues inserts via
/// `Commands`; the component isn't visible to an `Added` filter until
/// after the next command flush, and a ghost killed before the flush
/// would never be forwarded. Polling catches both same-frame inserts and
/// any that survive across a frame.
///
/// The host then owns status-effect bookkeeping (DoT stacks, durations,
/// snapshot flags). After forwarding, the local `PendingDamageEffect` is
/// removed so the guest doesn't *also* tick the DoT (which would double-
/// apply on top of the host-ticked damage that propagates back via CRDT).
///
/// Excremage conversion: a guest playing Excremage should turn every spell
/// hit into a Poop hit, but `process_pending_damage_effects` does that
/// lookup against the LOCAL `GameConfig.wizard_type` — and on the host
/// that's the host's wizard, not the guest's. So we do the conversion here
/// on the guest before forwarding, using the guest's own config.
pub fn forward_spell_hits_to_host(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    config: Res<crate::config::GameConfig>,
    hits: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::components::PendingDamageEffect,
        ),
        With<super::super::super::components::GhostEntity>,
    >,
) {
    let excremage = config.wizard_type == crate::config::WizardType::Excremage;
    for (entity, net_id, pending) in &hits {
        let damage_type = if excremage {
            crate::game::units::damage::DamageType::Poop
        } else {
            pending.damage_type
        };
        connection.outgoing_messages.push(
            crate::networking::protocol::NetworkMessage::SpellHitUnit {
                target_network_id: net_id.0,
                damage: pending.damage,
                damage_type: damage_type.to_u8(),
            },
        );
        commands
            .entity(entity)
            .remove::<crate::game::units::components::PendingDamageEffect>();
    }
}
