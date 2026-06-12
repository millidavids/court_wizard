use super::super::components::DispelImpact;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::utils::xz_distance;
use bevy::prelude::*;

/// Per-impact dedupe state for `forward_dispel_impacts_to_host`. Stores the
/// `NetworkEntityId`s that have already been forwarded for this impact so we
/// don't re-send messages every frame as the radius expands.
#[derive(Component, Clone, Default)]
pub struct DispelForwarded {
    pub spell_effects: Vec<u32>,
    pub shielded_units: Vec<u32>,
}

/// Guest-side visual ticker for ghost `DispelImpact` entities. The main
/// `update_dispel_impacts` system is filtered `Without<GhostSpellEffect>`
/// (so the guest doesn't double-despawn host spell effects), which leaves
/// ghost impacts with `time_alive = 0.0` and scale = `Vec3::ZERO` for
/// their entire lifetime — the expanding sphere is invisible to the
/// remote peer. This system ticks ONLY the ghost copies, animating the
/// growth without running any of the gameplay-mutating logic.
pub fn tick_ghost_dispel_impacts(
    time: Res<Time>,
    mut commands: Commands,
    mut impacts: Query<
        (Entity, &mut DispelImpact, &mut Transform),
        With<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
) {
    let delta = time.delta_secs();
    for (entity, mut impact, mut transform) in &mut impacts {
        impact.time_alive += delta;
        if impact.time_alive >= impact.duration {
            // Local cleanup — host's snapshot stale-id path will also tear
            // this down when the host's authoritative entity expires, but
            // doing it here avoids leaving a zero-radius shell after the
            // animation finishes.
            commands.entity(entity).try_despawn();
            continue;
        }
        let radius = impact.expand_speed * impact.time_alive;
        transform.scale = Vec3::splat(radius);
    }
}

/// Guest-side forwarder: ships `DispelSpellEffect` / `DispelShield` messages
/// for every dispel impact the guest owns. The host then performs the
/// authoritative despawn / shield-strip via `receive_dispel_messages`. Each
/// (impact, target) pair is recorded in `DispelForwarded` so we don't spam
/// the wire — once a target has been forwarded for a given impact entity,
/// we skip it on subsequent frames.
///
/// Runs only on the guest (gated in the plugin); on the host the existing
/// `update_dispel_impacts` mutates real entities directly.
#[allow(clippy::too_many_arguments)]
pub fn forward_dispel_impacts_to_host(
    mut commands: Commands,
    mut connection: ResMut<crate::networking::resources::NetworkConnection>,
    // Guest-local impacts only — host's own dispel impacts mirrored to the
    // guest as ghosts would otherwise be re-forwarded back at the host,
    // wasting wire and racing with the host's own update_dispel_impacts.
    impacts: Query<
        (Entity, &DispelImpact, &Transform, Option<&DispelForwarded>),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    // The host's persistent spell-effects are mirrored on the guest as ghosts
    // (`GhostSpellEffect` + `NetworkEntityId`, but NOT `NetworkedSpellEffect`).
    // We forward those ghosts by `net_id`; the host validates dispellability
    // authoritatively in `receive_dispel_messages`. (The guest's OWN effects are
    // dispelled locally by `update_dispel_impacts` — they are not ghosts.)
    spell_effects: Query<
        (&Transform, &crate::networking::entity_map::NetworkEntityId),
        With<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    shielded_units: Query<
        (&Transform, &crate::networking::entity_map::NetworkEntityId),
        With<SpellShield>,
    >,
) {
    for (impact_entity, impact, impact_transform, forwarded) in &impacts {
        let radius = impact.expand_speed * impact.time_alive;
        if radius <= 0.0 {
            continue;
        }
        let center = impact_transform.translation;

        let mut already = forwarded.cloned().unwrap_or_default();
        let mut changed = false;

        for (spell_transform, net_id) in &spell_effects {
            if already.spell_effects.contains(&net_id.0) {
                continue;
            }
            let dist = xz_distance(center, spell_transform.translation);
            if dist <= radius {
                connection.outgoing_messages.push(
                    crate::networking::protocol::NetworkMessage::DispelSpellEffect {
                        target_network_id: net_id.0,
                    },
                );
                already.spell_effects.push(net_id.0);
                changed = true;
            }
        }

        for (unit_transform, net_id) in &shielded_units {
            if already.shielded_units.contains(&net_id.0) {
                continue;
            }
            let dist = xz_distance(center, unit_transform.translation);
            if dist <= radius {
                connection.outgoing_messages.push(
                    crate::networking::protocol::NetworkMessage::DispelShield {
                        target_network_id: net_id.0,
                    },
                );
                already.shielded_units.push(net_id.0);
                changed = true;
            }
        }

        // Only write the dedupe state back if something was actually
        // forwarded this frame — avoids ~30 unnecessary Vec clones +
        // deferred-insert commands over the impact's ~0.5s lifetime.
        if changed {
            commands.entity(impact_entity).insert(already);
        }
    }
}
