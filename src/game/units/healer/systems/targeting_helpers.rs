use bevy::prelude::*;

use crate::game::units::components::Team;

/// Returns a priority score for heal targeting.
/// Commander=5, Brute=4, Elite=3, Dispeller=2, Healer=1, regular=0.
pub(crate) fn find_heal_priority(
    is_commander: bool,
    is_brute: bool,
    is_elite: bool,
    is_dispeller: bool,
    is_healer: bool,
) -> u32 {
    if is_commander {
        5
    } else if is_brute {
        4
    } else if is_elite {
        3
    } else if is_dispeller {
        2
    } else if is_healer {
        1
    } else {
        0
    }
}

/// Finds the best heal target from a snapshot of allies.
/// Prioritizes by unit type priority, then by lowest HP percentage within same priority.
/// Only considers hurt allies (current < max) on the same team, excluding self.
/// Returns (entity, position, distance) if a target is found.
pub(crate) fn find_best_heal_target(
    ally_snapshot: &[(Entity, Vec3, Team, f32, f32, u32)],
    self_entity: Entity,
    self_pos: Vec3,
    self_team: Team,
) -> Option<(Entity, Vec3, f32)> {
    ally_snapshot
        .iter()
        .filter(|(entity, _, team, current, max, _)| {
            *entity != self_entity && *team == self_team && *current < *max
        })
        .max_by(|a, b| {
            // Higher priority first
            let priority_cmp = a.5.cmp(&b.5);
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }
            // Within same priority, lower HP percentage is better (more hurt = higher priority)
            let hp_pct_a = a.3 / a.4;
            let hp_pct_b = b.3 / b.4;
            hp_pct_b
                .partial_cmp(&hp_pct_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, pos, _, _, _, _)| {
            let distance = ((self_pos.x - pos.x).powi(2) + (self_pos.z - pos.z).powi(2)).sqrt();
            (*entity, *pos, distance)
        })
}
