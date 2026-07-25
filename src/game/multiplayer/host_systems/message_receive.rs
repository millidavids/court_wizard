use bevy::prelude::*;

use crate::game::resources::GameOutcome;
use crate::game::units::components::Health;
use crate::networking::entity_map::NetworkEntityId;
use crate::networking::protocol::{GameOverResult, NetworkMessage};
use crate::networking::resources::NetworkConnection;
use crate::state::MultiplayerGameState;

use super::king_death::end_mp_match;

/// Host: the guest forfeited — end the match with the host winning. The resulting
/// `GameOver` drives the guest's normal score-screen transition.
pub fn receive_mp_forfeit(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    mut game_outcome: ResMut<GameOutcome>,
    mut next_state: ResMut<NextState<MultiplayerGameState>>,
    kill_stats: Res<crate::game::resources::KillStats>,
    local_stats: Res<crate::game::multiplayer::score_stats::LocalWizardStats>,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }
    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();
    let mut forfeited = false;
    for msg in messages {
        match msg {
            NetworkMessage::Forfeit => forfeited = true,
            other => unhandled.push(other),
        }
    }
    connection.incoming_messages.extend(unhandled);
    if forfeited {
        end_mp_match(
            GameOverResult::HostWins,
            &mut commands,
            &mut connection,
            &mut game_outcome,
            &mut next_state,
            &kill_stats,
            &local_stats,
        );
    }
}

/// Receives `TeleportUnits` messages from the guest and executes the teleport on the host.
///
/// Unit positions are host-authoritative, so when the guest casts Teleport it sends
/// a message with source/dest/radius. The host applies the actual position changes.
#[allow(clippy::type_complexity)]
pub fn receive_teleport_message(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    units_query: Query<
        (
            Entity,
            &Transform,
            Option<&crate::game::units::components::Team>,
        ),
        (
            With<crate::game::units::components::Teleportable>,
            Without<
                crate::game::units::wizard::spells::teleport::components::TeleportDestinationCircle,
            >,
            Without<crate::game::units::wizard::spells::teleport::components::TeleportSourceCircle>,
            Without<crate::game::units::components::Corpse>,
            // MP has no staging phase — this filter only keeps the query type
            // matching the shared teleport helpers.
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::TeleportUnits {
                source_x,
                source_z,
                dest_x,
                dest_z,
                radius,
            } => {
                let source = Vec3::new(source_x, 0.0, source_z);
                let dest = Vec3::new(dest_x, 0.0, dest_z);
                crate::game::units::wizard::spells::teleport::systems::teleport_units_with_radius(
                    &mut rand::rng(),
                    source,
                    dest,
                    radius,
                    &units_query,
                    &mut commands,
                );
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Receives `SpellHitUnit` messages from the guest and inserts the standard
/// `PendingDamageEffect` on the matching authoritative unit. From there the
/// host runs SP's full status-effect pipeline (`process_pending_damage_effects`
/// → `FireDoT` / `Shocked` / etc. → `update_fire_dot` → CRDT damage tick),
/// and the resulting status flag is shipped back to the guest in the next
/// state snapshot for visual rendering — the guest never owns status state
/// itself.
pub fn receive_spell_hit_messages(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    units: Query<(Entity, &NetworkEntityId), With<Health>>,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::SpellHitUnit {
                target_network_id,
                damage,
                damage_type,
            } => {
                // HP damage flows via the CRDT pipeline (guest's local
                // `apply_spell_damage` decrements ghost Health →
                // `sync_health_to_crdt` records into guest's CRDT slot →
                // CRDT snapshot → `receive_crdt_snapshot` merges →
                // host's Health.current re-derives). This message ONLY
                // carries the status-type so the host can stack the
                // right DoT/status — `apply_spell_damage` on the guest
                // already accounted for the immediate hit via CRDT.
                let Some(local_entity) = units
                    .iter()
                    .find_map(|(e, id)| (id.0 == target_network_id).then_some(e))
                else {
                    continue;
                };
                if let Ok(mut ec) = commands.get_entity(local_entity) {
                    ec.insert(crate::game::units::components::PendingDamageEffect {
                        damage,
                        damage_type: crate::game::units::damage::DamageType::from_u8(damage_type),
                        // Forwarded hits always come from the guest, who commands
                        // the Attacker army — so the guest's own (Attacker) King
                        // takes its own friendly fire, while the host's (Defender)
                        // King's shield still blocks these enemy spells.
                        source_team: Some(crate::game::units::components::Team::Attackers),
                    });
                }
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Receives `RaiseCorpse` messages: convert a specific corpse on the host
/// into an Undead unit. Uses the shared SP `resurrect_corpse_as_infantry`
/// helper with the undead asset set, so the resulting unit looks and behaves
/// identically to one raised by the SP code path.
///
/// The new undead unit gets a `NetworkEntityId` next frame via
/// `assign_network_ids`, and its existence then propagates to the guest via
/// the regular unit snapshot — no extra message needed.
pub fn receive_raise_corpse_messages(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    corpses: Query<
        (Entity, &NetworkEntityId, &Transform),
        With<crate::game::units::components::Corpse>,
    >,
    undead_assets: Option<Res<crate::game::units::undead::resources::UndeadAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }
    let Some(undead_assets) = undead_assets else {
        // Undead assets unavailable (e.g. preloaded by spell plugin not
        // initialised yet) — drop the message; the guest will retry on
        // the next cast.
        return;
    };

    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::RaiseCorpse {
                target_network_id,
                flags,
                empowerment,
            } => {
                let Some((corpse_entity, _, transform)) =
                    corpses.iter().find(|(_, id, _)| id.0 == target_network_id)
                else {
                    continue;
                };
                use crate::game::constants::{UNIT_HEALTH, UNIT_MOVEMENT_SPEED};
                use crate::game::units::components::{Effectiveness, Team};
                use crate::game::units::infantry::constants::UNDEAD_SPRITE_TINT;
                use crate::game::units::wizard::spells::raise_the_dead::{
                    components as raise_components, constants as raise_constants,
                };
                use crate::networking::protocol::status_flags as sf;

                // HP multiplier — match SP's `resurrect_nearest_corpse` flow:
                // stack EMPOWERED_UNDEAD and REVENANT_LORD multipliers, then
                // multiply by empowerment for the final HP.
                let mut hp_mult = 1.0_f32;
                if flags & sf::RAISE_EMPOWERED_UNDEAD != 0 {
                    hp_mult *= raise_constants::EMPOWERED_UNDEAD_HP_MULT;
                }
                if flags & sf::RAISE_REVENANT_LORD != 0 {
                    hp_mult *= raise_constants::REVENANT_HP_MULT;
                }
                let health = UNIT_HEALTH * empowerment.max(0.01) * hp_mult;
                let speed = UNIT_MOVEMENT_SPEED * 0.5 * empowerment.max(0.01);

                crate::game::units::systems::resurrect_corpse_as_infantry(
                    &mut commands,
                    corpse_entity,
                    transform.translation,
                    Team::Undead,
                    health,
                    speed,
                    UNDEAD_SPRITE_TINT,
                    undead_assets.sprite_texture.clone(),
                    undead_assets.sprite_mesh.clone(),
                    &mut materials,
                    Some(undead_assets.death_texture.clone()),
                );

                // Apply talent marker components matching the SP path's
                // `apply_talent_components` so all behaviors run on the
                // host's authoritative new undead. Includes Effectiveness
                // so damage bonuses from Empowered Undead / Revenant Lord
                // / empowerment are actually applied (was previously
                // missing — undead dealt base damage regardless of talent).
                if let Ok(mut ec) = commands.get_entity(corpse_entity) {
                    ec.insert(raise_components::RaisedUndead);

                    let mut damage_bonus = if empowerment > 1.0 { 0.25 } else { 0.0 };
                    if flags & sf::RAISE_EMPOWERED_UNDEAD != 0 {
                        damage_bonus += raise_constants::EMPOWERED_UNDEAD_DAMAGE_MULT - 1.0;
                    }
                    if flags & sf::RAISE_REVENANT_LORD != 0 {
                        damage_bonus += raise_constants::REVENANT_DAMAGE_MULT - 1.0;
                    }
                    if damage_bonus > 0.0 {
                        let mut effectiveness = Effectiveness::new();
                        effectiveness.spell_bonus = damage_bonus;
                        ec.insert(effectiveness);
                    }

                    if flags & sf::RAISE_PLAGUE_BEARER != 0 {
                        ec.insert(raise_components::PlagueBearerAura::new(
                            raise_constants::PLAGUE_BEARER_DPS,
                            raise_constants::PLAGUE_BEARER_RADIUS,
                            raise_constants::PLAGUE_BEARER_TICK_INTERVAL,
                        ));
                    }
                    if flags & sf::RAISE_PERPETUAL_UNREST != 0 {
                        ec.insert(raise_components::PerpetualUnrest {
                            raise_radius: raise_constants::PERPETUAL_UNREST_RADIUS,
                        });
                    }
                    if flags & sf::RAISE_REVENANT_LORD != 0 {
                        ec.insert(raise_components::RevenantLord {
                            raise_radius: raise_constants::REVENANT_RAISE_RADIUS,
                            raise_interval: raise_constants::REVENANT_RAISE_INTERVAL,
                            raise_timer: 0.0,
                        });
                    }
                    if flags & sf::RAISE_UNDEAD_DETONATION != 0 {
                        ec.insert(raise_components::UndeadDetonation {
                            damage: raise_constants::UNDEAD_DETONATION_DAMAGE,
                            radius: raise_constants::UNDEAD_DETONATION_RADIUS,
                        });
                    }
                }
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}
