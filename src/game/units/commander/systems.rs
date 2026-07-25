use bevy::prelude::*;
use std::collections::HashMap;

use super::components::*;
use crate::game::units::components::{CommanderAuraSpeedModifier, Corpse, DamageMultiplier, Team};
use crate::game::units::wizard::components::Wizard;

/// Generic commander aura system.
///
/// Applies buffs to units within commander's aura radius based on team filter.
/// Commanders always self-buff (they benefit from their own aura).
/// Buffs are proximity-based (no timers) - removed when leaving radius.
///
/// When multiple auras overlap, the maximum buff value is applied.
pub fn apply_commander_auras(
    mut commands: Commands,
    // Staging commanders project no aura (and don't self-buff): staging is a
    // no-advantage phase, and a speed aura would even skew march arrival times.
    commanders: Query<
        (
            Entity,
            &Transform,
            &Commander,
            Option<&AuraDamageBuff>,
            Option<&AuraSpeedBuff>,
        ),
        (
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    affected_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<Wizard>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    // Reused across frames so we don't heap-allocate a fresh map every tick.
    mut units_in_aura: Local<HashMap<Entity, (Option<f32>, Option<f32>)>>,
) {
    // Collect all commander data upfront to avoid borrow conflicts
    let commander_data: Vec<_> = commanders
        .iter()
        .map(|(entity, transform, commander, damage_buff, speed_buff)| {
            (
                entity,
                transform.translation,
                commander.aura_radius,
                commander.team_filter,
                damage_buff.map(|b| b.0),
                speed_buff.map(|b| b.0),
            )
        })
        .collect();

    // Track which units are in ANY commander's aura
    // HashMap<Entity, (max_damage_buff, max_speed_buff)>
    units_in_aura.clear();

    // For each commander, find units in range and track max buffs
    for (commander_entity, commander_pos, radius, team_filter, damage_buff, speed_buff) in
        &commander_data
    {
        // Self-buff the commander (speed only - damage handled separately)
        if let Some(speed_percentage) = speed_buff {
            commands
                .entity(*commander_entity)
                .insert(CommanderAuraSpeedModifier(*speed_percentage));
        }

        // Check each unit for proximity
        for (unit_entity, unit_transform, unit_team) in affected_units.iter() {
            // Skip if team filter doesn't allow this unit
            if !team_filter.allows(*unit_team) {
                continue;
            }

            // Distance check (3D spherical distance)
            let distance = unit_transform.translation.distance(*commander_pos);
            if distance < *radius && distance > 0.1 {
                // Unit is in aura range
                let entry = units_in_aura.entry(unit_entity).or_insert((None, None));

                // Track maximum damage buff
                if let Some(dmg) = damage_buff {
                    entry.0 = Some(entry.0.map_or(*dmg, |current| current.max(*dmg)));
                }

                // Track maximum speed buff
                if let Some(spd) = speed_buff {
                    entry.1 = Some(entry.1.map_or(*spd, |current| current.max(*spd)));
                }
            }
        }
    }

    // Apply buffs to units in aura range
    for (unit_entity, (max_damage, max_speed)) in &*units_in_aura {
        if let Some(damage) = max_damage {
            commands
                .entity(*unit_entity)
                .insert(DamageMultiplier(*damage));
        }

        if let Some(speed) = max_speed {
            commands
                .entity(*unit_entity)
                .insert(CommanderAuraSpeedModifier(*speed));
        }
    }

    // Remove buffs from units NOT in any aura
    for (unit_entity, _, _) in affected_units.iter() {
        let in_aura = units_in_aura.get(&unit_entity);

        // Remove damage buff if not in any damage aura
        if !matches!(in_aura, Some((Some(_), _))) {
            commands.entity(unit_entity).remove::<DamageMultiplier>();
        }

        // Remove speed buff if not in any speed aura
        if !matches!(in_aura, Some((_, Some(_)))) {
            commands
                .entity(unit_entity)
                .remove::<CommanderAuraSpeedModifier>();
        }
    }
}
