//! Upgrade selection logic for elite and commander units.
//!
//! This module handles randomly selecting which attacker units to upgrade
//! to elites, commanders, dispellers, healers, or shielders based on
//! level-scaled probabilities. Used by both the loading spawn queue and
//! the wave upgrade system.

use std::collections::HashSet;

use bevy::prelude::*;
use rand::prelude::*;

use super::constants::*;
use crate::game::units::dispeller::constants::calculate_attacker_dispellers;
use crate::game::units::healer::constants::calculate_attacker_healers;
use crate::game::units::shielder::constants::calculate_attacker_shielders;

/// Core selection logic: filters out excluded entities, shuffles with seeded RNG,
/// and returns up to `count` selected entities.
fn select_from_pool(
    candidates: &[Entity],
    count: usize,
    excluded: &HashSet<Entity>,
    seed_base: u64,
    seed_multiplier: u64,
) -> Vec<Entity> {
    if count == 0 {
        return Vec::new();
    }

    let mut available: Vec<Entity> = candidates
        .iter()
        .filter(|entity| !excluded.contains(entity))
        .copied()
        .collect();

    if available.is_empty() {
        return Vec::new();
    }

    let count = count.min(available.len());

    let mut rng = StdRng::seed_from_u64(seed_base.wrapping_mul(seed_multiplier));
    available.shuffle(&mut rng);

    available.into_iter().take(count).collect()
}

/// Selects attacker units to upgrade to commanders.
///
/// Takes a list of attacker entities of a single type, calculates commander count
/// based on level and probability, then randomly selects entities using a seeded RNG.
///
/// `seed_base` is the base value for RNG seeding (typically level, or level + wave offset).
/// `seed_multiplier` ensures different unit types get different RNG sequences.
pub(in crate::game) fn select_commander_entities(
    attackers: &[Entity],
    level: u32,
    max_commanders: u32,
    seed_base: u64,
    seed_multiplier: u64,
    type_name: &str,
) -> Vec<Entity> {
    let total_count = attackers.len();
    if total_count == 0 {
        return Vec::new();
    }

    let commander_count = calculate_commander_count(total_count, level, max_commanders);

    // Commanders select from all candidates (no exclusion set)
    let mut rng = StdRng::seed_from_u64(seed_base.wrapping_mul(seed_multiplier));
    let mut shuffled = attackers.to_vec();
    shuffled.shuffle(&mut rng);

    let selected: Vec<Entity> = shuffled.into_iter().take(commander_count).collect();

    info!(
        "{} upgrades selected: {} commanders (from {} total)",
        type_name,
        selected.len(),
        total_count
    );

    selected
}

/// Selects archer entities to upgrade to dispellers.
pub(in crate::game) fn select_dispeller_entities(
    archers: &[Entity],
    level: u32,
    excluded: &HashSet<Entity>,
    seed_base: u64,
) -> Vec<Entity> {
    let count = calculate_attacker_dispellers(level) as usize;
    let selected = select_from_pool(archers, count, excluded, seed_base, 1009);
    if !selected.is_empty() {
        info!("Dispeller upgrades selected: {}", selected.len());
    }
    selected
}

/// Selects archer entities to upgrade to healers.
pub(in crate::game) fn select_healer_entities(
    archers: &[Entity],
    level: u32,
    excluded: &HashSet<Entity>,
    seed_base: u64,
) -> Vec<Entity> {
    let count = calculate_attacker_healers(level) as usize;
    let selected = select_from_pool(archers, count, excluded, seed_base, 1013);
    if !selected.is_empty() {
        info!("Healer upgrades selected: {}", selected.len());
    }
    selected
}

/// Selects infantry entities to upgrade to shielders.
pub(in crate::game) fn select_shielder_entities(
    infantry: &[Entity],
    level: u32,
    excluded: &HashSet<Entity>,
    seed_base: u64,
) -> Vec<Entity> {
    let count = calculate_attacker_shielders(level) as usize;
    let selected = select_from_pool(infantry, count, excluded, seed_base, 1019);
    if !selected.is_empty() {
        info!("Shielder upgrades selected: {}", selected.len());
    }
    selected
}

/// Selects attacker entities of any type to upgrade to elites.
///
/// This should run AFTER all other upgrade passes (commander, dispeller,
/// healer, shielder) so that any remaining unit type can become elite.
pub(in crate::game) fn select_elite_entities(
    all_attackers: &[Entity],
    level: u32,
    excluded: &HashSet<Entity>,
    seed_base: u64,
) -> Vec<Entity> {
    let elite_chance = calculate_elite_chance(level);
    if elite_chance <= 0.0 {
        return Vec::new();
    }

    let available_count = all_attackers
        .iter()
        .filter(|entity| !excluded.contains(entity))
        .count();

    let raw_elite_count = (available_count as f32 * elite_chance).round() as usize;
    let elite_count = raw_elite_count.min(MAX_ELITES as usize);

    let selected = select_from_pool(all_attackers, elite_count, excluded, seed_base, 1031);
    if !selected.is_empty() {
        info!(
            "Elite upgrades selected: {} (from {} available)",
            selected.len(),
            available_count
        );
    }
    selected
}

/// Calculates how many units to upgrade to commanders.
fn calculate_commander_count(total_units: usize, level: u32, max_commanders: u32) -> usize {
    let commander_chance = calculate_commander_chance(level);
    let raw_commander_count = (total_units as f32 * commander_chance).round() as usize;
    raw_commander_count.min(max_commanders as usize)
}
