use bevy::prelude::*;
use std::collections::HashMap;

use super::super::constants::*;
use super::super::units::components::{Corpse, Effectiveness, Hitbox, SpellDamaged, Team};

/// Minimal per-unit data for the melee-proximity calculation.
#[derive(Clone, Copy)]
pub(crate) struct ProxUnit {
    pos: Vec3,
    radius: f32,
    team: Team,
}

/// Counts allies/enemies within melee range of each unit, returning `(ally, enemy)`
/// per unit in the SAME order as the input slice.
///
/// Uses a uniform spatial grid so each unit only tests its 3×3 cell neighbourhood
/// instead of every other unit (O(n·k) vs O(n²)). The cell size is the maximum
/// possible interaction distance — `(2·max_radius)·ATTACK_RANGE_MULTIPLIER` — so any
/// pair that satisfies the per-pair `(rA+rB)·MULT` test is guaranteed to fall in the
/// same or an adjacent cell. The per-pair test itself is identical to the brute-force
/// version, so the counts are exactly equal (see the equivalence test below).
fn count_melee_proximity(units: &[ProxUnit]) -> Vec<(u32, u32)> {
    let mut counts = vec![(0u32, 0u32); units.len()];
    if units.is_empty() {
        return counts;
    }

    let max_radius = units.iter().map(|u| u.radius).fold(0.0f32, f32::max);
    // Max interaction distance; guard against a zero cell size.
    let cell = ((2.0 * max_radius) * ATTACK_RANGE_MULTIPLIER).max(1.0);
    let key = |p: Vec3| ((p.x / cell).floor() as i32, (p.z / cell).floor() as i32);

    let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    for (i, u) in units.iter().enumerate() {
        grid.entry(key(u.pos)).or_default().push(i);
    }

    for (i, u) in units.iter().enumerate() {
        let (cx, cz) = key(u.pos);
        let (mut ally, mut enemy) = (0u32, 0u32);
        for gx in cx - 1..=cx + 1 {
            for gz in cz - 1..=cz + 1 {
                let Some(bucket) = grid.get(&(gx, gz)) else {
                    continue;
                };
                for &j in bucket {
                    if j == i {
                        continue;
                    }
                    let o = &units[j];
                    let dx = u.pos.x - o.pos.x;
                    let dz = u.pos.z - o.pos.z;
                    let distance = (dx * dx + dz * dz).sqrt();
                    let melee_range = (u.radius + o.radius) * ATTACK_RANGE_MULTIPLIER;
                    if distance <= melee_range {
                        if u.team.is_enemy(&o.team) {
                            enemy += 1;
                        } else {
                            ally += 1;
                        }
                    }
                }
            }
        }
        counts[i] = (ally, enemy);
    }
    counts
}

/// Calculates effectiveness for all units based on melee proximity.
///
/// Effectiveness is modified by:
/// - Number of allies in melee range (positive effect: +10% per ally)
/// - Number of enemies in melee range (negative effect: -15% per enemy)
///
/// The effectiveness coefficient is applied to attack damage in the combat system.
/// This encourages tactical positioning and rewards units that fight together
/// while penalizing isolated units.
pub fn calculate_effectiveness(
    mut units: Query<
        (Entity, &Transform, &Hitbox, &Team, &mut Effectiveness),
        (
            Without<Corpse>,
            Without<super::super::units::boss::components::Boss>,
        ),
    >,
    // Reused across frames so this every-frame hot path doesn't heap-allocate the
    // snapshot/entity buffers and the rejoin map each tick.
    mut entities: Local<Vec<Entity>>,
    mut snapshot: Local<Vec<ProxUnit>>,
    mut by_entity: Local<HashMap<Entity, (u32, u32)>>,
) {
    // Collect a snapshot (parallel to the entity list) and count proximity via the
    // spatial grid, then map each unit's counts back by Entity for the write pass.
    entities.clear();
    snapshot.clear();
    for (entity, transform, hitbox, team, _) in units.iter() {
        entities.push(entity);
        snapshot.push(ProxUnit {
            pos: transform.translation,
            radius: hitbox.radius,
            team: *team,
        });
    }

    let counts = count_melee_proximity(&snapshot);
    by_entity.clear();
    by_entity.extend(entities.iter().copied().zip(counts));

    for (entity, _, _, _, mut effectiveness) in units.iter_mut() {
        let (ally_count, enemy_count) = by_entity.get(&entity).copied().unwrap_or((0, 0));
        effectiveness.recalculate(ally_count as i32, enemy_count as i32);
    }
}

/// Activates all defenders when any defender is close enough to an enemy.
///
/// This creates coordinated defensive behavior - the entire defensive line
/// engages together rather than individually.
pub fn activate_defenders_on_proximity(
    mut defenders_activated: ResMut<super::super::units::infantry::components::DefendersActivated>,
    retreat_state: Res<super::super::units::infantry::components::RetreatState>,
    defenders: Query<(&Transform, &Team), Without<Corpse>>,
    all_units: Query<(&Transform, &Team), Without<Corpse>>,
) {
    // During retreat, defenders are force-deactivated — skip activation
    if retreat_state.is_active() {
        return;
    }

    // If already active, stay active (defenders don't deactivate once engaged)
    if defenders_activated.active {
        return;
    }

    // Check if any defender is close to any enemy
    for (defender_transform, defender_team) in &defenders {
        // Skip non-defenders
        if *defender_team != Team::Defenders {
            continue;
        }

        // Check distance to nearest enemy
        for (enemy_transform, enemy_team) in &all_units {
            // Skip same team
            if *enemy_team == *defender_team {
                continue;
            }

            let is_enemy = defender_team.is_enemy(enemy_team);

            if !is_enemy {
                continue;
            }

            let distance = defender_transform
                .translation
                .distance(enemy_transform.translation);
            if distance < DEFENDER_ACTIVATION_RANGE {
                // Enemy in range - activate all defenders
                defenders_activated.active = true;
                return;
            }
        }
    }
}

/// Tracks when any wizard spell damages an attacker or undead unit.
/// Uses `Added<SpellDamaged>` to detect newly damaged entities this frame.
/// Gated by `run_if` so it stops running once flagged.
pub fn track_wizard_enemy_damage(
    query: Query<&Team, Added<SpellDamaged>>,
    mut kill_stats: ResMut<super::super::resources::KillStats>,
) {
    for team in &query {
        if *team == Team::Attackers || *team == Team::Undead {
            kill_stats.wizard_damaged_enemies = true;
            return;
        }
    }
}

/// Run condition: returns true when no wizard spell has damaged enemies yet this battle.
pub fn wizard_has_not_damaged_enemies(kill_stats: Res<super::super::resources::KillStats>) -> bool {
    !kill_stats.wizard_damaged_enemies
}

#[cfg(test)]
mod proximity_tests {
    use super::*;

    /// Reference O(n²) implementation — the grid version must match it exactly.
    fn count_brute_force(units: &[ProxUnit]) -> Vec<(u32, u32)> {
        let mut out = vec![(0u32, 0u32); units.len()];
        for (i, u) in units.iter().enumerate() {
            for (j, o) in units.iter().enumerate() {
                if i == j {
                    continue;
                }
                let dx = u.pos.x - o.pos.x;
                let dz = u.pos.z - o.pos.z;
                let distance = (dx * dx + dz * dz).sqrt();
                let melee_range = (u.radius + o.radius) * ATTACK_RANGE_MULTIPLIER;
                if distance <= melee_range {
                    if u.team.is_enemy(&o.team) {
                        out[i].1 += 1;
                    } else {
                        out[i].0 += 1;
                    }
                }
            }
        }
        out
    }

    // A deterministic LCG so the scenario is fixed (no rand dependency, no flakiness).
    fn scenario(n: usize, spread: f32, radius_choices: &[f32]) -> Vec<ProxUnit> {
        let mut state: u64 = 0x9E3779B97F4A7C15;
        let mut next = || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((state >> 33) as f32) / (u32::MAX as f32)
        };
        let teams = [Team::Defenders, Team::Attackers, Team::Undead];
        (0..n)
            .map(|_| ProxUnit {
                pos: Vec3::new((next() - 0.5) * spread, 0.0, (next() - 0.5) * spread),
                radius: radius_choices
                    [(next() * radius_choices.len() as f32) as usize % radius_choices.len()],
                team: teams[(next() * 3.0) as usize % 3],
            })
            .collect()
    }

    #[test]
    fn grid_matches_brute_force_dense() {
        // Dense cluster: many units within range of each other.
        let units = scenario(300, 200.0, &[10.0, 15.0, 25.0]);
        assert_eq!(count_melee_proximity(&units), count_brute_force(&units));
    }

    #[test]
    fn grid_matches_brute_force_sparse() {
        // Sparse field with mixed radii (varying interaction distances).
        let units = scenario(250, 4000.0, &[5.0, 20.0, 40.0]);
        assert_eq!(count_melee_proximity(&units), count_brute_force(&units));
    }

    #[test]
    fn grid_matches_brute_force_edges() {
        // Empty, single, and uniform-radius cases.
        assert_eq!(count_melee_proximity(&[]), Vec::<(u32, u32)>::new());
        let one = scenario(1, 100.0, &[12.0]);
        assert_eq!(count_melee_proximity(&one), count_brute_force(&one));
        let uniform = scenario(120, 600.0, &[18.0]);
        assert_eq!(count_melee_proximity(&uniform), count_brute_force(&uniform));
    }
}
