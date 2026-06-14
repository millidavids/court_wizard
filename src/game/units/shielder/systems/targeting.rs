use bevy::prelude::*;

use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::{
    BanishedModifier, Corpse, MindControlled, TargetingVelocity, Team,
};
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::units::king::components::SpellShield;
use crate::game::units::shielder::components::Shielder;
use crate::game::units::shielder::constants::SHIELD_RANGE;
use crate::game::units::wizard::components::Wizard;

/// Finds the nearest unshielded ally within shield range, if any.
pub(super) fn find_shielder_target(
    ally_snapshot: &[(Entity, Vec3, Team, bool)],
    self_pos: Vec3,
    self_team: Team,
) -> Option<Entity> {
    ally_snapshot
        .iter()
        .filter(|(_, _, ally_team, has_shield)| *ally_team == self_team && !has_shield)
        .filter_map(|&(ally_entity, ally_pos, _, _)| {
            let dist =
                ((self_pos.x - ally_pos.x).powi(2) + (self_pos.z - ally_pos.z).powi(2)).sqrt();
            if dist <= SHIELD_RANGE {
                Some((ally_entity, dist))
            } else {
                None
            }
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(e, _)| e)
}

/// Updates shielder targeting — seeks nearest same-team ally without a spell shield,
/// or falls back to following the army toward nearest enemy.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_shielder_targeting(
    defenders_activated: Res<DefendersActivated>,
    mut commands: Commands,
    mut shielders: Query<
        (Entity, &Transform, &Team, &mut TargetingVelocity),
        (With<Shielder>, Without<Corpse>, Without<MindControlled>),
    >,
    potential_targets: Query<
        (Entity, &Transform, &Team, Has<SpellShield>),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<Shielder>,
            Without<Wizard>,
        ),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
            Without<Wizard>,
        ),
    >,
) {
    // Snapshot ally data for shield targeting
    let ally_snapshot: Vec<(Entity, Vec3, Team, bool)> = potential_targets
        .iter()
        .map(|(entity, transform, team, has_shield)| {
            (entity, transform.translation, *team, has_shield)
        })
        .collect();

    // Collect unit snapshot for enemy targeting fallback
    let unit_snapshot: Vec<(Entity, Vec3, Team)> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, transform, team, mut targeting_velocity) in &mut shielders {
        // Skip inactive defender shielders
        if *team == Team::Defenders && !defenders_activated.active {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
            continue;
        }

        // Priority 1: Find nearest same-team ally without a spell shield
        let best_target = ally_snapshot
            .iter()
            .filter(|(ally_entity, _, ally_team, has_shield)| {
                *ally_entity != entity && *ally_team == *team && !has_shield
            })
            .min_by(|a, b| {
                let dist_a = (transform.translation.x - a.1.x).powi(2)
                    + (transform.translation.z - a.1.z).powi(2);
                let dist_b = (transform.translation.x - b.1.x).powi(2)
                    + (transform.translation.z - b.1.z).powi(2);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(&(_, ally_pos, _, _)) = best_target {
            let diff = ally_pos - transform.translation;
            let distance = (diff.x.powi(2) + diff.z.powi(2)).sqrt();
            targeting_velocity.distance_to_target = distance;

            if distance <= SHIELD_RANGE {
                // In shield range — stop moving
                targeting_velocity.velocity = Vec3::ZERO;
            } else {
                // Move toward unshielded ally
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            }

            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
            continue;
        }

        // Priority 2: Fall back to following army toward nearest enemy
        let nearest_enemy = unit_snapshot
            .iter()
            .filter(|(other_entity, _, other_teams)| {
                *other_entity != entity && team.is_enemy(other_teams)
            })
            .min_by(|a, b| {
                let dist_a = (transform.translation.x - a.1.x).powi(2)
                    + (transform.translation.z - a.1.z).powi(2);
                let dist_b = (transform.translation.x - b.1.x).powi(2)
                    + (transform.translation.z - b.1.z).powi(2);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(&(_, target_pos, _)) = nearest_enemy {
            let diff = target_pos - transform.translation;
            let distance = (diff.x.powi(2) + diff.z.powi(2)).sqrt();
            targeting_velocity.distance_to_target = distance;

            // Stay at shield range from enemies
            if distance <= SHIELD_RANGE {
                targeting_velocity.velocity = Vec3::ZERO;
            } else {
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            }
        } else {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
        }

        commands
            .entity(entity)
            .remove::<crate::game::units::components::InMelee>();
    }
}
