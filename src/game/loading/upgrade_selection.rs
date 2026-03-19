//! Upgrade selection logic for elite and commander units.
//!
//! This module handles querying spawned attacker units and randomly selecting
//! which ones to upgrade to elites or commanders based on level-scaled probabilities.

use std::collections::HashSet;

use bevy::prelude::*;
use rand::prelude::*;

use super::constants::*;
use super::spawn_queue::SpawnTask;
use crate::game::units::archer::Archer;
use crate::game::units::components::Team;
use crate::game::units::dispeller::constants::calculate_attacker_dispellers;
use crate::game::units::healer::constants::calculate_attacker_healers;
use crate::game::units::infantry::Infantry;
use crate::game::units::shielder::constants::calculate_attacker_shielders;

/// Collects entities already targeted for upgrades in existing spawn tasks.
/// Used by upgrade selection functions to avoid double-selecting units.
fn collect_excluded_upgrade_entities(existing_tasks: &[SpawnTask]) -> HashSet<Entity> {
    existing_tasks
        .iter()
        .filter_map(|task| match task {
            SpawnTask::UpgradeToElite { entity, .. }
            | SpawnTask::UpgradeToCommander { entity, .. }
            | SpawnTask::UpgradeToDispeller { entity }
            | SpawnTask::UpgradeToHealer { entity }
            | SpawnTask::UpgradeToShielder { entity } => Some(*entity),
            _ => None,
        })
        .collect()
}

/// Selects attacker units of a given type to upgrade to commanders.
///
/// Queries all spawned attackers matching the component filter, calculates
/// commander count based on level and probability, then randomly selects
/// entities using a seeded RNG.
///
/// `seed_multiplier` ensures different unit types get different RNG sequences.
pub(super) fn select_commander_upgrades<T: Component>(
    query: &Query<(Entity, &Team), With<T>>,
    level: u32,
    max_commanders: u32,
    seed_multiplier: u64,
    type_name: &str,
) -> Vec<SpawnTask> {
    let attackers: Vec<Entity> = query
        .iter()
        .filter(|(_, team)| **team == Team::Attackers)
        .map(|(entity, _)| entity)
        .collect();

    let total_count = attackers.len();
    if total_count == 0 {
        return Vec::new();
    }

    let commander_count = calculate_commander_count(total_count, level, max_commanders);

    let mut rng = StdRng::seed_from_u64((level as u64).wrapping_mul(seed_multiplier));
    let mut shuffled = attackers;
    shuffled.shuffle(&mut rng);

    let tasks: Vec<SpawnTask> = shuffled
        .into_iter()
        .take(commander_count)
        .map(|entity| SpawnTask::UpgradeToCommander { entity })
        .collect();

    info!(
        "{} upgrades selected: {} commanders (from {} total)",
        type_name,
        tasks.len(),
        total_count
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

    let excluded = collect_excluded_upgrade_entities(existing_tasks);

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

/// Selects archer units to upgrade to healers.
///
/// Queries all spawned attacker archers, excludes any already selected for other
/// upgrades (elite/commander/dispeller/healer), then randomly selects entities to
/// become healers based on level scaling.
///
/// Returns a list of upgrade tasks to enqueue (spread across frames).
pub(super) fn select_healer_upgrades(
    archer_query: &Query<(Entity, &Team), With<Archer>>,
    level: u32,
    existing_tasks: &[SpawnTask],
) -> Vec<SpawnTask> {
    let healer_count = calculate_attacker_healers(level) as usize;
    if healer_count == 0 {
        return Vec::new();
    }

    let excluded = collect_excluded_upgrade_entities(existing_tasks);

    // Collect all attacker archer entities not already selected for other upgrades
    let available_archers: Vec<Entity> = archer_query
        .iter()
        .filter(|(entity, team)| **team == Team::Attackers && !excluded.contains(entity))
        .map(|(entity, _)| entity)
        .collect();

    if available_archers.is_empty() {
        return Vec::new();
    }

    let count = healer_count.min(available_archers.len());

    // Create seeded RNG with unique seed for healer selection
    let mut rng = StdRng::seed_from_u64((level as u64).wrapping_mul(1013));

    let mut shuffled = available_archers;
    shuffled.shuffle(&mut rng);

    let tasks: Vec<SpawnTask> = shuffled
        .into_iter()
        .take(count)
        .map(|entity| SpawnTask::UpgradeToHealer { entity })
        .collect();

    info!(
        "Healer upgrades selected: {} (from attacker archers)",
        tasks.len()
    );

    tasks
}

/// Selects infantry units to upgrade to shielders.
///
/// Queries all spawned attacker infantry, excludes any already selected for other
/// upgrades (elite/commander), then randomly selects entities to become shielders
/// based on level scaling.
///
/// Returns a list of upgrade tasks to enqueue (spread across frames).
pub(super) fn select_shielder_upgrades(
    infantry_query: &Query<(Entity, &Team), With<Infantry>>,
    level: u32,
    existing_tasks: &[SpawnTask],
) -> Vec<SpawnTask> {
    let shielder_count = calculate_attacker_shielders(level) as usize;
    if shielder_count == 0 {
        return Vec::new();
    }

    // Collect entities already targeted for upgrades so we don't pick them
    let excluded = collect_excluded_upgrade_entities(existing_tasks);

    // Collect all attacker infantry entities not already selected for other upgrades
    let available_infantry: Vec<Entity> = infantry_query
        .iter()
        .filter(|(entity, team)| **team == Team::Attackers && !excluded.contains(entity))
        .map(|(entity, _)| entity)
        .collect();

    if available_infantry.is_empty() {
        return Vec::new();
    }

    let count = shielder_count.min(available_infantry.len());

    // Create seeded RNG with unique seed for shielder selection
    let mut rng = StdRng::seed_from_u64((level as u64).wrapping_mul(1019));

    let mut shuffled = available_infantry;
    shuffled.shuffle(&mut rng);

    let tasks: Vec<SpawnTask> = shuffled
        .into_iter()
        .take(count)
        .map(|entity| SpawnTask::UpgradeToShielder { entity })
        .collect();

    info!(
        "Shielder upgrades selected: {} (from attacker infantry)",
        tasks.len()
    );

    tasks
}

/// Selects attacker units of any type to upgrade to elites.
///
/// Takes pre-collected attacker entity lists from infantry and archer queries,
/// excludes those already targeted for other upgrades, then randomly selects
/// entities to become elites based on level scaling.
///
/// This runs AFTER all other upgrade passes (commander, dispeller, healer, shielder)
/// so that any unit type can become elite.
///
/// Returns a list of upgrade tasks to enqueue (spread across frames).
pub(super) fn select_elite_upgrades(
    attacker_entities: &[Entity],
    level: u32,
    existing_tasks: &[SpawnTask],
) -> Vec<SpawnTask> {
    let elite_chance = calculate_elite_chance(level);
    if elite_chance <= 0.0 {
        return Vec::new();
    }

    let excluded = collect_excluded_upgrade_entities(existing_tasks);

    // Filter out units already selected for other upgrades
    let mut available: Vec<Entity> = attacker_entities
        .iter()
        .filter(|entity| !excluded.contains(entity))
        .copied()
        .collect();

    if available.is_empty() {
        return Vec::new();
    }

    let total_count = available.len();
    let raw_elite_count = (total_count as f32 * elite_chance).round() as usize;
    let elite_count = raw_elite_count.min(MAX_ELITES as usize).min(total_count);

    if elite_count == 0 {
        return Vec::new();
    }

    // Create seeded RNG with unique seed for elite selection
    let mut rng = StdRng::seed_from_u64((level as u64).wrapping_mul(1031));
    available.shuffle(&mut rng);

    let tasks: Vec<SpawnTask> = available
        .into_iter()
        .take(elite_count)
        .map(|entity| SpawnTask::UpgradeToElite { entity })
        .collect();

    info!(
        "Elite upgrades selected: {} (from {} available attackers)",
        tasks.len(),
        total_count
    );

    tasks
}

/// Calculates how many units to upgrade to commanders.
///
/// Uses probability formulas from constants and applies caps.
///
/// Returns commander_count.
fn calculate_commander_count(
    total_units: usize,
    level: u32,
    max_commanders: u32,
) -> usize {
    let commander_chance = calculate_commander_chance(level);
    let raw_commander_count = (total_units as f32 * commander_chance).round() as usize;
    raw_commander_count.min(max_commanders as usize)
}
