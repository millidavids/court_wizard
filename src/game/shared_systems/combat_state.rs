use bevy::prelude::*;

use super::super::constants::*;
use super::super::units::components::{Corpse, Effectiveness, Hitbox, SpellDamaged, Team};

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
) {
    // Collect snapshot for symmetric calculations
    let unit_data: Vec<_> = units
        .iter()
        .map(|(entity, transform, hitbox, team, _)| (entity, transform.translation, *hitbox, *team))
        .collect();

    // Calculate effectiveness for each unit
    for (entity, transform, hitbox, team, mut effectiveness) in units.iter_mut() {
        let mut ally_count = 0;
        let mut enemy_count = 0;

        for (other_entity, other_pos, other_hitbox, other_team) in &unit_data {
            if *other_entity == entity {
                continue;
            }

            // Calculate XZ plane distance
            let dx = transform.translation.x - other_pos.x;
            let dz = transform.translation.z - other_pos.z;
            let distance = (dx * dx + dz * dz).sqrt();

            // Use same melee range formula as combat
            let melee_range = (hitbox.radius + other_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;

            if distance <= melee_range {
                let is_enemy = team.is_enemy(other_team);

                if is_enemy {
                    enemy_count += 1;
                } else {
                    ally_count += 1;
                }
            }
        }

        effectiveness.recalculate(ally_count, enemy_count);
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
