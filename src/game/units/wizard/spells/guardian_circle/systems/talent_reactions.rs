use super::super::components::GuardianCircleShielded;
use crate::game::units::components::{Corpse, Health, Team, TemporaryHitPoints};
use crate::game::units::wizard::components::Wizard;
use bevy::prelude::*;

use super::buff::deal_aoe_force_damage;

/// Tier 2, Choice 0: Retaliating Wards.
///
/// When a unit's temp HP is fully broken (amount reaches 0 but component still exists),
/// deal AoE force damage to nearby enemies. Fires once then removes the marker.
pub fn retaliating_wards_check(
    mut commands: Commands,
    shielded_query: Query<(
        Entity,
        &GuardianCircleShielded,
        &TemporaryHitPoints,
        &Transform,
        &Team,
    )>,
    mut targets: Query<
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
    for (entity, shielded, temp_hp, transform, team) in &shielded_query {
        if shielded.retaliating_damage <= 0.0 || temp_hp.amount > 0.0 {
            continue;
        }

        deal_aoe_force_damage(
            &mut commands,
            transform.translation,
            shielded.retaliating_radius,
            shielded.retaliating_damage,
            team,
            &mut targets,
        );

        // One-shot: remove marker so retaliation doesn't fire again
        commands.entity(entity).remove::<GuardianCircleShielded>();
    }
}

/// Tier 3, Choice 1: Martyrdom.
///
/// When a shielded unit dies, the stored shield amount explodes as AoE damage.
/// Damage is the full temp HP granted at cast time (not what remained at death).
/// Fires once then removes the marker from the corpse.
pub fn martyrdom_on_death(
    mut commands: Commands,
    dead_query: Query<(Entity, &GuardianCircleShielded, &Transform, &Team), With<Corpse>>,
    mut targets: Query<
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
    for (corpse_entity, shielded, transform, team) in &dead_query {
        if shielded.martyrdom_damage <= 0.0 {
            continue;
        }

        deal_aoe_force_damage(
            &mut commands,
            transform.translation,
            shielded.martyrdom_radius,
            shielded.martyrdom_damage,
            team,
            &mut targets,
        );

        // One-shot: remove marker so martyrdom doesn't fire again
        commands
            .entity(corpse_entity)
            .remove::<GuardianCircleShielded>();
    }
}

/// Tier 3, Choice 2: Chain Ward.
///
/// When a shielded unit dies, its temp HP jumps to the nearest unshielded ally.
/// Fires once then removes the marker from the corpse.
pub fn chain_ward_on_death(
    mut commands: Commands,
    dead_query: Query<(Entity, &GuardianCircleShielded, &Transform, &Team), With<Corpse>>,
    alive_query: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<GuardianCircleShielded>,
            Without<Wizard>,
        ),
    >,
) {
    for (corpse_entity, shielded, transform, shielded_team) in &dead_query {
        if shielded.chain_ward_hops == 0 || shielded.chain_ward_amount <= 0.0 {
            continue;
        }

        // Find the nearest allied unit without a shield
        let mut nearest: Option<(Entity, f32)> = None;
        for (candidate, candidate_transform, candidate_team) in &alive_query {
            if *candidate_team != *shielded_team {
                continue;
            }
            let dist = candidate_transform
                .translation
                .distance(transform.translation);
            if nearest.is_none_or(|(_, d)| dist < d) {
                nearest = Some((candidate, dist));
            }
        }

        if let Some((target_entity, _)) = nearest {
            commands
                .entity(target_entity)
                .try_insert(TemporaryHitPoints::new(
                    shielded.chain_ward_amount,
                    shielded.chain_ward_duration,
                ));

            // Clone and pass along with one fewer hop
            let mut chained = shielded.clone();
            chained.chain_ward_hops -= 1;
            commands.entity(target_entity).try_insert(chained);
        }

        // One-shot: remove marker so chain ward doesn't fire again
        commands
            .entity(corpse_entity)
            .remove::<GuardianCircleShielded>();
    }
}
