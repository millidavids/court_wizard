use super::super::components::GuardianCircleShielded;
use super::super::constants;
use crate::game::achievements::messages::GuardianCircleHitAttackerMessage;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::wizard::components::{Spell, Wizard};
use crate::game::units::wizard::talents::resources::ActiveTalents;
use bevy::prelude::*;

/// Helper function to apply Guardian Circle buff to all units in radius.
///
/// Grants temporary HP to units with talent modifications applied.
/// Also inserts GuardianCircleShielded marker for Tier 2/3 talent effects.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_guardian_circle_buff(
    commands: &mut Commands,
    circle_pos: Vec3,
    radius: f32,
    temp_hp_amount: f32,
    duration: f32,
    empowerment: f32,
    targets: &mut Query<(Entity, &Transform, &Team), Without<Wizard>>,
    attacker_hit_msg: &mut MessageWriter<GuardianCircleHitAttackerMessage>,
    talent_progress: &mut Option<
        ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
    active_talents: Option<&ActiveTalents>,
) {
    let t1 = active_talents.and_then(|t| t.get_selection(Spell::GuardianCircle, 0));
    let t2 = active_talents.and_then(|t| t.get_selection(Spell::GuardianCircle, 1));
    let t3 = active_talents.and_then(|t| t.get_selection(Spell::GuardianCircle, 2));

    // Scale values by empowerment
    let scale = empowerment;
    let mut scaled_temp_hp = temp_hp_amount * scale;
    let mut scaled_duration = duration * scale;

    // Tier 1 modifications
    match t1 {
        Some(0) => scaled_temp_hp *= constants::REINFORCED_WARDS_MULT, // +40% temp HP
        Some(1) => scaled_duration *= constants::ENDURING_PROTECTION_MULT, // +60% duration
        Some(2) => scaled_temp_hp *= constants::EXPANSIVE_AEGIS_HP_MULT, // -15% temp HP
        _ => {}
    }

    // Build the GuardianCircleShielded component based on T2/T3 selections
    let has_talent_effects = t2.is_some() || t3.is_some();
    let shielded = if has_talent_effects {
        let mut s = GuardianCircleShielded::default();

        // Tier 2
        match t2 {
            Some(0) => {
                // Retaliating Wards
                s.retaliating_damage = constants::RETALIATING_WARDS_DAMAGE * scale;
                s.retaliating_radius = constants::RETALIATING_WARDS_RADIUS;
            }
            Some(1) => {
                // Fortified Resolve
                s.fortified_damage_bonus = constants::FORTIFIED_RESOLVE_DAMAGE_MULT;
            }
            // Rapid Deployment is handled in casting, not here
            _ => {}
        }

        // Tier 3
        match t3 {
            Some(0) => {
                // Sanctuary
                s.sanctuary_reduction = constants::SANCTUARY_DAMAGE_REDUCTION;
            }
            Some(1) => {
                // Martyrdom — store the granted temp HP as explosion damage
                s.martyrdom_damage = scaled_temp_hp;
                s.martyrdom_radius = constants::MARTYRDOM_DAMAGE_RADIUS;
            }
            Some(2) => {
                // Chain Ward
                s.chain_ward_hops = constants::CHAIN_WARD_MAX_HOPS;
                s.chain_ward_amount = scaled_temp_hp;
                s.chain_ward_duration = scaled_duration;
            }
            _ => {}
        }

        Some(s)
    } else {
        None
    };

    let mut buffed_count = 0u32;
    for (entity, transform, team) in targets.iter() {
        let distance = transform.translation.distance(circle_pos);

        if distance <= radius {
            // Unit is in range - add or update TemporaryHitPoints
            commands
                .entity(entity)
                .insert(TemporaryHitPoints::new(scaled_temp_hp, scaled_duration));

            // Insert talent marker if any T2/T3 talents are active
            if let Some(ref s) = shielded {
                commands.entity(entity).insert(s.clone());
            }

            // Protective Instincts: Guardian Circle hit an attacker or undead
            if *team == Team::Attackers || *team == Team::Undead {
                attacker_hit_msg.write(GuardianCircleHitAttackerMessage);
            }

            buffed_count += 1;
        }
    }

    if buffed_count > 0
        && let Some(progress) = talent_progress.as_deref_mut()
    {
        progress.increment(Spell::GuardianCircle, buffed_count);
    }
}

/// Cleanup system: remove GuardianCircleShielded when temp HP expires or is removed.
pub fn cleanup_guardian_circle_shielded(
    mut commands: Commands,
    query: Query<Entity, (With<GuardianCircleShielded>, Without<TemporaryHitPoints>)>,
) {
    for entity in &query {
        commands.entity(entity).remove::<GuardianCircleShielded>();
    }
}

/// Deals AoE force damage to enemies within radius of a position.
pub(crate) fn deal_aoe_force_damage(
    commands: &mut Commands,
    origin: Vec3,
    radius: f32,
    damage: f32,
    source_team: &Team,
    targets: &mut Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (Without<Corpse>, Without<GuardianCircleShielded>),
    >,
) {
    for (entity, transform, team, mut health, temp_hp) in targets.iter_mut() {
        if *team == *source_team {
            continue;
        }
        if transform.translation.distance(origin) <= radius {
            apply_spell_damage(
                commands,
                entity,
                &mut health,
                temp_hp.map(|t| t.into_inner()),
                damage,
                DamageType::Force,
                false,
            );
        }
    }
}
