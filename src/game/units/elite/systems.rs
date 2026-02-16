use bevy::prelude::*;

use super::components::*;
use super::constants::ELITE_HEALTH_BONUS;
use crate::game::units::components::{Corpse, Health};

/// Applies health bonus to units with EliteHealthBonus component.
///
/// This system runs when a unit gains the EliteHealthBonus component
/// and increases both max HP and current HP by the bonus amount.
///
/// The system is idempotent - it only applies the bonus once when
/// the component is first added (via Added<EliteHealthBonus> filter).
pub fn apply_elite_health_bonus(
    mut query: Query<(&mut Health, &EliteHealthBonus), (Without<Corpse>, Added<EliteHealthBonus>)>,
) {
    for (mut health, bonus) in query.iter_mut() {
        // Increase both max and current HP proportionally
        health.max += bonus.0;
        health.current += bonus.0;
    }
}

/// Removes health bonus when EliteHealthBonus component is removed.
///
/// This system handles cleanup when a unit loses elite status.
/// It reduces max HP by the standard elite bonus amount and clamps
/// current HP to the new maximum (preventing HP from exceeding max).
///
/// Note: This assumes the standard ELITE_HEALTH_BONUS value.
/// If units have custom bonus amounts, they will still be reduced
/// by the standard amount.
pub fn remove_elite_health_bonus(
    mut removed: RemovedComponents<EliteHealthBonus>,
    mut health_query: Query<&mut Health>,
) {
    for entity in removed.read() {
        if let Ok(mut health) = health_query.get_mut(entity) {
            // Reduce max HP by standard elite bonus
            health.max -= ELITE_HEALTH_BONUS;
            // Clamp current HP to new maximum
            health.current = health.current.min(health.max);
        }
    }
}
