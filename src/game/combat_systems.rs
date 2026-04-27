use std::cmp::Ordering;

use bevy::prelude::*;
use rand::Rng;

use super::cauldron::components::{
    CauldronDamageBonus, CauldronDamageResistance, CauldronSpeedModifier,
};
use super::cauldron::resources::CauldronBuffs;
use super::components::Velocity;
use super::constants::*;
use super::plugin::GlobalAttackCycle;
use super::units::archer::Archer;
use super::units::boss::components::Boss;
use super::units::components::{
    AttackTiming, CORPSE_MATERIAL_VARIANTS, Corpse, DamageMultiplier, Effectiveness,
    EliteAttackSpeedBonus, EliteDamageBonus, Flying, Health, Hitbox, Invulnerable, MovementSpeed,
    ResidualFireDamaged, RetaliationTarget, SpellDamaged, Team, TemporaryHitPoints,
    apply_damage_to_unit,
};
use super::units::infantry::components::Infantry;
use super::units::systems::corpse_material_for_team;

use crate::game::achievements::messages::{
    CloseCallMessage, DefenderKilledBySpellMessage, EnemyKilledMessage, ScorchedEarthMessage,
};
use crate::game::achievements::resources::AchievementResource;

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
            Has<super::units::components::SleepModifier>,
            Option<&super::units::components::BanishedModifier>,
            Option<&super::units::components::BattleHymnModifier>,
            Option<&super::units::components::BerserkerRageModifier>,
            Option<&super::units::components::FrozenSolidModifier>,
            (
                Option<&RetaliationTarget>,
                Option<&super::units::wizard::spells::guardian_circle::components::GuardianCircleShielded>,
                Has<super::units::infantry::components::Retreating>,
                Has<super::units::wizard::spells::mind_control::components::MassHysteriaTarget>,
                Option<&super::units::components::HasteModifier>,
                Option<&super::units::wizard::spells::haste::components::MomentumBuff>,
                Option<&super::units::components::Stunned>,
                Option<&super::units::wizard::spells::teleport::components::DisorientingHaste>,
                Option<&super::units::wizard::spells::fog_cloud::components::BlindingMistDebuff>,
                Option<&super::units::wizard::spells::berserker_rage::components::Frenzy>,
                Has<super::units::wizard::spells::berserker_rage::components::FrenzyActive>,
                Option<&super::units::wizard::spells::berserker_rage::components::Bloodlust>,
                Has<super::units::wizard::spells::berserker_rage::components::ContagiousRage>,
                Option<&EliteAttackSpeedBonus>,
                (
                    Has<super::units::archer::Archer>,
                    Has<super::units::infantry::Infantry>,
                    Has<super::units::assassin::Assassin>,
                    Option<&super::units::components::MeleeRangeBonus>,
                    Option<&super::units::components::Petrified>,
                    Has<super::units::components::FearModifier>,
                ),
            ),
        ),
        (
            Without<Corpse>,
            Without<Boss>,
            Without<Flying>,
            Without<super::units::components::MindControlled>,
        ),
    >,
    boss_units: Query<(Entity, &Transform, &Hitbox, &Team), (With<Boss>, Without<Corpse>)>,
    flying_units: Query<Entity, With<Flying>>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Option<&CauldronDamageResistance>,
        // New spell modifiers on target side
        Option<&super::units::components::FogEvasionModifier>,
        Option<&super::units::components::MarkedForDeathModifier>,
        Option<&super::units::wizard::spells::mind_control::components::Demoralized>,
        Option<&super::units::components::BerserkerRageModifier>,
        Option<&mut super::units::components::SleepModifier>,
        Option<&super::units::components::Comatose>,
        Option<&super::units::components::AnthemResilience>,
        Option<&super::units::wizard::spells::guardian_circle::components::GuardianCircleShielded>,
        Option<&super::units::wizard::spells::haste::components::FleetFeet>,
        Has<super::units::shielder::components::ShielderDamageReduction>,
        (
            Has<super::units::assassin::Assassin>,
            Has<super::units::archer::Archer>,
            Option<&super::units::boss::ogre::MeleeDamageReduction>,
        ),
    )>,
    // Fog Cloud talent zones
    disorienting_zones: Query<
        &super::units::wizard::spells::fog_cloud::components::FogCloudZone,
        With<super::units::wizard::spells::fog_cloud::components::DisorientingVaporsZone>,
    >,
    mut talent_progress: Option<
        ResMut<super::units::wizard::talents::resources::BattleTalentProgress>,
    >,
    mut contagious_rage_events: MessageWriter<
        super::units::wizard::spells::berserker_rage::messages::ContagiousRageKillMessage,
    >,
    combat_anim_assets: (
        Res<super::units::infantry::resources::InfantryAssets>,
        Res<super::units::assassin::resources::AssassinAssets>,
        Res<super::units::undead::resources::UndeadAssets>,
    ),
) {
    let (combat_infantry_assets, combat_assassin_assets, combat_undead_assets) =
        &combat_anim_assets;
    let current_time = attack_cycle.current_time;
    let last_time = (current_time - APPROX_FRAME_TIME).max(0.0);

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
            let effective_last_time = if attack_speed_bonus > 0.0 {
                // Shrink the window between last_time and current_time to simulate faster attacks
                current_time - (current_time - last_time) * (1.0 + attack_speed_bonus)
            } else {
                last_time
            };

            if attack_timing.can_attack(current_time, effective_last_time) {
                // Disorienting Vapors: if attacker is in a disorienting fog zone,
                // 20% chance to redirect the attack to a same-team ally
                let mut actual_target = *target_entity;
                if !disorienting_snapshot.is_empty() {
                    use super::units::wizard::spells::fog_cloud::systems::is_in_fog_zone;
                    let attacker_pos = attacker_transform.translation;
                    if is_in_fog_zone(attacker_pos, &disorienting_snapshot)
                        && game_rng.0.random::<f32>() < super::units::wizard::spells::fog_cloud::constants::DISORIENTING_VAPORS_CHANCE
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
                                    super::units::wizard::components::Spell::FogCloud,
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
                        super::units::wizard::spells::berserker_rage::messages::ContagiousRageKillMessage {
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
                            super::units::components::CombatAnimation::new_attack(
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
                    super::units::components::SleepModifier,
                    super::units::components::NightTerrors,
                    super::units::components::Comatose,
                    super::units::components::NarcolepticWave,
                    super::units::components::Sleepwalking,
                )>();
            }
            PostCombatAction::ConsumeFleetFeetDodge => {
                // Remove FleetFeet — single dodge consumed
                commands
                    .entity(entity)
                    .remove::<super::units::wizard::spells::haste::components::FleetFeet>();
            }
        }
    }
}

/// Post-combat actions to defer component removal after the main combat loop.
enum PostCombatAction {
    RemoveSleep,
    ConsumeFleetFeetDodge,
}

/// Converts dead units to corpses instead of despawning them.
///
/// Negates all damage for units with the `Invulnerable` component by restoring
/// their health to the snapshot each frame. Must run after all combat systems.
pub fn enforce_invulnerability(
    mut query: Query<(&mut Invulnerable, &mut Health), Without<Corpse>>,
) {
    for (mut invuln, mut health) in &mut query {
        // Restore health to at least the snapshot (damage negated, heals preserved)
        health.current = health.current.max(invuln.health_snapshot).min(health.max);
        invuln.health_snapshot = health.current;
    }
}

/// When a unit's health reaches zero, this system replaces the unit's material with
/// a pre-loaded corpse material based on team and converts the unit into a corpse
/// that slows living units walking over it.
/// Also records the kill in the kill statistics resource.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn convert_dead_to_corpses(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut kill_stats: ResMut<super::resources::KillStats>,
    mut spell_kill_events: MessageWriter<DefenderKilledBySpellMessage>,
    mut enemy_kill_events: MessageWriter<EnemyKilledMessage>,
    mut scorched_earth_events: MessageWriter<ScorchedEarthMessage>,
    mut marked_kill_events: MessageWriter<super::achievements::messages::MarkedForDeathKillMessage>,
    mut drop_events: MessageWriter<super::drops::messages::SpawnIngredientDropMessage>,
    mut close_call_events: MessageWriter<CloseCallMessage>,
    close_call_achievement: Res<super::achievements::resources::CloseCallAchievement>,
    wizard_query: Query<&Transform, With<super::units::wizard::components::Wizard>>,
    query: Query<
        (
            Entity,
            &Health,
            &Team,
            &Transform,
            Option<&Infantry>,
            Option<&Archer>,
            Option<&super::units::assassin::Assassin>,
            Option<&super::units::dispeller::components::Dispeller>,
            Option<&super::units::shielder::components::Shielder>,
            Option<&super::units::healer::components::Healer>,
            Option<&super::units::king::components::King>,
            Option<&Boss>,
            Option<&SpellDamaged>,
            (
                Option<&ResidualFireDamaged>,
                Option<&super::units::components::MarkedForDeathModifier>,
                Option<&super::units::aerialist::Aerialist>,
                Option<&super::units::brute::components::Brute>,
                Has<Flying>,
            ),
        ),
        (
            Without<Corpse>,
            Without<super::units::wizard::spells::fog_cloud::components::PhantomUnit>,
        ),
    >,
    death_assets: (
        Res<super::units::infantry::resources::InfantryAssets>,
        Res<super::units::archer::resources::ArcherAssets>,
        Res<super::units::assassin::resources::AssassinAssets>,
        Res<super::units::dispeller::resources::DispellerAssets>,
        Res<super::units::undead::resources::UndeadAssets>,
        Res<super::units::king::resources::KingAssets>,
    ),
    death_assets_2: (
        Res<super::units::shielder::resources::ShielderAssets>,
        Res<super::units::healer::resources::HealerAssets>,
        Res<super::units::aerialist::resources::AerialistAssets>,
    ),
    mut velocity_query: Query<&mut Velocity>,
) {
    let (
        infantry_assets,
        archer_assets,
        assassin_assets,
        dispeller_assets,
        undead_assets,
        king_assets,
    ) = &death_assets;
    let (shielder_assets, healer_assets, aerialist_assets) = &death_assets_2;
    for (
        entity,
        health,
        team,
        transform,
        is_infantry,
        is_archer,
        is_assassin,
        is_dispeller,
        is_shielder,
        is_healer,
        is_king,
        _is_boss,
        spell_damaged,
        (residual_fire_damaged, marked_for_death, is_aerialist, is_brute, is_flying),
    ) in &query
    {
        if health.is_dead() {
            // Record the kill
            kill_stats.record_kill(*team);

            // Send enemy killed message for multi-kill achievements
            if *team == Team::Attackers || *team == Team::Undead {
                enemy_kill_events.write(EnemyKilledMessage);
                // Notify drops system of potential ingredient drop
                drop_events.write(super::drops::messages::SpawnIngredientDropMessage {
                    position: transform.translation,
                });
                // Close Call: enemy died within CLOSE_CALL_DISTANCE of wizard
                if close_call_achievement.is_locked()
                    && let Ok(wiz_transform) = wizard_query.single()
                {
                    use super::units::wizard::archetypes::battlemage::CLOSE_CALL_DISTANCE;
                    let diff = transform.translation - wiz_transform.translation;
                    let xz_dist = (diff.x * diff.x + diff.z * diff.z).sqrt();
                    if xz_dist <= CLOSE_CALL_DISTANCE {
                        close_call_events.write(CloseCallMessage);
                    }
                }
            }

            // Track spell kills on defenders and king
            if spell_damaged.is_some() {
                if *team == Team::Defenders {
                    kill_stats.record_spell_kill_defender();
                    spell_kill_events.write(DefenderKilledBySpellMessage);
                }

                if is_king.is_some() {
                    kill_stats.record_king_killed_by_spell();
                }
            }

            // Scorched Earth: unit died from residual fire damage
            if residual_fire_damaged.is_some() {
                scorched_earth_events.write(ScorchedEarthMessage);
            }

            // Marked for Death kill: enemy died while marked by Finger of Death
            if marked_for_death.is_some() && (*team == Team::Attackers || *team == Team::Undead) {
                marked_kill_events.write(super::achievements::messages::MarkedForDeathKillMessage);
            }

            // Determine if this unit type has a death animation sprite sheet.
            // Undead infantry use undead-specific death texture.
            let death_texture = if is_dispeller.is_some() {
                Some(dispeller_assets.death_texture.clone())
            } else if is_shielder.is_some() {
                Some(shielder_assets.death_texture.clone())
            } else if is_healer.is_some() {
                Some(healer_assets.death_texture.clone())
            } else if is_infantry.is_some() {
                if *team == Team::Undead {
                    Some(undead_assets.death_texture.clone())
                } else {
                    Some(infantry_assets.death_texture.clone())
                }
            } else if is_archer.is_some() {
                Some(archer_assets.death_texture.clone())
            } else if is_assassin.is_some() {
                Some(assassin_assets.death_texture.clone())
            } else if is_aerialist.is_some() {
                Some(aerialist_assets.death_texture.clone())
            } else if is_brute.is_some() {
                Some(infantry_assets.death_texture.clone())
            } else {
                None
            };

            let mut entity_commands = commands.entity(entity);
            entity_commands.insert(Corpse);

            // Flying units drop to ground level on death
            if is_flying {
                entity_commands.remove::<Flying>();
            }

            if let Some(death_tex) = death_texture {
                // Unit has death animation: play it before laying flat
                entity_commands
                    .insert(super::units::components::DyingAnimation::new(death_tex))
                    .remove::<super::units::components::CombatAnimation>();
                // Billboard stays so sprite faces camera during death animation
            } else {
                // No death animation: instant corpse swap (king, boss, fallback)
                let idx = game_rng.0.random_range(0..CORPSE_MATERIAL_VARIANTS);

                let (mat, mesh) = if is_king.is_some() {
                    (
                        king_assets.corpse_materials[idx].clone(),
                        king_assets.sprite_mesh.clone(),
                    )
                } else {
                    // Boss and fallback use infantry corpse materials + circle mesh
                    let mat = corpse_material_for_team(
                        &infantry_assets.defender_corpse_materials,
                        &infantry_assets.attacker_corpse_materials,
                        &infantry_assets.undead_corpse_materials,
                        *team,
                        idx,
                    );
                    (mat, infantry_assets.mesh.clone())
                };

                entity_commands
                    .insert(MeshMaterial3d(mat))
                    .insert(Mesh3d(mesh));

                super::units::systems::lay_corpse_flat(&mut entity_commands, transform.translation);
            }

            // Mark undead corpses as permanent (cannot be resurrected)
            if *team == Team::Undead {
                entity_commands.insert(super::units::components::PermanentCorpse);
            }

            // Reset velocity so corpses don't continue moving
            if let Ok(mut velocity) = velocity_query.get_mut(entity) {
                velocity.x = 0.0;
                velocity.z = 0.0;
            }

            entity_commands
                .remove::<MovementSpeed>()
                .remove::<AttackTiming>()
                .remove::<Hitbox>()
                .remove::<super::units::components::CommanderAuraSpeedModifier>()
                .remove::<super::units::components::SlowMovementModifier>()
                .remove::<super::units::components::FrostAccumulation>()
                .remove::<super::units::components::RootedModifier>()
                .remove::<super::units::components::HasteModifier>()
                .remove::<super::units::components::FireDoT>()
                .remove::<super::units::components::ElectricCharge>()
                .remove::<super::units::components::PendingDamageEffect>()
                .remove::<super::units::components::OriginalMaterial>()
                .remove::<super::units::components::RoughTerrainModifier>()
                .remove::<super::units::components::MarkedForDeathModifier>()
                .remove::<super::units::components::SleepModifier>()
                .remove::<super::units::components::NightTerrors>()
                .remove::<super::units::components::Comatose>()
                .remove::<super::units::components::NarcolepticWave>()
                .remove::<super::units::components::Sleepwalking>()
                .remove::<super::units::components::BattleHymnModifier>()
                .remove::<super::units::components::EchoingSong>()
                .remove::<super::units::components::AnthemResilience>()
                .remove::<super::units::components::BerserkerRageModifier>()
                // Berserker rage talent components (FinalStand intentionally NOT removed — it fires on death)
                .remove::<super::units::wizard::spells::berserker_rage::components::Bloodlust>()
                .remove::<super::units::wizard::spells::berserker_rage::components::Frenzy>()
                .remove::<super::units::wizard::spells::berserker_rage::components::FrenzyActive>()
                .remove::<super::units::wizard::spells::berserker_rage::components::UndyingFury>()
                .remove::<super::units::wizard::spells::berserker_rage::components::UndyingFuryActive>()
                .remove::<super::units::wizard::spells::berserker_rage::components::ContagiousRage>()
                .remove::<super::units::components::FogEvasionModifier>()
                .remove::<super::units::wizard::spells::fog_cloud::components::BlindingMistDebuff>()
                .remove::<super::units::components::FrozenSolidModifier>()
                .remove::<super::units::components::Stunned>()
                .remove::<super::units::wizard::spells::teleport::components::DisorientingHaste>()
                .remove::<super::units::components::BanishedModifier>()
                .remove::<super::units::components::PolymorphedModifier>()
                .remove::<super::units::components::PoisonedModifier>()
                .remove::<super::units::components::SickenedModifier>()
                .remove::<super::units::components::SmellyModifier>()
                .remove::<super::units::wizard::spells::mind_control::components::TraitorsMarkAura>()
                .remove::<super::units::wizard::spells::mind_control::components::Demoralized>()
                .remove::<super::units::wizard::spells::mind_control::components::AmnesiaOnExpiry>()
                .remove::<super::units::wizard::spells::mind_control::components::AmnesiaEffect>()
                .remove::<super::units::wizard::spells::mind_control::components::DominatedUnit>()
                .remove::<super::units::wizard::spells::mind_control::components::MassHysteriaTarget>()
                .remove::<super::units::wizard::spells::mind_control::components::SleeperAgentPending>()
                .remove::<super::units::wizard::spells::mind_control::components::SleeperAgentActive>()
                .remove::<super::units::wizard::spells::haste::components::MomentumBuff>()
                .remove::<super::units::wizard::spells::haste::components::MomentumPending>()
                .remove::<super::units::wizard::spells::haste::components::FleetFeet>()
                .remove::<super::units::wizard::spells::haste::components::ChainHasteSource>()
                .remove::<super::units::wizard::spells::healing_plume::components::FieldMedicConverted>()
                .remove::<CauldronDamageBonus>()
                .remove::<CauldronDamageResistance>()
                .remove::<CauldronSpeedModifier>()
                .remove::<super::units::components::WalkingAnimation>()
                .remove::<super::units::components::FacingDirection>()
                .remove::<super::units::king::components::SpellShield>()
                .remove::<super::units::shielder::components::ShielderDamageReduction>();
        }
    }
}
