//! Melee combat resolution system.

use std::cmp::Ordering;

use bevy::prelude::*;
use rand::Rng;

use super::super::attack_cycle::GlobalAttackCycle;
use super::super::cauldron::components::{CauldronDamageBonus, CauldronDamageResistance};
use super::super::cauldron::resources::CauldronBuffs;
use super::super::constants::*;
use super::super::units::boss::components::Boss;
use super::super::units::components::{
    AttackTiming, Corpse, DamageMultiplier, Effectiveness, EliteAttackSpeedBonus, EliteDamageBonus,
    Flying, Health, Hitbox, RetaliationTarget, Team, TemporaryHitPoints, apply_damage_to_unit,
};

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn combat(
    attack_cycle: Res<GlobalAttackCycle>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    cauldron_buffs: Res<CauldronBuffs>,
    mut commands: Commands,
    mut all_units: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut AttackTiming,
            &Effectiveness,
            Option<&DamageMultiplier>,
            Option<&CauldronDamageBonus>,
            Option<&EliteDamageBonus>,
            // Spell modifiers on attacker side
            Has<super::super::units::components::SleepModifier>,
            Option<&super::super::units::components::BanishedModifier>,
            Option<&super::super::units::components::BattleHymnModifier>,
            Option<&super::super::units::components::BerserkerRageModifier>,
            Option<&super::super::units::components::FrozenSolidModifier>,
            (
                Option<&RetaliationTarget>,
                Option<&super::super::units::wizard::spells::guardian_circle::components::GuardianCircleShielded>,
                Has<super::super::units::infantry::components::Retreating>,
                Has<super::super::units::wizard::spells::mind_control::components::MassHysteriaTarget>,
                Option<&super::super::units::components::HasteModifier>,
                Option<&super::super::units::wizard::spells::haste::components::MomentumBuff>,
                Option<&super::super::units::components::Stunned>,
                Option<&super::super::units::wizard::spells::teleport::components::DisorientingHaste>,
                Option<&super::super::units::wizard::spells::fog_cloud::components::BlindingMistDebuff>,
                Option<&super::super::units::wizard::spells::berserker_rage::components::Frenzy>,
                Has<super::super::units::wizard::spells::berserker_rage::components::FrenzyActive>,
                Option<&super::super::units::wizard::spells::berserker_rage::components::Bloodlust>,
                Has<super::super::units::wizard::spells::berserker_rage::components::ContagiousRage>,
                Option<&EliteAttackSpeedBonus>,
                (
                    Has<super::super::units::archer::Archer>,
                    Has<super::super::units::infantry::Infantry>,
                    Has<super::super::units::assassin::Assassin>,
                    Option<&super::super::units::components::MeleeRangeBonus>,
                    Option<&super::super::units::components::Petrified>,
                    Has<super::super::units::components::FearModifier>,
                ),
            ),
        ),
        (
            Without<Corpse>,
            Without<Boss>,
            Without<Flying>,
            Without<super::super::units::components::MindControlled>,
        ),
    >,
    boss_units: Query<(Entity, &Transform, &Hitbox, &Team), (With<Boss>, Without<Corpse>)>,
    flying_units: Query<Entity, With<Flying>>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Option<&CauldronDamageResistance>,
        // New spell modifiers on target side
        Option<&super::super::units::components::FogEvasionModifier>,
        Option<&super::super::units::components::MarkedForDeathModifier>,
        Option<&super::super::units::wizard::spells::mind_control::components::Demoralized>,
        Option<&super::super::units::components::BerserkerRageModifier>,
        Option<&mut super::super::units::components::SleepModifier>,
        Option<&super::super::units::components::Comatose>,
        Option<&super::super::units::components::AnthemResilience>,
        Option<&super::super::units::wizard::spells::guardian_circle::components::GuardianCircleShielded>,
        Option<&super::super::units::wizard::spells::haste::components::FleetFeet>,
        Has<super::super::units::shielder::components::ShielderDamageReduction>,
        (
            Has<super::super::units::assassin::Assassin>,
            Has<super::super::units::archer::Archer>,
            Option<&super::super::units::components::MeleeDamageReduction>,
        ),
    )>,
    // Fog Cloud talent zones
    disorienting_zones: Query<
        &super::super::units::wizard::spells::fog_cloud::components::FogCloudZone,
        With<super::super::units::wizard::spells::fog_cloud::components::DisorientingVaporsZone>,
    >,
    mut talent_progress: Option<
        ResMut<super::super::units::wizard::talents::resources::BattleTalentProgress>,
    >,
    mut contagious_rage_events: MessageWriter<
        super::super::units::wizard::spells::berserker_rage::messages::ContagiousRageKillMessage,
    >,
    combat_anim_assets: (
        Res<super::super::units::infantry::resources::InfantryAssets>,
        Res<super::super::units::assassin::resources::AssassinAssets>,
        Res<super::super::units::undead::resources::UndeadAssets>,
    ),
) {
    let (combat_infantry_assets, combat_assassin_assets, combat_undead_assets) =
        &combat_anim_assets;
    let current_time = attack_cycle.current_time;
    // Real previous cycle time, recorded by `tick()`. Using the actual
    // delta (rather than `current_time - APPROX_FRAME_TIME`) keeps the
    // can_attack window exactly as wide as the cycle advanced this frame,
    // which prevents the "fast-frame re-fire" instakill (window too wide
    // → unit fires twice) and the "slow-frame slot skip" miss (window
    // too narrow → unit misses its turn) both inherent to the old
    // constant-subtraction approximation.
    let last_time = attack_cycle.previous_time;

    // Collect snapshot of all units for enemy detection (includes bosses as targets)
    // Exclude banished units so they cannot be targeted while removed from play.
    let mut units_snapshot: Vec<_> = all_units
        .iter()
        .filter(|(_, _, _, _, _, _, _, _, _, _, banished, _, _, _, _)| banished.is_none())
        .map(
            |(entity, transform, hitbox, team, _, _, _, _, _, _, _, _, _, _, (..))| {
                (entity, transform.translation, *hitbox, *team)
            },
        )
        .collect();

    // Include boss units in the snapshot so they can be targeted by defenders
    for (entity, transform, hitbox, team) in &boss_units {
        units_snapshot.push((entity, transform.translation, *hitbox, *team));
    }

    // Collect flying entity set for targeting restriction (only archers can melee-hit flying units)
    let flying_set: std::collections::HashSet<Entity> = flying_units.iter().collect();

    // Collect fog cloud talent zone snapshots for combat checks
    let disorienting_snapshot: Vec<(Vec3, f32)> = disorienting_zones
        .iter()
        .map(|z| (z.origin, z.radius))
        .collect();

    // Collect post-combat actions to apply after the main loop
    let mut post_combat_removes: Vec<(Entity, PostCombatAction)> = Vec::new();

    // Bloodlust: accumulate heal amounts per attacker entity
    let mut bloodlust_heals: Vec<(Entity, f32)> = Vec::new();

    // Process each unit's combat
    for (
        attacker_entity,
        attacker_transform,
        attacker_hitbox,
        attacker_team,
        mut attack_timing,
        effectiveness,
        damage_mult,
        cauldron_damage_bonus,
        elite_damage_bonus,
        is_sleeping,
        banished,
        battle_hymn,
        berserker_rage_attacker,
        frozen_solid,
        (
            retaliation,
            guardian_circle_attacker,
            is_retreating,
            has_mass_hysteria,
            haste_modifier,
            momentum_buff,
            stunned,
            disorienting_haste,
            blinding_mist_debuff,
            frenzy,
            has_frenzy_active,
            bloodlust,
            has_contagious_rage,
            elite_attack_speed,
            (
                attacker_is_archer,
                attacker_is_infantry,
                attacker_is_assassin,
                melee_range_bonus,
                petrified,
                has_fear,
            ),
        ),
    ) in &mut all_units
    {
        // Skip attack if sleeping, banished, frozen, stunned, or retreating
        if is_sleeping
            || banished.is_some()
            || frozen_solid.is_some()
            || stunned.is_some()
            || petrified.is_some()
            || has_fear
            || is_retreating
        {
            continue;
        }

        let retaliation_entity = retaliation.map(|r| r.0);

        // Find nearest enemy within attack range (also considers retaliation target)
        if let Some((target_entity, _, _)) = units_snapshot
            .iter()
            .filter(|(entity, _, _, team)| {
                // Flying targets can only be hit by archers in melee combat
                if flying_set.contains(entity) && !attacker_is_archer {
                    return false;
                }
                *entity != attacker_entity
                    && (has_mass_hysteria
                        || retaliation_entity == Some(*entity)
                        || attacker_team.is_enemy(team))
            })
            .filter_map(|(entity, target_pos, target_hitbox, _)| {
                let dx = attacker_transform.translation.x - target_pos.x;
                let dz = attacker_transform.translation.z - target_pos.z;
                let distance = (dx * dx + dz * dz).sqrt();
                let mut attack_range = (attacker_hitbox.radius + target_hitbox.radius)
                    * ATTACK_RANGE_MULTIPLIER
                    + melee_range_bonus.map_or(0.0, |b| b.0);
                // Blinding Mist: halve attack range
                if let Some(debuff) = blinding_mist_debuff {
                    attack_range *= debuff.range_mult;
                }
                if distance <= attack_range {
                    Some((entity, target_pos, distance))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal))
        {
            // Calculate effective attack speed (BattleHymn + Haste + DisorientingHaste + Frenzy + Elite + cauldron buff for defenders)
            let mut attack_speed_bonus = battle_hymn.map_or(0.0, |b| b.attack_speed)
                + haste_modifier.map_or(0.0, |h| h.attack_speed)
                + disorienting_haste.map_or(0.0, |d| d.attack_speed)
                + if has_frenzy_active {
                    frenzy.map_or(0.0, |f| f.attack_speed_bonus)
                } else {
                    0.0
                }
                + elite_attack_speed.map_or(0.0, |e| e.0)
                + if attacker_is_assassin {
                    crate::game::units::assassin::constants::ASSASSIN_ATTACK_SPEED_BONUS
                } else {
                    0.0
                };
            if *attacker_team == Team::Defenders {
                let cauldron_speed = cauldron_buffs.attack_speed_multiplier();
                if cauldron_speed > 1.0 {
                    attack_speed_bonus += cauldron_speed - 1.0;
                }
            }
            // Attack speed is a per-unit cooldown, not a widened can_attack
            // window (which re-fired every frame — see can_attack_with_speed_bonus).
            if attack_timing.can_attack_with_speed_bonus(
                current_time,
                last_time,
                attack_cycle.cycle_duration,
                attack_speed_bonus,
            ) {
                // Disorienting Vapors: if attacker is in a disorienting fog zone,
                // 20% chance to redirect the attack to a same-team ally
                let mut actual_target = *target_entity;
                if !disorienting_snapshot.is_empty() {
                    use super::super::units::wizard::spells::fog_cloud::systems::is_in_fog_zone;
                    let attacker_pos = attacker_transform.translation;
                    if is_in_fog_zone(attacker_pos, &disorienting_snapshot)
                        && game_rng.0.random::<f32>() < super::super::units::wizard::spells::fog_cloud::constants::DISORIENTING_VAPORS_CHANCE
                    {
                        // Find a random same-team unit to attack instead
                        let count = units_snapshot
                            .iter()
                            .filter(|(e, _, _, t)| *e != attacker_entity && *t == *attacker_team)
                            .count();
                        if count > 0 {
                            let idx = game_rng.0.random_range(0..count);
                            if let Some((e, _, _, _)) = units_snapshot
                                .iter()
                                .filter(|(e, _, _, t)| *e != attacker_entity && *t == *attacker_team)
                                .nth(idx)
                            {
                                actual_target = *e;
                            }
                        }
                    }
                }

                if let Ok((
                    mut target_health,
                    mut temp_hp,
                    target_resistance,
                    fog_evasion,
                    marked_for_death,
                    demoralized,
                    berserker_rage_target,
                    mut target_sleeping,
                    target_comatose,
                    target_anthem_resilience,
                    guardian_circle_shielded,
                    target_fleet_feet,
                    has_shielder_reduction,
                    (target_is_assassin, target_is_archer, melee_damage_reduction),
                )) = health_query.get_mut(actual_target)
                {
                    // Check fog evasion
                    if let Some(evasion) = fog_evasion {
                        let roll = game_rng.0.random::<f32>();
                        if roll < evasion.evasion_chance {
                            // Attack evaded - still record the attack timing
                            attack_timing.record_attack(current_time);
                            // Track talent progress
                            if let Some(ref mut progress) = talent_progress {
                                progress.increment(
                                    super::super::units::wizard::components::Spell::FogCloud,
                                    1,
                                );
                            }
                            continue;
                        }
                    }

                    // Check Fleet Feet dodge (Haste talent)
                    if let Some(ff) = target_fleet_feet
                        && ff.dodges_remaining > 0
                    {
                        attack_timing.record_attack(current_time);
                        post_combat_removes
                            .push((actual_target, PostCombatAction::ConsumeFleetFeetDodge));
                        continue;
                    }

                    // Calculate base damage with attacker bonuses
                    let damage_percentage = damage_mult.map_or(0.0, |d| d.0)
                        + cauldron_damage_bonus.map_or(0.0, |b| b.0)
                        + elite_damage_bonus.map_or(0.0, |b| b.0)
                        + battle_hymn.map_or(0.0, |b| b.damage_bonus)
                        + berserker_rage_attacker.map_or(0.0, |b| b.damage_bonus)
                        + guardian_circle_attacker.map_or(0.0, |g| g.fortified_damage_bonus)
                        + momentum_buff.map_or(0.0, |m| m.damage_mult);
                    let damage_multiplier = 1.0 + damage_percentage;
                    let mut modified_damage =
                        ATTACK_DAMAGE * effectiveness.multiplier() * damage_multiplier;

                    // Apply target's damage resistance (Wormwood brew)
                    if let Some(resistance) = target_resistance {
                        modified_damage *= 1.0 - resistance.0;
                    }

                    // Apply Battle Hymn damage reduction (Anthem of Resilience talent)
                    if let Some(resilience) = target_anthem_resilience {
                        modified_damage *= 1.0 - resilience.damage_reduction;
                    }

                    // Apply Guardian Circle Sanctuary damage reduction
                    if let Some(gc) = guardian_circle_shielded
                        && gc.sanctuary_reduction > 0.0
                    {
                        modified_damage *= 1.0 - gc.sanctuary_reduction;
                    }

                    // Apply Shielder damage reduction (20% less damage from melee)
                    if has_shielder_reduction {
                        modified_damage *=
                            crate::game::units::shielder::constants::SHIELDER_DAMAGE_REDUCTION;
                    }

                    // Apply Assassin damage modifiers (defensive)
                    if target_is_assassin {
                        if attacker_is_archer {
                            // Assassins take 50% less damage from archers
                            modified_damage *=
                                crate::game::units::assassin::constants::ARCHER_DAMAGE_REDUCTION;
                        } else if attacker_is_infantry {
                            // Assassins take 20% more damage from infantry
                            modified_damage *=
                                crate::game::units::assassin::constants::INFANTRY_DAMAGE_INCREASE;
                        }
                    }

                    // Apply Assassin damage bonus (offensive)
                    if attacker_is_assassin && target_is_archer {
                        modified_damage *=
                            crate::game::units::assassin::constants::ASSASSIN_VS_ARCHER_DAMAGE;
                    }

                    // Apply target's Mark of Death amplification
                    if let Some(mark) = marked_for_death {
                        modified_damage *= 1.0 + mark.damage_amplification;
                    }

                    // Apply Traitor's Mark demoralization
                    if let Some(demoralized) = demoralized {
                        modified_damage *= 1.0 + demoralized.damage_amplification;
                    }

                    // Apply target's Berserker Rage vulnerability
                    if let Some(rage) = berserker_rage_target {
                        modified_damage *= 1.0 + rage.damage_vulnerability;
                    }

                    // Apply Sleep bonus damage (first hit wakes and deals bonus)
                    if let Some(sleep) = &mut target_sleeping {
                        modified_damage *= sleep.bonus_damage_multiplier;

                        // Comatose: only wake if damage exceeds threshold
                        let comatose_blocks_wake = target_comatose.is_some_and(|c| {
                            modified_damage < c.wake_threshold * target_health.max
                        });

                        if !comatose_blocks_wake {
                            post_combat_removes
                                .push((actual_target, PostCombatAction::RemoveSleep));
                        }
                    }

                    // Apply melee damage reduction (e.g. Ogre boss)
                    if let Some(reduction) = melee_damage_reduction {
                        modified_damage *= reduction.multiplier;
                    }

                    apply_damage_to_unit(
                        &mut target_health,
                        temp_hp.as_deref_mut(),
                        modified_damage,
                    );

                    // Bloodlust: heal attacker for a fraction of damage dealt
                    if let Some(bl) = bloodlust {
                        bloodlust_heals.push((attacker_entity, modified_damage * bl.heal_fraction));
                    }

                    // Contagious Rage: track kills by enraged units
                    if has_contagious_rage && target_health.is_dead() {
                        contagious_rage_events.write(
                        super::super::units::wizard::spells::berserker_rage::messages::ContagiousRageKillMessage {
                            killer: attacker_entity,
                        },
                    );
                    }

                    attack_timing.record_attack(current_time);

                    // Trigger melee attack animation.
                    // Archers handle their own animations in archer_melee_combat.
                    let attack_textures = if attacker_is_infantry {
                        if *attacker_team == Team::Undead {
                            Some((
                                combat_undead_assets.attacking_texture.clone(),
                                combat_undead_assets.sprite_texture.clone(),
                            ))
                        } else {
                            Some((
                                combat_infantry_assets.attacking_texture.clone(),
                                combat_infantry_assets.sprite_texture.clone(),
                            ))
                        }
                    } else if attacker_is_assassin {
                        Some((
                            combat_assassin_assets.attacking_texture.clone(),
                            combat_assassin_assets.sprite_texture.clone(),
                        ))
                    } else {
                        None
                    };
                    if let Some((attack_tex, walk_tex)) = attack_textures {
                        commands.entity(attacker_entity).insert(
                            super::super::units::components::CombatAnimation::new_attack(
                                attack_tex, walk_tex,
                            ),
                        );
                    }
                }
            }
        }
    }

    // Apply bloodlust healing to attackers
    for (entity, heal_amount) in bloodlust_heals {
        if let Ok((mut health, ..)) = health_query.get_mut(entity) {
            health.heal(heal_amount);
        }
    }

    // Apply post-combat actions
    for (entity, action) in post_combat_removes {
        match action {
            PostCombatAction::RemoveSleep => {
                commands.entity(entity).remove::<(
                    super::super::units::components::SleepModifier,
                    super::super::units::components::NightTerrors,
                    super::super::units::components::Comatose,
                    super::super::units::components::NarcolepticWave,
                    super::super::units::components::Sleepwalking,
                )>();
            }
            PostCombatAction::ConsumeFleetFeetDodge => {
                // Remove FleetFeet — single dodge consumed
                commands
                    .entity(entity)
                    .remove::<super::super::units::wizard::spells::haste::components::FleetFeet>();
            }
        }
    }
}

/// Post-combat actions to defer component removal after the main combat loop.
enum PostCombatAction {
    RemoveSleep,
    ConsumeFleetFeetDodge,
}
