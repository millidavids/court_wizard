use super::super::components::{
    InsidePlagueCloud, PlagueCarrierDoT, PlagueWindCloud, ToxicWeaknessDebuff,
};
use super::super::constants;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Health, SlowMovementModifier, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::utils::{UniqueHitTracker, local_player_team};
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;
use bevy::prelude::*;

fn horizontal_distance(a: Vec3, b: Vec3) -> f32 {
    Vec2::new(a.x - b.x, a.z - b.z).length()
}

/// Applies periodic necrotic damage to all units within the cloud.
/// Handles Toxic Weakness (vulnerability), Choking Gas (slow), Necrotic Rot (max HP reduction),
/// and tracks units inside cloud for Plague Carrier.
pub fn apply_plague_wind_damage(
    mut commands: Commands,
    time: Res<Time>,
    // Host-only — ghost cloud on the guest must NOT also apply DPS, or every
    // tick deals damage on both peers and CRDT max-merge masks the doubling
    // but talent status effects (Toxic Weakness, Choking Gas) get applied
    // twice locally.
    mut clouds: Query<
        (&mut PlagueWindCloud, &mut UniqueHitTracker),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut units: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        Has<InsidePlagueCloud>,
        &Team,
    )>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    let delta = time.delta_secs();
    let mut unique_hits: u32 = 0;

    for (mut cloud, mut hit_tracker) in &mut clouds {
        cloud.time_alive += delta;
        cloud.time_since_last_tick += delta;

        let has_plague_carrier = cloud.talent_params.plague_carrier;
        let has_toxic_weakness = cloud.talent_params.toxic_weakness;
        let has_choking_gas = cloud.talent_params.choking_gas;
        let has_necrotic_rot = cloud.talent_params.necrotic_rot;

        let should_tick = cloud.time_since_last_tick >= cloud.tick_interval;
        if should_tick {
            cloud.time_since_last_tick = 0.0;
        }

        // Skip unit iteration if nothing to do this frame
        if !should_tick && !has_plague_carrier {
            continue;
        }

        for (entity, transform, mut health, mut temp_hp, has_spell_shield, already_marked, team) in
            &mut units
        {
            let inside = horizontal_distance(cloud.origin, transform.translation) <= cloud.radius;

            if inside {
                // Mark unit as inside cloud (for Plague Carrier tracking), skip if already marked
                if has_plague_carrier && !already_marked {
                    commands.entity(entity).insert(InsidePlagueCloud);
                }

                if should_tick {
                    // Toxic Weakness: additive vulnerability while inside cloud
                    if has_toxic_weakness {
                        health.spell_vulnerability += constants::TOXIC_WEAKNESS_VULNERABILITY;
                        commands
                            .entity(entity)
                            .insert(ToxicWeaknessDebuff(constants::TOXIC_WEAKNESS_VULNERABILITY));
                    }

                    apply_spell_damage_with_team(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        cloud.damage_per_tick,
                        DamageType::Poison,
                        has_spell_shield,
                        caster_team,
                        *team,
                    );
                    if hit_tracker.track_hit(entity) {
                        unique_hits += 1;
                    }

                    // Necrotic Rot: reduce max HP by the damage dealt
                    if has_necrotic_rot {
                        let max_hp_reduction = cloud.damage_per_tick
                            * constants::NECROTIC_ROT_MAX_HP_REDUCTION_FRACTION;
                        health.max = (health.max - max_hp_reduction).max(1.0);
                        health.current = health.current.min(health.max);
                    }

                    // Choking Gas: slow enemies inside
                    if has_choking_gas {
                        commands.entity(entity).insert(SlowMovementModifier::new(
                            constants::CHOKING_GAS_SLOW,
                            constants::CHOKING_GAS_SLOW_DURATION,
                        ));
                    }
                }
            }
        }
    }

    if unique_hits > 0
        && let Some(ref mut progress) = talent_progress
    {
        progress.increment(Spell::PlagueWind, unique_hits);
    }
}

/// Removes Toxic Weakness vulnerability from units no longer in any cloud with the talent.
pub fn cleanup_toxic_weakness(
    mut commands: Commands,
    clouds: Query<&PlagueWindCloud>,
    mut debuffed_units: Query<(Entity, &Transform, &ToxicWeaknessDebuff, &mut Health)>,
) {
    for (entity, transform, debuff, mut health) in &mut debuffed_units {
        let still_inside = clouds.iter().any(|cloud| {
            cloud.talent_params.toxic_weakness
                && horizontal_distance(cloud.origin, transform.translation) <= cloud.radius
        });

        if !still_inside {
            health.spell_vulnerability = (health.spell_vulnerability - debuff.0).max(0.0);
            commands.entity(entity).remove::<ToxicWeaknessDebuff>();
        }
    }
}

/// Removes InsidePlagueCloud marker from units no longer in any cloud,
/// and applies Plague Carrier lingering DoT when they leave.
pub fn track_plague_carrier(
    mut commands: Commands,
    clouds: Query<&PlagueWindCloud>,
    marked_units: Query<(Entity, &Transform), With<InsidePlagueCloud>>,
) {
    for (entity, transform) in &marked_units {
        let mut still_inside = false;
        let mut carrier_damage = 0.0_f32;

        for cloud in &clouds {
            if !cloud.talent_params.plague_carrier {
                continue;
            }

            if horizontal_distance(cloud.origin, transform.translation) <= cloud.radius {
                still_inside = true;
                break;
            }
            // Track highest damage cloud for the lingering DoT
            carrier_damage = carrier_damage
                .max(cloud.damage_per_tick * constants::PLAGUE_CARRIER_DAMAGE_FRACTION);
        }

        if !still_inside {
            commands.entity(entity).remove::<InsidePlagueCloud>();

            if carrier_damage > 0.0 {
                commands.entity(entity).insert(PlagueCarrierDoT::new(
                    carrier_damage,
                    constants::PLAGUE_CARRIER_TICK_INTERVAL,
                    constants::PLAGUE_CARRIER_DURATION,
                ));
            }
        }
    }
}

/// Applies lingering Plague Carrier DoT damage and cleans up expired DoTs.
pub fn apply_plague_carrier_dot(
    mut commands: Commands,
    time: Res<Time>,
    mut dot_units: Query<(
        Entity,
        &mut PlagueCarrierDoT,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    let delta = time.delta_secs();

    for (entity, mut dot, mut health, mut temp_hp, has_spell_shield, team) in &mut dot_units {
        dot.time_remaining -= delta;
        dot.time_since_last_tick += delta;

        if dot.time_remaining <= 0.0 {
            commands.entity(entity).remove::<PlagueCarrierDoT>();
            continue;
        }

        if dot.time_since_last_tick >= dot.tick_interval {
            dot.time_since_last_tick = 0.0;
            apply_spell_damage_with_team(
                &mut commands,
                entity,
                &mut health,
                temp_hp.as_deref_mut(),
                dot.damage_per_tick,
                DamageType::Poison,
                has_spell_shield,
                caster_team,
                *team,
            );
        }
    }
}
