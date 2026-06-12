use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::{
    BanishedModifier, Corpse, MindControlled, TargetingVelocity, Team,
};
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::units::wizard::spells::dispel::systems::{is_dispellable, spell_edge_distance};
use crate::game::units::wizard::spells::grease::components::{GreaseIgnited, GreaseZone};
use crate::game::units::wizard::spells::meteor_fall::components::MeteorGroundFire;
use crate::game::units::wizard::spells::spike_growth::components::SpikeGrowthZone;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;

/// Updates dispeller targeting — seeks nearest dispellable spell effect, or falls back to enemy targeting.
#[allow(clippy::too_many_arguments)]
pub fn update_dispeller_targeting(
    defenders_activated: Res<DefendersActivated>,
    mut commands: Commands,
    mut dispellers: Query<
        (Entity, &Transform, &Team, &mut TargetingVelocity),
        (With<Dispeller>, Without<Corpse>, Without<MindControlled>),
    >,
    spell_effects: Query<(Entity, &Transform, &NetworkedSpellEffect)>,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
    // Spell-specific queries for volume-aware distance
    wall_of_fire_query: Query<&WallOfFireEffect>,
    wall_of_stone_query: Query<&WallOfStone>,
    spike_growth_query: Query<&SpikeGrowthZone>,
    grease_query: Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: Query<&MeteorGroundFire>,
) {
    // Collect dispellable spell effects (entity + center position)
    let spell_targets: Vec<(Entity, Vec3)> = spell_effects
        .iter()
        .filter(|(_, _, nse)| is_dispellable(nse.kind))
        .map(|(entity, transform, _)| (entity, transform.translation))
        .collect();

    // Collect unit snapshot for enemy targeting fallback
    let unit_snapshot: Vec<(Entity, Vec3, Team)> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, transform, team, mut targeting_velocity) in &mut dispellers {
        // Skip inactive defender dispellers
        if *team == Team::Defenders && !defenders_activated.active {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
            continue;
        }

        // Priority 1: Nearest dispellable spell effect (using edge distance)
        if !spell_targets.is_empty() {
            // Compute each edge distance once, then pick the minimum by the cached
            // value (the comparator previously recomputed it twice per pair).
            let nearest_spell = spell_targets
                .iter()
                .map(|&(spell_entity, target_pos)| {
                    let dist = spell_edge_distance(
                        transform.translation,
                        spell_entity,
                        target_pos,
                        &wall_of_fire_query,
                        &wall_of_stone_query,
                        &spike_growth_query,
                        &grease_query,
                        &meteor_fire_query,
                    );
                    (spell_entity, target_pos, dist)
                })
                .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

            if let Some((_spell_entity, target_pos, distance)) = nearest_spell {
                targeting_velocity.distance_to_target = distance;

                if distance <= DISPEL_RANGE {
                    // In dispel range — stop moving
                    targeting_velocity.velocity = Vec3::ZERO;
                } else {
                    // Move toward spell effect center
                    let diff = target_pos - transform.translation;
                    let direction = diff.normalize_or_zero();
                    targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
                }

                // Dispellers don't engage in melee — remove InMelee
                commands
                    .entity(entity)
                    .remove::<crate::game::units::components::InMelee>();
                continue;
            }
        }

        // Priority 2: Fall back to ranged enemy targeting (like archers)
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

        if let Some(&(_, target_pos, _)) = nearest_enemy {
            let diff = target_pos - transform.translation;
            let distance = (diff.x.powi(2) + diff.z.powi(2)).sqrt();
            targeting_velocity.distance_to_target = distance;

            if distance <= ATTACK_RANGE {
                // In attack range — stop and shoot
                targeting_velocity.velocity = Vec3::ZERO;
            } else {
                // Move toward enemy
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            }
        } else {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
        }

        commands
            .entity(entity)
            .remove::<crate::game::units::components::InMelee>();
    }
}
