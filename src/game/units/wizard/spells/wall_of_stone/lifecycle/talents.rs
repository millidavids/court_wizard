use super::super::components::{
    CollapseExploded, LivingStoneTracker, PermafrostAuraTimer, WallHealth, WallOfStone, WallTalents,
};
use super::super::constants::*;
use crate::game::units::components::{Corpse, SlowMovementModifier, Team, TemporaryHitPoints};
use bevy::prelude::*;

/// Permafrost Aura: slows enemies within range of any wall that has the talent.
pub fn apply_permafrost_aura(
    time: Res<Time>,
    mut timer: ResMut<PermafrostAuraTimer>,
    walls: Query<(&WallOfStone, &WallTalents), Without<Corpse>>,
    mut enemies: Query<
        (&Transform, &Team, Option<&mut SlowMovementModifier>, Entity),
        (
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut commands: Commands,
) {
    timer.0 += time.delta_secs();
    if timer.0 < PERMAFROST_AURA_TICK_INTERVAL {
        return;
    }
    timer.0 = 0.0;

    // Collect wall positions that have the permafrost aura talent
    let frost_walls: Vec<_> = walls
        .iter()
        .filter(|(_, talents)| talents.0.permafrost_aura)
        .map(|(wall, _)| wall.center)
        .collect();

    if frost_walls.is_empty() {
        return;
    }

    let radius_sq = PERMAFROST_AURA_RADIUS * PERMAFROST_AURA_RADIUS;

    for (transform, team, slow_mod, entity) in &mut enemies {
        // Only slow attackers and undead
        if *team == Team::Defenders {
            continue;
        }

        let pos = transform.translation;
        let in_range = frost_walls.iter().any(|center| {
            let dx = pos.x - center.x;
            let dz = pos.z - center.z;
            dx * dx + dz * dz <= radius_sq
        });

        if in_range {
            if let Some(mut existing) = slow_mod {
                existing.apply(PERMAFROST_AURA_SLOW, PERMAFROST_AURA_SLOW_DURATION);
            } else {
                commands.entity(entity).insert(SlowMovementModifier::new(
                    PERMAFROST_AURA_SLOW,
                    PERMAFROST_AURA_SLOW_DURATION,
                ));
            }
        }
    }
}

/// Living Stone: regenerates wall HP when not being attacked recently.
pub fn regenerate_living_stone(
    time: Res<Time>,
    mut walls: Query<(&mut WallHealth, &mut LivingStoneTracker), With<WallOfStone>>,
) {
    let delta = time.delta_secs();
    for (mut health, mut tracker) in &mut walls {
        tracker.time_since_last_damage += delta;

        if tracker.time_since_last_damage >= LIVING_STONE_REGEN_DELAY && health.current < health.max
        {
            let regen = health.max * LIVING_STONE_REGEN_FRACTION * delta;
            health.current = (health.current + regen).min(health.max);
        }
    }
}

/// Collapsing Wall: deals AoE damage when a wall is destroyed.
/// Uses `CollapseExploded` marker to ensure each wall only explodes once.
pub fn collapsing_wall_explosion(
    mut commands: Commands,
    walls: Query<(Entity, &WallOfStone, &WallHealth, &WallTalents), Without<CollapseExploded>>,
    mut enemies: Query<
        (
            &Transform,
            &Team,
            &mut crate::game::units::components::Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) {
    let radius_sq = COLLAPSING_WALL_RADIUS * COLLAPSING_WALL_RADIUS;

    for (entity, wall, health, talents) in &walls {
        if !talents.0.collapsing_wall || !health.is_dead() || !wall.sinking {
            continue;
        }

        // Mark as exploded so we don't fire again
        commands.entity(entity).insert(CollapseExploded);

        let center = wall.center;

        for (transform, team, mut unit_health, temp_hp) in &mut enemies {
            if *team == Team::Defenders {
                continue;
            }
            let dx = transform.translation.x - center.x;
            let dz = transform.translation.z - center.z;
            if dx * dx + dz * dz <= radius_sq {
                crate::game::units::components::apply_damage_to_unit(
                    &mut unit_health,
                    temp_hp.map(|t| t.into_inner()),
                    COLLAPSING_WALL_DAMAGE,
                );
            }
        }
    }
}

/// Maze Architect: when 3+ walls exist, boost all wall max HP.
/// Runs every frame to adjust wall health as walls are placed or destroyed.
pub fn maze_architect_bonus(mut walls: Query<(&WallTalents, &mut WallHealth), With<WallOfStone>>) {
    // Single pass: count walls and check for maze talent simultaneously
    let mut wall_count = 0usize;
    let mut has_maze = false;
    for (talents, _) in walls.iter() {
        wall_count += 1;
        if talents.0.maze_architect {
            has_maze = true;
        }
    }

    if !has_maze {
        return;
    }

    let bonus_active = wall_count >= MAZE_ARCHITECT_WALL_THRESHOLD;

    for (talents, mut health) in &mut walls {
        let base = WALL_HEALTH * talents.0.health_mult;
        let expected_max = if bonus_active {
            base * MAZE_ARCHITECT_HEALTH_MULT
        } else {
            base
        };

        // Only adjust if the max HP doesn't match expectation
        if (health.max - expected_max).abs() > 0.1 {
            let hp_fraction = health.fraction();
            health.max = expected_max;
            health.current = expected_max * hp_fraction;
        }
    }
}
