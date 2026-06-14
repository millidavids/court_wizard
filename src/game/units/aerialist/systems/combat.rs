use bevy::prelude::*;

use super::super::components::Aerialist;
use super::super::constants::AERIALIST_ATTACK_RANGE;
use crate::game::attack_cycle::GlobalAttackCycle;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, Corpse, Flying, FrozenSolidModifier, MindControlled,
    SleepModifier, TargetingVelocity, Team,
};
use crate::game::units::wizard::components::Wizard;

/// Aerialist targeting: find nearest ground enemy and set targeting velocity.
/// Flying units ignore wall LOS since they fly above obstacles.
#[allow(clippy::type_complexity)]
pub(crate) fn update_aerialist_targeting(
    mut aerialists: Query<
        (Entity, &Transform, &Team, &mut TargetingVelocity),
        (With<Aerialist>, Without<Corpse>, Without<MindControlled>),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<Flying>,
            Without<crate::game::pathfinding::StagingAttacker>,
            Without<Wizard>,
        ),
    >,
) {
    // Collect snapshot of ground unit positions (excludes flying and staging)
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, transform, team, mut targeting_velocity) in &mut aerialists {
        let nearest_enemy = unit_snapshot
            .iter()
            .filter(|(other_entity, _, other_team)| {
                *other_entity != entity && team.is_enemy(other_team)
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

        if let Some(&(_, target_pos, _enemy_team)) = nearest_enemy {
            let direction = (target_pos - transform.translation).normalize_or_zero();
            targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);

            let dx = transform.translation.x - target_pos.x;
            let dz = transform.translation.z - target_pos.z;
            let distance = (dx * dx + dz * dz).sqrt();
            targeting_velocity.distance_to_target = distance;

            // Flying units are never in melee — they attack from above.
            // Keeping them out of InMelee ensures archers can always target them.
        } else {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
        }
    }
}

/// Aerialist combat: direct damage to ground enemies within very short range.
#[allow(clippy::type_complexity)]
pub(crate) fn aerialist_combat(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    attack_cycle: Res<GlobalAttackCycle>,
    archer_assets: Res<crate::game::units::archer::resources::ArcherAssets>,
    mut aerialists: Query<
        (
            &Transform,
            &Team,
            &mut AttackTiming,
            Has<SleepModifier>,
            Option<&BanishedModifier>,
            Option<&FrozenSolidModifier>,
            Option<&crate::game::units::components::Stunned>,
        ),
        (With<Aerialist>, Without<Corpse>),
    >,
    targets: Query<
        (Entity, &Transform, &Team),
        (Without<Corpse>, Without<Flying>, Without<Wizard>),
    >,
) {
    let current_time = attack_cycle.current_time;
    // `previous_time` is the cycle position before the most recent `tick()` —
    // see `GlobalAttackCycle` doc for why this is preferred over the
    // `current_time - APPROX_FRAME_TIME` approximation.
    let last_time = attack_cycle.previous_time;

    for (
        aerialist_transform,
        aerialist_team,
        mut attack_timing,
        is_sleeping,
        banished,
        frozen,
        stunned,
    ) in &mut aerialists
    {
        if is_sleeping || banished.is_some() || frozen.is_some() || stunned.is_some() {
            continue;
        }

        if !attack_timing.can_attack(current_time, last_time) {
            continue;
        }

        // Find nearest enemy within attack range (full 3D distance — accounts for fly height)
        let nearest = targets
            .iter()
            .filter(|(_, _, team)| aerialist_team.is_enemy(team))
            .filter_map(|(entity, target_transform, _)| {
                let distance = aerialist_transform
                    .translation
                    .distance(target_transform.translation);
                if distance <= AERIALIST_ATTACK_RANGE {
                    Some((entity, target_transform.translation, distance))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((_target_entity, target_pos, _)) = nearest {
            attack_timing.last_attack_time = Some(current_time);

            // Spawn an arrow projectile toward the target
            crate::game::units::archer::systems::spawn_arrow(
                &mut game_rng.0,
                &mut commands,
                &archer_assets,
                aerialist_transform.translation,
                target_pos,
                *aerialist_team,
            );
        }
    }
}
