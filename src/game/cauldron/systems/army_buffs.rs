use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::{MAX_SHIELD_HP, SHIELD_KEEPALIVE_SECS};
use super::super::resources::CauldronBuffs;
use crate::game::units::components::{Corpse, Effectiveness, Health, Team, TemporaryHitPoints};
use crate::game::units::wizard::components::{LocalWizard, Mana, Wizard};

/// Heals all living defender units based on the active DefenderHealPerSecond buff.
pub fn heal_defenders(
    time: Res<Time>,
    cauldron_buffs: Res<CauldronBuffs>,
    mut defenders: Query<
        (
            &mut Health,
            &Team,
            Has<crate::game::units::wizard::archetypes::meteorologist::components::DryModifier>,
        ),
        (Without<Corpse>, Without<Wizard>),
    >,
) {
    use crate::game::units::wizard::archetypes::meteorologist::systems::apply_dry_healing_reduction;

    let heal_per_second = cauldron_buffs.defender_heal_per_second();
    if heal_per_second <= 0.0 {
        return;
    }
    let heal_amount = heal_per_second * time.delta_secs();
    for (mut health, team, is_dry) in &mut defenders {
        if *team == Team::Defenders {
            health.heal(apply_dry_healing_reduction(heal_amount, is_dry));
        }
    }
}

/// Applies or removes CauldronDamageBonus on all defenders based on active buffs.
pub fn buff_defender_damage(
    mut commands: Commands,
    cauldron_buffs: Res<CauldronBuffs>,
    defenders: Query<
        (Entity, &Team, Option<&CauldronDamageBonus>),
        (Without<Corpse>, Without<Wizard>),
    >,
) {
    let bonus = cauldron_buffs.defender_damage_bonus();
    for (entity, team, existing) in &defenders {
        if *team == Team::Defenders {
            if bonus > 0.0 {
                if existing.map(|b| b.0) != Some(bonus) {
                    commands.entity(entity).insert(CauldronDamageBonus(bonus));
                }
            } else if existing.is_some() {
                commands.entity(entity).remove::<CauldronDamageBonus>();
            }
        }
    }
}

/// Applies or removes CauldronDamageResistance on all defenders based on active buffs.
pub fn buff_defender_resistance(
    mut commands: Commands,
    cauldron_buffs: Res<CauldronBuffs>,
    defenders: Query<
        (Entity, &Team, Option<&CauldronDamageResistance>),
        (Without<Corpse>, Without<Wizard>),
    >,
) {
    let resistance = cauldron_buffs.damage_resistance_percent();
    for (entity, team, existing) in &defenders {
        if *team == Team::Defenders {
            if resistance > 0.0 {
                if existing.map(|r| r.0) != Some(resistance) {
                    commands
                        .entity(entity)
                        .insert(CauldronDamageResistance(resistance));
                }
            } else if existing.is_some() {
                commands.entity(entity).remove::<CauldronDamageResistance>();
            }
        }
    }
}

/// Applies or removes CauldronSpeedModifier on units based on active buffs.
///
/// Defenders get a speed bonus (Meadowsweet), attackers/undead get a slow
/// (Valerian). This is the SINGLE owner of `CauldronSpeedModifier` on every
/// team: in multiplayer the guest Alchemist's Meadowsweet (replicated as
/// `RemoteCauldronBuffs.speed_bonus`) speeds up the guest's own army
/// (Team::Attackers), so the Attacker value is the NET of the guest's bonus and
/// the host's Valerian slow. Keeping it in one system avoids two systems
/// fighting over the same component each frame.
pub fn apply_cauldron_speed_modifiers(
    mut commands: Commands,
    cauldron_buffs: Res<CauldronBuffs>,
    remote: Res<super::super::resources::RemoteCauldronBuffs>,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
    units: Query<
        (Entity, &Team, Option<&CauldronSpeedModifier>),
        (
            Without<Corpse>,
            Without<Wizard>,
            // Staging attackers can't be slowed (or sped up) by brews —
            // they're immune to all spell-side effects until their wave
            // activates. Harmless in MP: StagingAttacker never exists there.
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) {
    let defender_bonus = cauldron_buffs.defender_speed_bonus();
    let attacker_slow = cauldron_buffs.attacker_slow_percent();
    // The guest Alchemist's Meadowsweet speed buff. In VERSUS it speeds the
    // guest's own army (Attackers); in CO-OP both wizards defend, so it stacks
    // onto the shared Defender army instead.
    let remote_speed = remote.0.speed_bonus;
    let coop = session.is_some_and(|s| s.is_coop());

    for (entity, team, existing) in &units {
        let modifier = match team {
            Team::Defenders => {
                // Host's Meadowsweet, plus the guest Alchemist's in co-op. Both
                // sources are non-negative speed bonuses, so keep the original
                // `> 0.0` guard (a `!= 0.0` test would also apply a negative
                // value as a slow, which Defenders should never receive here).
                let bonus = defender_bonus + if coop { remote_speed } else { 0.0 };
                if bonus > 0.0 { Some(bonus) } else { None }
            }
            Team::Attackers => {
                if coop {
                    // Co-op: Attackers are the real enemies — host's slow only.
                    if attacker_slow > 0.0 {
                        Some(-attacker_slow)
                    } else {
                        None
                    }
                } else {
                    // Versus: the guest's own army — their Meadowsweet bonus
                    // minus the host's Valerian slow.
                    let net = remote_speed - attacker_slow;
                    if net != 0.0 { Some(net) } else { None }
                }
            }
            // Undead are nobody's brewed army — only the host's Valerian slow.
            Team::Undead => {
                if attacker_slow > 0.0 {
                    Some(-attacker_slow)
                } else {
                    None
                }
            }
        };

        if let Some(value) = modifier {
            // Insert when absent OR when the net value changed (the combined
            // Attacker value shifts as either side's buffs come and go).
            if existing.map(|m| m.0) != Some(value) {
                commands.entity(entity).insert(CauldronSpeedModifier(value));
            }
        } else if existing.is_some() {
            commands.entity(entity).remove::<CauldronSpeedModifier>();
        }
    }
}

/// Removes all cauldron buff components from units when no buffs are active.
///
/// This runs when buffs have just expired to clean up lingering components
/// that were inserted by the per-frame buff systems.
pub fn cleanup_cauldron_buff_components(
    mut commands: Commands,
    units: Query<
        (
            Entity,
            &Team,
            Option<&CauldronDamageBonus>,
            Option<&CauldronDamageResistance>,
            Option<&CauldronSpeedModifier>,
        ),
        (Without<Corpse>, Without<Wizard>),
    >,
    mut wizard: Query<&mut Mana, With<LocalWizard>>,
) {
    for (entity, team, damage_bonus, resistance, speed_mod) in &units {
        // Only clean up the HOST's own (Defender) buffs. The guest's replicated
        // Attacker buffs are managed by `apply_guest_army_buffs`.
        if *team != Team::Defenders {
            continue;
        }
        if damage_bonus.is_some() {
            commands.entity(entity).remove::<CauldronDamageBonus>();
        }
        if resistance.is_some() {
            commands.entity(entity).remove::<CauldronDamageResistance>();
        }
        if speed_mod.is_some() {
            commands.entity(entity).remove::<CauldronSpeedModifier>();
        }
    }
    // Effectiveness is self-reset every frame by `buff_defender_effectiveness`
    // (it writes 0.0 when no buff is active), so it is intentionally not handled
    // here — that also keeps cleanup from ever touching the poison-shared field.
    // Reset wizard mana max to base
    let base_max = crate::game::units::wizard::constants::MANA;
    for mut mana in &mut wizard {
        if mana.max != base_max {
            mana.max = base_max;
            if mana.current > mana.max {
                mana.current = mana.max;
            }
        }
    }
}

/// Grants temporary hit points to defenders based on active DefenderShieldPerSecond buff.
pub fn shield_defenders(
    time: Res<Time>,
    cauldron_buffs: Res<CauldronBuffs>,
    mut defenders: Query<
        (Entity, &Team, Option<&mut TemporaryHitPoints>),
        (Without<Corpse>, Without<Wizard>),
    >,
    mut commands: Commands,
) {
    let shield_per_second = cauldron_buffs.defender_shield_per_second();
    if shield_per_second <= 0.0 {
        return;
    }
    let shield_amount = shield_per_second * time.delta_secs();

    for (entity, team, temp_hp) in &mut defenders {
        if *team != Team::Defenders {
            continue;
        }
        if let Some(mut existing) = temp_hp {
            existing.amount = (existing.amount + shield_amount).min(MAX_SHIELD_HP);
            existing.time_remaining = SHIELD_KEEPALIVE_SECS; // Keep alive while buff is active
        } else {
            commands.entity(entity).insert(TemporaryHitPoints::new(
                shield_amount,
                SHIELD_KEEPALIVE_SECS,
            ));
        }
    }
}

/// Applies max mana multiplier to the wizard's mana pool based on active cauldron buffs.
pub fn apply_max_mana_buff(
    cauldron_buffs: Res<CauldronBuffs>,
    mut wizard: Query<&mut Mana, With<LocalWizard>>,
) {
    let multiplier = cauldron_buffs.max_mana_multiplier();
    let base_max = crate::game::units::wizard::constants::MANA;
    // Set the cap from the buff, OR reset it to base when no max-mana buff is
    // active. This system runs on BOTH multiplayer peers (the local wizard's
    // mana is local state); the host-only `cleanup_cauldron_buff_components`
    // never runs on the guest, so this self-contained reset is the only thing
    // that restores the guest's mana cap after the buff expires. The
    // change-guard below keeps it idempotent, so it's cheap to run every frame.
    let new_max = if multiplier > 1.0 {
        base_max * multiplier
    } else {
        base_max
    };
    for mut mana in &mut wizard {
        if (mana.max - new_max).abs() > f32::EPSILON {
            mana.max = new_max;
            if mana.current > mana.max {
                mana.current = mana.max;
            }
        }
    }
}

/// Applies the cauldron effectiveness bonus to all defenders. Writes the
/// dedicated `cauldron_spell_bonus` field (NOT the poison-shared `spell_bonus`).
///
/// Runs every frame (not gated on `has_active_buffs`) so it is self-resetting:
/// when no effectiveness buff is active `effectiveness_bonus()` returns 0.0 and
/// the field is zeroed. This is why `cleanup_cauldron_buff_components` no longer
/// touches effectiveness — a pure-effectiveness brew (no damage/resistance/speed
/// component) would otherwise never be reset by the cleanup pass.
pub fn buff_defender_effectiveness(
    cauldron_buffs: Res<CauldronBuffs>,
    remote: Res<super::super::resources::RemoteCauldronBuffs>,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
    mut defenders: Query<(&mut Effectiveness, &Team), Without<Corpse>>,
) {
    // In co-op the guest Alchemist's effectiveness brew also targets the shared
    // Defender army, so MERGE host + remote here — this system is the single
    // owner of `cauldron_spell_bonus` on Defenders, and `apply_guest_army_buffs`
    // skips the effectiveness write in co-op to avoid the two fighting.
    let remote_bonus = if session.is_some_and(|s| s.is_coop()) {
        remote.0.effectiveness_bonus
    } else {
        0.0
    };
    let bonus = cauldron_buffs.effectiveness_bonus() + remote_bonus;
    for (mut effectiveness, team) in &mut defenders {
        if *team == Team::Defenders
            && (effectiveness.cauldron_spell_bonus - bonus).abs() > f32::EPSILON
        {
            effectiveness.cauldron_spell_bonus = bonus;
        }
    }
}
