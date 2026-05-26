//! Wall of stone lifecycle: cancel, tick, sink animation, cleanup, permanent.

use super::super::super::components::{CastingState, LocalWizard};
use super::components::{
    CollapseExploded, DispelledWall, LivingStoneTracker, PermafrostAuraTimer, WallHealth,
    WallOfStone, WallOfStoneCaster, WallRising, WallTalents,
};
use super::constants::*;
use super::wall_material::WallOfStoneMaterial;
use crate::config::save_data::SavedWall;
use crate::game::battlefield::trampling::constants::TRAMPLING_CELL_SIZE;
use crate::game::battlefield::trampling::resources::TramplingGrid;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{FlowFieldVelocity, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::plugin::GlobalAttackCycle;
use crate::game::units::components::{
    AttackTiming, Corpse, Hitbox, SlowMovementModifier, TargetingVelocity, Team,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
pub fn handle_wall_of_stone_cancel(
    mut mouse_right_pressed: MessageReader<crate::game::input::messages::MouseRightPressed>,
    mut commands: Commands,
    mut wizard_query: Query<&mut CastingState, With<LocalWizard>>,
    mut caster_query: Query<&mut WallOfStoneCaster, With<LocalWizard>>,
    mut mouse_state: ResMut<MouseButtonState>,
) {
    if mouse_right_pressed.read().next().is_none() {
        return;
    }

    let Ok(mut casting_state) = wizard_query.single_mut() else {
        return;
    };

    let Ok(mut caster) = caster_query.single_mut() else {
        return;
    };

    if let Some(preview_entity) = caster.preview_entity {
        commands.entity(preview_entity).try_despawn();
    }

    caster.anchor = None;
    caster.preview_entity = None;
    casting_state.cancel();
    mouse_state.left_consumed = true;
}

/// Advances wall lifetime and triggers sinking phase (skips permanent walls).
pub fn tick_wall_lifetime(time: Res<Time>, mut walls: Query<&mut WallOfStone>) {
    let delta = time.delta_secs();
    for mut wall in &mut walls {
        if wall.permanent {
            continue;
        }
        wall.time_alive += delta;
        if !wall.sinking && wall.time_alive >= wall.duration - WALL_SINK_DURATION {
            wall.sinking = true;
        }
    }
}

/// Animates walls sinking into the ground during their final seconds.
pub fn animate_sinking_walls(mut walls: Query<(&WallOfStone, &mut Transform)>) {
    for (wall, mut transform) in &mut walls {
        if wall.sinking {
            let sink_elapsed = wall.time_alive - (wall.duration - WALL_SINK_DURATION);
            let sink_progress = (sink_elapsed / WALL_SINK_DURATION).clamp(0.0, 1.0);
            let target_y = wall.height / 2.0 - wall.height * sink_progress;
            transform.translation.y = target_y;
        }
    }
}

/// Despawns walls that have exceeded their duration (skips permanent walls).
pub fn cleanup_expired_walls(
    mut commands: Commands,
    walls: Query<(Entity, &WallOfStone)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    mut connection: Option<ResMut<crate::networking::resources::NetworkConnection>>,
) {
    for (entity, wall) in &walls {
        if wall.permanent {
            continue;
        }
        if wall.time_alive >= wall.duration {
            commands.entity(entity).try_despawn();

            // Notify pathfinding system that the obstacle is removed
            let obs_bounds = wall.obstacle_bounds();
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::obb_from_center(
                    wall.center,
                    wall.forward,
                    wall.half_length,
                    wall.half_width,
                )),
                rebuild: false,
            });

            // Notify remote peer to update their pathfinding grid
            if let Some(ref mut conn) = connection {
                conn.outgoing_messages.push(
                    crate::networking::protocol::NetworkMessage::WallPlaced {
                        bounds: obs_bounds,
                        placed: false,
                    },
                );
            }
        }
    }
}

/// Spawns a permanent wall entity from saved wall data.
pub(crate) fn spawn_permanent_wall(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    saved: &SavedWall,
) {
    let forward = Vec3::new(saved.forward_x, 0.0, saved.forward_z);
    let right = Vec3::new(-forward.z, 0.0, forward.x);
    let center = Vec3::new(saved.center_x, 0.0, saved.center_z);
    let rotation = Quat::from_rotation_arc(Vec3::X, forward);

    commands.spawn((
        Mesh3d(assets.unit_cuboid.clone()),
        MeshMaterial3d(assets.wall_of_stone.clone()),
        Transform::from_xyz(center.x, saved.height / 2.0, center.z)
            .with_rotation(rotation)
            .with_scale(Vec3::new(
                saved.half_length * 2.0,
                saved.height,
                saved.half_width * 2.0,
            )),
        WallOfStone {
            center,
            half_length: saved.half_length,
            half_width: saved.half_width,
            forward,
            right,
            height: saved.height,
            time_alive: 0.0,
            duration: f32::MAX,
            sinking: false,
            empowerment: saved.empowerment,
            permanent: true,
        },
        WallHealth::new(WALL_HEALTH),
        NetworkedSpellEffect {
            kind: SpellEffectKind::WallOfStone,
        },
        OnGameplayScreen,
    ));
}

/// Registers pathfinding obstacles for all permanent walls after loading completes.
pub(crate) fn register_permanent_wall_obstacles(
    walls: Query<&WallOfStone>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for wall in &walls {
        if !wall.permanent {
            continue;
        }
        let obs_bounds = wall.obstacle_bounds();
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
            obstacle_type: ObstacleType::Blocked,
            shape: Some(ObstacleShape::obb_from_center(
                wall.center,
                wall.forward,
                wall.half_length,
                wall.half_width,
            )),
            rebuild: false,
        });
    }
}

/// Units with no valid path (pathfinding_distance == INFINITY) move toward the
/// king and attack any wall they end up pressed against. This prevents players
/// from exploiting wall placement to permanently trap units — blocked attackers
/// naturally converge on the walls surrounding the king rather than scattering
/// to the nearest wall on the map.
pub fn units_attack_blocking_walls(
    attack_cycle: Res<GlobalAttackCycle>,
    mut blocked_units: Query<
        (
            &Transform,
            &Hitbox,
            &FlowFieldVelocity,
            &mut TargetingVelocity,
            &mut AttackTiming,
            &mut crate::game::units::components::Health,
            Option<&mut crate::game::units::components::TemporaryHitPoints>,
        ),
        (Without<Corpse>, Without<WallOfStone>),
    >,
    king_query: Query<&Transform, With<crate::game::units::king::components::King>>,
    mut walls: Query<(
        Entity,
        &WallOfStone,
        &mut WallHealth,
        Option<&WallTalents>,
        Option<&mut LivingStoneTracker>,
    )>,
) {
    let current_time = attack_cycle.current_time;
    // `previous_time` is the cycle position before the most recent `tick()` —
    // see `GlobalAttackCycle` doc for why this is preferred over the
    // `current_time - APPROX_FRAME_TIME` approximation.
    let last_time = attack_cycle.previous_time;

    let king_pos = king_query.iter().next().map(|t| t.translation);

    for (transform, hitbox, flow_vel, mut targeting_vel, mut attack_timing, mut health, temp_hp) in
        &mut blocked_units
    {
        // Only target walls if this unit has no valid path
        if !flow_vel.pathfinding_distance.is_infinite() {
            continue;
        }

        let unit_pos = transform.translation;

        // Move toward the king — wall collision will stop the unit at the
        // blocking wall, causing units to pile up where they need to attack.
        if let Some(king) = king_pos {
            let diff = Vec3::new(king.x - unit_pos.x, 0.0, king.z - unit_pos.z);
            targeting_vel.velocity = diff.normalize_or_zero();
        }

        // Find nearest wall by distance to surface for melee damage
        let mut nearest_wall_entity = None;
        let mut nearest_distance = f32::MAX;

        for (entity, wall, _, _, _) in walls.iter() {
            let dist = wall.distance_to_surface(unit_pos);
            if dist < nearest_distance {
                nearest_distance = dist;
                nearest_wall_entity = Some(entity);
            }
        }

        // Deal damage if close enough to a wall
        let attack_range = hitbox.radius + WALL_ATTACK_RANGE;
        if let Some(wall_entity) = nearest_wall_entity
            && nearest_distance <= attack_range
            && attack_timing.can_attack(current_time, last_time)
            && let Ok((_, _, mut wall_health, wall_talents, living_stone_tracker)) =
                walls.get_mut(wall_entity)
        {
            wall_health.take_damage(WALL_DAMAGE_PER_HIT);
            attack_timing.record_attack(current_time);

            // Reset Living Stone regen timer on damage
            if let Some(mut tracker) = living_stone_tracker {
                tracker.time_since_last_damage = 0.0;
            }

            // Jagged Stone: reflect damage back to attacker
            if let Some(talents) = wall_talents
                && talents.0.jagged_stone
            {
                crate::game::units::components::apply_damage_to_unit(
                    &mut health,
                    temp_hp.map(|t| t.into_inner()),
                    JAGGED_STONE_REFLECT_DAMAGE,
                );
            }
        }
    }
}

/// Processes walls marked for dispel — starts the sinking animation.
pub fn handle_dispelled_walls(
    mut commands: Commands,
    mut walls: Query<(Entity, &mut WallOfStone, &DispelledWall)>,
) {
    for (entity, mut wall, dispelled) in &mut walls {
        if !wall.sinking {
            wall.sinking = true;
            wall.permanent = false;
            wall.duration = wall.time_alive + dispelled.sink_duration;
        }
        commands.entity(entity).remove::<DispelledWall>();
    }
}

/// Destroys walls that have lost all HP by triggering the existing sink + cleanup pipeline.
pub fn destroy_dead_walls(mut walls: Query<(&mut WallOfStone, &WallHealth)>) {
    for (mut wall, wall_health) in &mut walls {
        if wall_health.is_dead() && !wall.sinking {
            // Enter sinking phase — existing tick_wall_lifetime + cleanup_expired_walls
            // will handle the rest (obstacle removal, despawn, network sync).
            wall.sinking = true;
            wall.permanent = false;
            wall.duration = wall.time_alive + WALL_SINK_DURATION;
        }
    }
}

/// Tints wall material toward the damaged color based on remaining HP.
///
/// On first damage, clones the shared material into a per-wall instance so
/// tinting one wall doesn't affect others. Uses the `damage_tint` uniform
/// which the shader applies as a final lerp over the computed texture/noise.
pub fn update_wall_damage_tint(
    mut walls: Query<(&WallHealth, &mut MeshMaterial3d<WallOfStoneMaterial>), With<WallOfStone>>,
    mut materials: ResMut<Assets<WallOfStoneMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
) {
    let damaged = WALL_DAMAGED_COLOR.to_srgba();

    for (wall_health, mut material_handle) in &mut walls {
        if wall_health.current >= wall_health.max {
            continue;
        }

        // If still using the shared material, clone it into a per-wall instance
        if material_handle.0 == visual_assets.wall_of_stone {
            let Some(shared_mat) = materials.get(&visual_assets.wall_of_stone) else {
                continue;
            };
            let cloned = shared_mat.clone();
            material_handle.0 = materials.add(cloned);
        }

        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };

        // damage_tint.a goes from 0 (full HP) to 1 (0 HP)
        let hp_frac = wall_health.fraction();
        material.damage_tint = Vec4::new(damaged.red, damaged.green, damaged.blue, 1.0 - hp_frac);
    }
}

// =============================================================================
// Talent Systems
// =============================================================================

/// Permafrost Aura: slows enemies within range of any wall that has the talent.
pub fn apply_permafrost_aura(
    time: Res<Time>,
    mut timer: ResMut<PermafrostAuraTimer>,
    walls: Query<(&WallOfStone, &WallTalents), Without<Corpse>>,
    mut enemies: Query<
        (&Transform, &Team, Option<&mut SlowMovementModifier>, Entity),
        Without<Corpse>,
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
            Option<&mut crate::game::units::components::TemporaryHitPoints>,
        ),
        Without<Corpse>,
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

// =============================================================================
// VFX Systems
// =============================================================================

/// Animates walls rising up from the ground when first placed.
/// Moves the wall from below ground to its final position over WALL_RISE_DURATION.
pub fn animate_rising_walls(
    mut commands: Commands,
    time: Res<Time>,
    mut walls: Query<(Entity, &WallOfStone, &mut WallRising, &mut Transform)>,
) {
    let delta = time.delta_secs();
    for (entity, wall, mut rising, mut transform) in &mut walls {
        rising.elapsed += delta;
        let progress = rising.progress();

        // Ease-out: starts fast, slows at the top
        let eased = 1.0 - (1.0 - progress) * (1.0 - progress);

        // Move wall from underground to final height
        let final_y = wall.height / 2.0;
        transform.translation.y = final_y * eased - wall.height * (1.0 - eased);

        if progress >= 1.0 {
            // Snap to final position and remove rising component
            transform.translation.y = final_y;
            commands.entity(entity).remove::<WallRising>();
        }
    }
}

/// Applies trampling around a wall when it finishes rising.
/// Creates a dirt patch around the wall footprint as if the ground was churned up.
pub fn apply_wall_trampling(
    walls: Query<(&WallOfStone, &WallRising)>,
    mut grid: Option<ResMut<TramplingGrid>>,
    time: Res<Time>,
) {
    let Some(ref mut grid) = grid else {
        return;
    };
    let delta = time.delta_secs();
    let cell_size = TRAMPLING_CELL_SIZE;

    for (wall, rising) in &walls {
        // Only apply once as the wall nears the end of its rise
        let prev_progress = ((rising.elapsed - delta) / rising.duration).clamp(0.0, 1.0);
        if prev_progress >= 0.5 || rising.progress() < 0.5 {
            continue;
        }

        // Compute AABB of the wall footprint with a buffer for the disturbed area
        let buffer = 30.0;
        let bounds = wall.obstacle_bounds();
        let min_x = bounds[0] - buffer;
        let min_z = bounds[1] - buffer;
        let max_x = bounds[2] + buffer;
        let max_z = bounds[3] + buffer;

        // Iterate over grid cells in the AABB
        let mut x = min_x;
        while x <= max_x {
            let mut z = min_z;
            while z <= max_z {
                let dist = wall.distance_to_surface(Vec3::new(x, 0.0, z));
                // Strong trampling on the wall footprint, fading outward
                let intensity = if dist < 1.0 {
                    0.5
                } else {
                    (1.0 - (dist / buffer).min(1.0)) * 0.4
                };
                if intensity > 0.0
                    && let Some(idx) = grid.world_to_index(x, z)
                {
                    grid.values[idx] = (grid.values[idx] + intensity).min(1.0);
                }
                z += cell_size;
            }
            x += cell_size;
        }
        grid.dirty = true;
    }
}

/// Spawns dust puffs along walls that are rising or sinking.
pub fn spawn_wall_dust(
    mut commands: Commands,
    rising_walls: Query<(&WallOfStone, &WallRising)>,
    sinking_walls: Query<&WallOfStone, Without<WallRising>>,
    visual_assets: Res<crate::game::units::wizard::spells::visual_assets::SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < WALL_DUST_INTERVAL {
        return;
    }
    *timer -= WALL_DUST_INTERVAL;

    let t = time.elapsed_secs();

    // Spawn dust for rising walls
    for (wall, _rising) in &rising_walls {
        spawn_dust_along_wall(&mut commands, &visual_assets, wall, t);
    }

    // Spawn dust for sinking walls
    for wall in &sinking_walls {
        if wall.sinking {
            spawn_dust_along_wall(&mut commands, &visual_assets, wall, t);
        }
    }
}

/// Helper: spawns dust puffs distributed along a wall's length.
fn spawn_dust_along_wall(
    commands: &mut Commands,
    assets: &crate::game::units::wizard::spells::visual_assets::SpellVisualAssets,
    wall: &WallOfStone,
    time_secs: f32,
) {
    let wall_len = wall.half_length * 2.0;
    let num_points = ((wall_len / 50.0) as usize).max(2);

    for j in 0..num_points {
        let frac = (j as f32 + (time_secs * 2.3 + j as f32 * 1.7).fract()) / num_points as f32;
        let pos = wall.center - wall.forward * wall.half_length
            + wall.forward * (wall_len * frac.clamp(0.0, 1.0));

        crate::game::units::wizard::spells::vfx::systems::spawn_dust_smoke(
            commands,
            assets,
            pos,
            wall.half_width,
            WALL_DUST_PUFFS_PER_POINT,
            time_secs + j as f32,
        );
    }
}
