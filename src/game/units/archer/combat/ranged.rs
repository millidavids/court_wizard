use std::cmp::Ordering;

use super::super::arrows::spawn_arrow;
use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::ArcherAssets;
use crate::game::components::Velocity;
use crate::game::constants::*;
use crate::game::pathfinding::{StagingAttacker, WaveGroup};
use crate::game::units::components::{BanishedModifier, Corpse, Hitbox, SleepModifier, Team};
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;

/// Updates archer movement timers to track time since stopped moving.
pub fn update_archer_movement_timers(
    time: Res<Time>,
    mut archers: Query<(&Velocity, &mut ArcherMovementTimer), With<Archer>>,
) {
    let delta = time.delta_secs();
    for (velocity, mut timer) in &mut archers {
        // Check if archer is moving (velocity threshold - very low to catch nearly stationary archers)
        let is_moving = velocity.x.abs() > 0.1 || velocity.z.abs() > 0.1;

        if is_moving {
            // Archer is moving
            timer.time_since_stopped = 0.0;
            timer.was_moving = true;
        } else if timer.was_moving {
            // Just stopped moving
            timer.time_since_stopped = 0.0;
            timer.was_moving = false;
        } else {
            // Stationary - accumulate time
            timer.time_since_stopped += delta;
        }

        // Always tick attack cooldown
        timer.time_since_last_attack += delta;
    }
}

/// Checks if a target is valid for the given team (same logic as combat system).
pub(crate) fn is_valid_target(source_team: &Team, target_team: &Team) -> bool {
    source_team.is_enemy(target_team)
}

/// Returns true if any wall is near the straight-line path between two points.
///
/// Unlike `line_segment_intersects` (which checks exact LOS), this uses a broader
/// check: if a wall's center projects onto the line segment and is within
/// (wall_extent + buffer) of the line, the wall is considered an obstruction.
/// This catches walls that force the flow field to detour even if they don't
/// block the geometric line-of-sight.
pub(crate) fn wall_near_approach_path(walls: &[&WallOfStone], from: Vec3, to: Vec3) -> bool {
    let line_dir = Vec3::new(to.x - from.x, 0.0, to.z - from.z);
    let line_len = line_dir.length();
    if line_len < 1.0 {
        return false;
    }
    let line_normalized = line_dir / line_len;

    for wall in walls {
        let to_wall = Vec3::new(wall.center.x - from.x, 0.0, wall.center.z - from.z);
        let projection = to_wall.dot(line_normalized);

        // Wall center must project between the two endpoints (with margin)
        if projection < 0.0 || projection > line_len {
            continue;
        }

        // Perpendicular distance from wall center to the line
        let closest_on_line = Vec3::new(
            from.x + line_normalized.x * projection,
            0.0,
            from.z + line_normalized.z * projection,
        );
        let perp_dist = Vec3::new(
            wall.center.x - closest_on_line.x,
            0.0,
            wall.center.z - closest_on_line.z,
        )
        .length();

        // Use the wall's largest extent plus a generous buffer so that walls
        // near (but not directly on) the LOS line still count as obstructions.
        let wall_extent = wall.half_length.max(wall.half_width);
        if perp_dist < wall_extent + WALL_APPROACH_PATH_BUFFER {
            return true;
        }
    }
    false
}

/// Archer ranged combat system that spawns arrows instead of dealing direct damage.
/// Only fires if no melee targets are available.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn archer_ranged_combat(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    archer_assets: Res<ArcherAssets>,
    mut archers: Query<
        (
            Entity,
            &Transform,
            &Team,
            &AttackRange,
            &mut ArcherMovementTimer,
            Option<&SleepModifier>,
            Option<&BanishedModifier>,
            Has<crate::game::units::infantry::components::Retreating>,
            Option<&crate::game::units::wizard::spells::fog_cloud::components::BlindingMistDebuff>,
            Has<StagingAttacker>,
            Has<WaveGroup>,
        ),
        (With<Archer>, Without<Corpse>),
    >,
    targets: Query<
        (
            Entity,
            &Transform,
            &Team,
            &Hitbox,
            Option<&crate::game::units::components::InMelee>,
            Has<crate::game::units::boss::components::Boss>,
        ),
        (Without<Corpse>, Without<BanishedModifier>),
    >,
    walls: Query<&WallOfStone>,
    rocks_query: Query<&crate::game::terrain::boulder::components::Boulder>,
    trees_query: Query<&crate::game::terrain::tree::components::Tree>,
    concealing_veil_zones: Query<
        &crate::game::units::wizard::spells::fog_cloud::components::FogCloudZone,
        With<crate::game::units::wizard::spells::fog_cloud::components::ConcealingVeilZone>,
    >,
) {
    let wall_snapshot: Vec<_> = walls.iter().collect();
    let rock_snapshot: Vec<_> = rocks_query.iter().filter(|r| !r.sinking).collect();
    let tree_snapshot: Vec<_> = trees_query.iter().collect();

    // Collect concealing veil zone snapshots for ranged targeting checks
    let concealing_veil_snapshot: Vec<(Vec3, f32)> = concealing_veil_zones
        .iter()
        .map(|z| (z.origin, z.radius))
        .collect();

    for (
        archer_entity,
        archer_transform,
        archer_team,
        attack_range,
        mut movement_timer,
        sleeping,
        banished,
        is_retreating,
        blinding_mist,
        has_staging,
        has_wave_group,
    ) in archers.iter_mut()
    {
        // Skip staging attackers (includes 1-frame delay before WaveGroup is added)
        if crate::game::units::systems::is_staging_attacker(
            archer_team,
            has_staging,
            has_wave_group,
        ) {
            continue;
        }

        // Skip attack if sleeping, banished, or retreating
        if sleeping.is_some() || banished.is_some() || is_retreating {
            continue;
        }

        // Check if enough time has passed since stopping to attack
        if !movement_timer.can_attack(ARCHER_ATTACK_DELAY_AFTER_MOVEMENT) {
            continue;
        }

        // Check attack cooldown
        let attack_cooldown = ATTACK_CYCLE_DURATION * ARCHER_ATTACK_COOLDOWN_MULTIPLIER;
        if movement_timer.time_since_last_attack < attack_cooldown {
            continue;
        }

        // Blinding Mist: halve max attack range when debuffed
        let effective_max_range = if let Some(debuff) = blinding_mist {
            attack_range.max_range * debuff.range_mult
        } else {
            attack_range.max_range
        };

        // Concealing Veil: skip check if archer is also inside a veil zone
        // (units inside fog together can still target each other)
        let archer_is_in_veil = !concealing_veil_snapshot.is_empty()
            && crate::game::units::wizard::spells::fog_cloud::systems::is_in_fog_zone(
                archer_transform.translation,
                &concealing_veil_snapshot,
            );

        // Find nearest enemy within ranged attack max_range
        // Exclude targets in melee with someone on the archer's own team
        let nearest_enemy = targets
            .iter()
            .filter(|(entity, _, team, _, in_melee, is_boss)| {
                // Skip self
                if *entity == archer_entity {
                    return false;
                }
                // Must be a valid enemy
                if !is_valid_target(archer_team, team) {
                    return false;
                }
                // Skip if target is in melee with archer's own team
                // (but always allow targeting the boss even when in melee)
                if !is_boss
                    && let Some(in_melee_component) = in_melee
                    && in_melee_component.0 == *archer_team
                {
                    return false;
                }
                true
            })
            .filter(|(_, transform, _, _, _, _)| {
                let distance = archer_transform.translation.distance(transform.translation);
                if distance > effective_max_range || distance < attack_range.min_range {
                    return false;
                }
                // Concealing Veil: units in fog can't be targeted by ranged attacks from outside
                if !archer_is_in_veil
                    && !concealing_veil_snapshot.is_empty()
                    && crate::game::units::wizard::spells::fog_cloud::systems::is_in_fog_zone(
                        transform.translation,
                        &concealing_veil_snapshot,
                    )
                {
                    return false;
                }
                // Skip targets blocked by walls, rocks, or trees
                !WallOfStone::any_blocks_los(
                    &wall_snapshot,
                    archer_transform.translation,
                    transform.translation,
                ) && !crate::game::terrain::boulder::components::Boulder::any_blocks_los(
                    &rock_snapshot,
                    archer_transform.translation,
                    transform.translation,
                ) && !crate::game::terrain::tree::components::Tree::any_blocks_los(
                    &tree_snapshot,
                    archer_transform.translation,
                    transform.translation,
                )
            })
            .min_by(|a, b| {
                let dist_a = archer_transform.translation.distance(a.1.translation);
                let dist_b = archer_transform.translation.distance(b.1.translation);
                dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal)
            });

        if let Some((_, target_transform, _, _, _, _)) = nearest_enemy {
            // Spawn arrow projectile directly above the archer
            spawn_arrow(
                &mut game_rng.0,
                &mut commands,
                &archer_assets,
                archer_transform.translation + Vec3::Y * 10.0,
                target_transform.translation,
                *archer_team,
            );
            // Reset attack cooldown
            movement_timer.time_since_last_attack = 0.0;

            // Trigger shooting animation
            commands.entity(archer_entity).insert(
                crate::game::units::components::CombatAnimation::new_shooting(
                    archer_assets.shooting_texture.clone(),
                    archer_assets.sprite_texture.clone(),
                ),
            );
        }
    }
}
