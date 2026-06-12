use super::super::combat::wall_near_approach_path;
use crate::game::constants::MELEE_SLOWDOWN_DISTANCE;
use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::{BanishedModifier, Corpse, Team};
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;

/// Sets the targeting velocity for archers, steering them toward or away from enemies.
pub fn update_archer_targeting(
    defenders_activated: Res<DefendersActivated>,
    mut commands: Commands,
    mut archers: Query<
        (
            Entity,
            &Transform,
            &Team,
            &AttackRange,
            &mut crate::game::units::components::TargetingVelocity,
        ),
        (
            With<Archer>,
            Without<Corpse>,
            Without<crate::game::units::components::MindControlled>,
        ),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
    walls: Query<&WallOfStone>,
    rocks_query2: Query<&crate::game::terrain::boulder::components::Boulder>,
    trees_query2: Query<&crate::game::terrain::tree::components::Tree>,
) {
    // Collect snapshot of all unit positions (excludes staging attackers)
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    // Collect wall, rock, and tree snapshots for line-of-sight checks
    let wall_snapshot: Vec<_> = walls.iter().collect();
    let rock_snapshot: Vec<_> = rocks_query2.iter().filter(|r| !r.sinking).collect();
    let tree_snapshot: Vec<_> = trees_query2.iter().collect();

    // Update each archer's targeting velocity
    for (entity, transform, team, attack_range, mut targeting_velocity) in &mut archers {
        // Skip inactive defender archers (but always process attackers)
        if *team == Team::Defenders && !defenders_activated.active {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
            continue;
        }

        let pos = transform.translation;

        // Find nearest enemy in seek zone [min_range, seek_range]
        // Archers advance until enemies are within seek range, then stop.
        // They can still shoot up to max_range, but won't stop that far out.
        // Only count targets with clear line-of-sight (no walls blocking).
        let ranged_target = unit_snapshot
            .iter()
            .filter(|(other_entity, _, other_team)| {
                *other_entity != entity && team.is_enemy(other_team)
            })
            .filter_map(|&(_, target_pos, _)| {
                let dx = pos.x - target_pos.x;
                let dz = pos.z - target_pos.z;
                let dist = (dx * dx + dz * dz).sqrt();
                if dist >= attack_range.min_range && dist <= ARCHER_SEEK_RANGE {
                    // Check line-of-sight: skip if any wall, rock, or tree blocks the shot
                    if WallOfStone::any_blocks_los(&wall_snapshot, pos, target_pos)
                        || crate::game::terrain::boulder::components::Boulder::any_blocks_los(
                            &rock_snapshot,
                            pos,
                            target_pos,
                        )
                        || crate::game::terrain::tree::components::Tree::any_blocks_los(
                            &tree_snapshot,
                            pos,
                            target_pos,
                        )
                    {
                        None
                    } else {
                        Some((dist, target_pos))
                    }
                } else {
                    None
                }
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Find nearest enemy overall (fallback for melee or advancing)
        let nearest_enemy = unit_snapshot
            .iter()
            .filter(|(other_entity, _, other_team)| {
                *other_entity != entity && team.is_enemy(other_team)
            })
            .min_by(|a, b| {
                let dist_a = (pos.x - a.1.x).powi(2) + (pos.z - a.1.z).powi(2);
                let dist_b = (pos.x - b.1.x).powi(2) + (pos.z - b.1.z).powi(2);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        // Prefer ranged targets — only fall back to melee/advance if none in range
        if let Some((ranged_dist, target_pos)) = ranged_target {
            // If a wall is near the approach path (even if it doesn't block direct
            // LOS), keep advancing so the flow field routes the archer around it.
            // Otherwise enter shooting stance.
            targeting_velocity.velocity =
                if wall_near_approach_path(&wall_snapshot, pos, target_pos) {
                    Vec3::new(target_pos.x - pos.x, 0.0, target_pos.z - pos.z).normalize_or_zero()
                } else {
                    Vec3::ZERO
                };
            targeting_velocity.distance_to_target = ranged_dist;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
        } else if let Some(&(_, target_pos, enemy_team)) = nearest_enemy {
            let diff = target_pos - pos;
            let distance = (diff.x.powi(2) + diff.z.powi(2)).sqrt();
            targeting_velocity.distance_to_target = distance;

            let in_melee_range = distance < MELEE_SLOWDOWN_DISTANCE;
            if in_melee_range {
                commands
                    .entity(entity)
                    .insert(crate::game::units::components::InMelee(enemy_team));
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            } else {
                commands
                    .entity(entity)
                    .remove::<crate::game::units::components::InMelee>();
                // Beyond max range — advance toward enemy
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            }
        } else {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
        }
    }
}
