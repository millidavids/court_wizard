//! Upgrade selection logic for elite and commander units.
//!
//! This module handles querying spawned attacker units and randomly selecting
//! which ones to upgrade to elites or commanders based on level-scaled probabilities.

use bevy::prelude::*;
use rand::prelude::*;

use super::constants::*;
use super::spawn_queue::{SpawnTask, UnitType};
use crate::game::units::archer::Archer;
use crate::game::units::components::Team;
use crate::game::units::dispeller::constants::calculate_attacker_dispellers;
use crate::game::units::infantry::Infantry;

/// Selects infantry units to upgrade to elites or commanders.
///
/// Queries all spawned attacker infantry, calculates upgrade counts based on
/// level and probability formulas, then randomly selects entities using seeded RNG.
///
/// Returns a list of upgrade tasks to enqueue (spread across frames).
pub(super) fn select_infantry_upgrades(
    infantry_query: &Query<(Entity, &Team), With<Infantry>>,
    level: u32,
) -> Vec<SpawnTask> {
    // Collect all attacker infantry entities
    let attacker_infantry: Vec<Entity> = infantry_query
        .iter()
        .filter(|(_, team)| **team == Team::Attackers)
        .map(|(entity, _)| entity)
        .collect();

    let total_count = attacker_infantry.len();
    if total_count == 0 {
        return Vec::new();
    }

    // Calculate upgrade counts
    let (commander_count, elite_count) = calculate_upgrade_counts(
        total_count,
        level,
        MAX_COMMANDER_INFANTRY,
        MAX_ELITE_INFANTRY,
    );

    // Create seeded RNG for deterministic upgrades per level
    let mut rng = StdRng::seed_from_u64(level as u64);

    // Shuffle all entities
    let mut shuffled = attacker_infantry.clone();
    shuffled.shuffle(&mut rng);

    // Select commanders first (they don't also get elite bonuses)
    let mut tasks = Vec::new();
    for entity in shuffled.iter().take(commander_count) {
        tasks.push(SpawnTask::UpgradeToCommander {
            entity: *entity,
            unit_type: UnitType::Infantry,
        });
    }

    // Select elites from remaining pool
    for entity in shuffled.iter().skip(commander_count).take(elite_count) {
        tasks.push(SpawnTask::UpgradeToElite {
            entity: *entity,
            unit_type: UnitType::Infantry,
        });
    }

    info!(
        "Infantry upgrades selected: {} commanders, {} elites (from {} total)",
        commander_count, elite_count, total_count
    );

    tasks
}

/// Selects archer units to upgrade to elites or commanders.
///
/// Queries all spawned attacker archers, calculates upgrade counts based on
/// level and probability formulas, then randomly selects entities using seeded RNG.
///
/// Returns a list of upgrade tasks to enqueue (spread across frames).
pub(super) fn select_archer_upgrades(
    archer_query: &Query<(Entity, &Team), With<Archer>>,
    level: u32,
) -> Vec<SpawnTask> {
    // Collect all attacker archer entities
    let attacker_archers: Vec<Entity> = archer_query
        .iter()
        .filter(|(_, team)| **team == Team::Attackers)
        .map(|(entity, _)| entity)
        .collect();

    let total_count = attacker_archers.len();
    if total_count == 0 {
        return Vec::new();
    }

    // Calculate upgrade counts
    let (commander_count, elite_count) =
        calculate_upgrade_counts(total_count, level, MAX_COMMANDER_ARCHERS, MAX_ELITE_ARCHERS);

    // Create seeded RNG for deterministic upgrades per level (offset seed to avoid same pattern as infantry)
    let mut rng = StdRng::seed_from_u64((level as u64).wrapping_mul(997)); // Prime multiplier for different sequence

    // Shuffle all entities
    let mut shuffled = attacker_archers.clone();
    shuffled.shuffle(&mut rng);

    // Select commanders first
    let mut tasks = Vec::new();
    for entity in shuffled.iter().take(commander_count) {
        tasks.push(SpawnTask::UpgradeToCommander {
            entity: *entity,
            unit_type: UnitType::Archer,
        });
    }

    // Select elites from remaining pool
    for entity in shuffled.iter().skip(commander_count).take(elite_count) {
        tasks.push(SpawnTask::UpgradeToElite {
            entity: *entity,
            unit_type: UnitType::Archer,
        });
    }

    info!(
        "Archer upgrades selected: {} commanders, {} elites (from {} total)",
        commander_count, elite_count, total_count
    );

    tasks
}

/// Selects archer units to upgrade to dispellers.
///
/// Queries all spawned attacker archers, excludes any already selected for elite/commander
/// upgrades, then randomly selects entities to become dispellers based on level scaling.
///
/// Returns a list of upgrade tasks to enqueue (spread across frames).
pub(super) fn select_dispeller_upgrades(
    archer_query: &Query<(Entity, &Team), With<Archer>>,
    level: u32,
    existing_tasks: &[SpawnTask],
) -> Vec<SpawnTask> {
    let dispeller_count = calculate_attacker_dispellers(level) as usize;
    if dispeller_count == 0 {
        return Vec::new();
    }

    // Collect entities already targeted for upgrades so we don't pick them
    let excluded: Vec<Entity> = existing_tasks
        .iter()
        .filter_map(|task| match task {
            SpawnTask::UpgradeToElite { entity, .. }
            | SpawnTask::UpgradeToCommander { entity, .. } => Some(*entity),
            _ => None,
        })
        .collect();

    // Collect all attacker archer entities not already selected for other upgrades
    let available_archers: Vec<Entity> = archer_query
        .iter()
        .filter(|(entity, team)| **team == Team::Attackers && !excluded.contains(entity))
        .map(|(entity, _)| entity)
        .collect();

    if available_archers.is_empty() {
        return Vec::new();
    }

    let count = dispeller_count.min(available_archers.len());

    // Create seeded RNG with unique seed for dispeller selection
    let mut rng = StdRng::seed_from_u64((level as u64).wrapping_mul(1009));

    let mut shuffled = available_archers;
    shuffled.shuffle(&mut rng);

    let tasks: Vec<SpawnTask> = shuffled
        .into_iter()
        .take(count)
        .map(|entity| SpawnTask::UpgradeToDispeller { entity })
        .collect();

    info!(
        "Dispeller upgrades selected: {} (from attacker archers)",
        tasks.len()
    );

    tasks
}

/// Calculates how many units to upgrade to commanders and elites.
///
/// Uses probability formulas from constants, applies caps, and ensures
/// commanders and elites don't overlap (commanders are selected first).
///
/// Returns (commander_count, elite_count).
fn calculate_upgrade_counts(
    total_units: usize,
    level: u32,
    max_commanders: u32,
    max_elites: u32,
) -> (usize, usize) {
    // Calculate raw counts based on probability
    let commander_chance = calculate_commander_chance(level);
    let elite_chance = calculate_elite_chance(level);

    let raw_commander_count = (total_units as f32 * commander_chance).round() as usize;
    let raw_elite_count = (total_units as f32 * elite_chance).round() as usize;

    // Apply caps
    let commander_count = raw_commander_count.min(max_commanders as usize);

    // Elites selected from remaining pool (after commanders)
    let remaining_for_elites = total_units.saturating_sub(commander_count);
    let elite_count = raw_elite_count
        .min(remaining_for_elites)
        .min(max_elites as usize);

    (commander_count, elite_count)
}
