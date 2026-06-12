use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::{MAX_SHIELD_HP, SHIELD_KEEPALIVE_SECS};
use super::super::resources::{CauldronArmyScalars, CauldronBuffs, RemoteCauldronBuffs};
use crate::game::units::components::{Corpse, Effectiveness, Health, Team, TemporaryHitPoints};
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::NetworkConnection;
use crate::networking::session::MultiplayerSession;

// ── Multiplayer: replicate the guest Alchemist's army buffs to the host ──────

/// Guest → host: sends the local Alchemist's army-buff scalars whenever they
/// change, so the host can apply them to the guest's army. No-op in
/// single-player (no connection).
pub fn send_cauldron_buffs_to_host(
    cauldron_buffs: Res<CauldronBuffs>,
    connection: Option<ResMut<NetworkConnection>>,
    mut last_sent: Local<Option<CauldronArmyScalars>>,
) {
    let Some(mut connection) = connection else {
        return;
    };
    let scalars = cauldron_buffs.army_scalars();
    if *last_sent == Some(scalars) {
        return;
    }
    *last_sent = Some(scalars);
    connection
        .outgoing_messages
        .push(NetworkMessage::CauldronBuffsSync {
            heal_per_second: scalars.heal_per_second,
            damage_bonus: scalars.damage_bonus,
            resistance_percent: scalars.resistance_percent,
            shield_per_second: scalars.shield_per_second,
            speed_bonus: scalars.speed_bonus,
            effectiveness_bonus: scalars.effectiveness_bonus,
        });
}

/// Host: receives the guest Alchemist's army-buff scalars.
pub fn receive_cauldron_buffs(
    connection: Option<ResMut<NetworkConnection>>,
    mut remote: ResMut<RemoteCauldronBuffs>,
) {
    let Some(mut connection) = connection else {
        return;
    };
    if connection.incoming_messages.is_empty() {
        return;
    }
    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();
    for msg in messages {
        match msg {
            NetworkMessage::CauldronBuffsSync {
                heal_per_second,
                damage_bonus,
                resistance_percent,
                shield_per_second,
                speed_bonus,
                effectiveness_bonus,
            } => {
                remote.0 = CauldronArmyScalars {
                    heal_per_second,
                    damage_bonus,
                    resistance_percent,
                    shield_per_second,
                    speed_bonus,
                    effectiveness_bonus,
                };
            }
            other => unhandled.push(other),
        }
    }
    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Resets the replicated guest buffs at game start so a stale multiplayer value
/// never buffs a single-player enemy army.
pub fn reset_remote_cauldron_buffs(mut remote: ResMut<RemoteCauldronBuffs>) {
    *remote = RemoteCauldronBuffs::default();
}

/// Host: applies the guest Alchemist's replicated buffs to the guest's army
/// (Team::Attackers). Additive alongside the host's own Defender buffs and
/// self-managing: it inserts/removes the buff components (and writes the
/// effectiveness field) as the scalars change, so no separate cleanup pass is
/// needed. Covers heal / shield / damage / resistance / effectiveness.
/// Effectiveness rides `Effectiveness.cauldron_spell_bonus` (its own field, kept
/// clear of poison's `spell_bonus`), and is written every frame so it self-resets
/// to 0 when the guest's brew lapses. The guest's Meadowsweet SPEED buff is
/// handled separately by `apply_cauldron_speed_modifiers` (the single owner of
/// `CauldronSpeedModifier`); the enemy-slow stays host-only (it targets the
/// host's army, which the host's own cleanup manages).
#[allow(clippy::type_complexity)]
pub fn apply_guest_army_buffs(
    time: Res<Time>,
    remote: Res<RemoteCauldronBuffs>,
    session: Option<Res<MultiplayerSession>>,
    mut commands: Commands,
    mut vitals: Query<
        (Entity, &Team, &mut Health, Option<&mut TemporaryHitPoints>),
        Without<Corpse>,
    >,
    mut component_q: Query<
        (
            Entity,
            &Team,
            &mut Effectiveness,
            Has<CauldronDamageBonus>,
            Has<CauldronDamageResistance>,
        ),
        Without<Corpse>,
    >,
) {
    let s = remote.0;
    let heal = s.heal_per_second * time.delta_secs();
    let shield = s.shield_per_second * time.delta_secs();

    // The guest Alchemist's army: their own Attackers in versus, the SHARED
    // Defender army in co-op.
    //
    // CO-OP buff ownership (see also `buff_defender_effectiveness`,
    // `needs_buff_cleanup`, `apply_cauldron_speed_modifiers`):
    //   - effectiveness: owned/merged by `buff_defender_effectiveness` → skipped
    //     here in co-op.
    //   - speed: owned/merged by `apply_cauldron_speed_modifiers`.
    //   - heal/shield: additive — these add on top of the host's own
    //     `heal_defenders`/`shield_defenders`, so they STACK naturally.
    //   - cleanup: `needs_buff_cleanup` is co-op-aware, so the guest's Defender
    //     components are no longer stripped while the guest is brewing.
    // REMAINING (cauldron co-op pass): `CauldronDamageBonus`/`CauldronDamageResistance`
    //   are inserted with the guest's value (insert-if-none), so if BOTH wizards
    //   brew damage/resistance they do not SUM (first-writer-wins). True summing
    //   needs an owner that reads host `CauldronBuffs` + remote and live-updates
    //   the component value; do that when co-op is wired and testable.
    let coop = session.is_some_and(|sess| sess.is_coop());
    let guest_army = if coop {
        Team::Defenders
    } else {
        Team::Attackers
    };

    for (entity, team, mut health, temp_hp) in &mut vitals {
        if *team != guest_army {
            continue;
        }
        if s.heal_per_second > 0.0 {
            health.heal(heal);
        }
        if s.shield_per_second > 0.0 {
            match temp_hp {
                Some(mut existing) => {
                    existing.amount = (existing.amount + shield).min(MAX_SHIELD_HP);
                    existing.time_remaining = SHIELD_KEEPALIVE_SECS;
                }
                None => {
                    commands
                        .entity(entity)
                        .insert(TemporaryHitPoints::new(shield, SHIELD_KEEPALIVE_SECS));
                }
            }
        }
    }

    for (entity, team, mut effectiveness, has_damage, has_resistance) in &mut component_q {
        if *team != guest_army {
            continue;
        }
        // Effectiveness (VERSUS only): owns the guest's Attacker effectiveness,
        // written every frame so it self-resets when the brew lapses. In co-op
        // the shared Defender effectiveness is merged by
        // `buff_defender_effectiveness`, so skip here to avoid the two fighting.
        if !coop
            && (effectiveness.cauldron_spell_bonus - s.effectiveness_bonus).abs() > f32::EPSILON
        {
            effectiveness.cauldron_spell_bonus = s.effectiveness_bonus;
        }
        if s.damage_bonus > 0.0 {
            if !has_damage {
                commands
                    .entity(entity)
                    .insert(CauldronDamageBonus(s.damage_bonus));
            }
        } else if has_damage {
            commands.entity(entity).remove::<CauldronDamageBonus>();
        }
        if s.resistance_percent > 0.0 {
            if !has_resistance {
                commands
                    .entity(entity)
                    .insert(CauldronDamageResistance(s.resistance_percent));
            }
        } else if has_resistance {
            commands.entity(entity).remove::<CauldronDamageResistance>();
        }
    }
}
