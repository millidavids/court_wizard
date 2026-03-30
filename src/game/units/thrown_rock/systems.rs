use bevy::prelude::*;

use super::components::{RockProjectile, RockShadow, ThrownRock};
use super::constants::*;
use super::messages::*;
use super::resources::ThrownRockAssets;
use crate::config::GameConfig;
use crate::game::components::{Billboard, ObstacleHealth, OnGameplayScreen};
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::shared_systems::ShadowAssets;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::pathfinding::FlowFieldVelocity;
use crate::game::plugin::GlobalAttackCycle;
use crate::game::units::components::{
    AttackTiming, Corpse, Health, Hitbox, TemporaryHitPoints,
};
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::meteor_fall::components::MeteorExplosion;
use crate::game::units::wizard::spells::squall::components::IceExplosion;

pub fn spawn_rock_projectile(
    mut commands: Commands,
    mut events: MessageReader<RockThrownMessage>,
    rock_assets: Res<ThrownRockAssets>,
) {
    for event in events.read() {
        let start_y = 20.0; // Launch from unit height
        let start = Vec3::new(event.origin.x, start_y, event.origin.z);
        let target = Vec3::new(event.target.x, 0.0, event.target.z);

        let projectile = RockProjectile {
            start,
            target,
            duration: ROCK_PROJECTILE_DURATION,
            elapsed: 0.0,
            arc_height: ROCK_PROJECTILE_ARC_HEIGHT,
        };

        let pos = projectile.current_position();

        commands.spawn((
            Mesh3d(rock_assets.mesh.clone()),
            MeshMaterial3d(rock_assets.material.clone()),
            Transform::from_translation(pos),
            projectile,
            Billboard,
            OnGameplayScreen,
        ));
    }
}

/// Animates rock projectiles along their parabolic arc and spawns landed rocks.
pub fn animate_rock_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    rock_assets: Res<ThrownRockAssets>,
    shadow_assets: Res<ShadowAssets>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    mut projectiles: Query<(Entity, &mut RockProjectile, &mut Transform)>,
    existing_rocks: Query<&ThrownRock>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let delta = time.delta_secs();

    for (entity, mut projectile, mut transform) in &mut projectiles {
        projectile.elapsed += delta;
        let pos = projectile.current_position();
        transform.translation = pos;

        if projectile.is_landed() {
            // Despawn the projectile
            commands.entity(entity).despawn();

            // Determine landing position, resolving overlaps with existing rocks
            let mut land_pos = Vec3::new(projectile.target.x, 0.0, projectile.target.z);
            land_pos = resolve_overlap(land_pos, &existing_rocks);

            let rock_y = ROCK_HEIGHT / 2.0;

            // Spawn the permanent rock obstacle
            let rock = ThrownRock {
                center: Vec3::new(land_pos.x, 0.0, land_pos.z),
                radius: ROCK_RADIUS,
                height: ROCK_HEIGHT,
                sinking: false,
                time_alive: 0.0,
                sink_deadline: f32::MAX,
            };

            let obs_bounds = rock.obstacle_bounds();
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
                obstacle_type: ObstacleType::Blocked,
                shape: Some(ObstacleShape::circle(
                    Vec2::new(rock.center.x, rock.center.z),
                    rock.radius,
                )),
                rebuild: false,
            });

            // Play boulder impact sound
            audio::play_impact_sfx(
                &mut commands,
                &sfx.boulder_impact,
                Vec3::new(land_pos.x, 0.0, land_pos.z),
                &game_config,
                &sfx,
            );

            // Spawn the permanent rock obstacle entity
            let rock_entity = commands.spawn((
                Mesh3d(rock_assets.mesh.clone()),
                MeshMaterial3d(rock_assets.material.clone()),
                Transform::from_xyz(land_pos.x, rock_y, land_pos.z),
                rock,
                ObstacleHealth::new(ROCK_HEALTH),
                Billboard,
                OnGameplayScreen,
            )).id();

            // Spawn a shadow under the rock
            commands.spawn((
                Mesh3d(shadow_assets.mesh.clone()),
                MeshMaterial3d(shadow_assets.material.clone()),
                Transform::from_xyz(land_pos.x, ROCK_SHADOW_Y, land_pos.z)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::splat(ROCK_SHADOW_SCALE)),
                RockShadow { owner: rock_entity },
                OnGameplayScreen,
            ));
        }
    }
}

/// Resolves overlap between a new rock landing position and existing rocks.
/// Pushes the new rock outward so it sits adjacent to any overlapping rock.
fn resolve_overlap(
    mut pos: Vec3,
    existing_rocks: &Query<&ThrownRock>,
) -> Vec3 {
    // Iterate a few times to resolve cascading overlaps
    for _ in 0..8 {
        let mut adjusted = false;
        for rock in existing_rocks.iter() {
            if rock.sinking {
                continue;
            }
            let dx = pos.x - rock.center.x;
            let dz = pos.z - rock.center.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if dist < ROCK_MIN_SEPARATION {
                if dist < 0.001 {
                    // Exactly overlapping, push in arbitrary direction
                    pos.x += ROCK_MIN_SEPARATION;
                } else {
                    let push = (ROCK_MIN_SEPARATION - dist) / dist;
                    pos.x += dx * push;
                    pos.z += dz * push;
                }
                adjusted = true;
            }
        }
        if !adjusted {
            break;
        }
    }
    pos
}

/// Ticks rock lifetime and handles sinking animation.
pub fn tick_rock_lifetime(
    time: Res<Time>,
    mut rocks: Query<(&mut ThrownRock, &mut Transform)>,
) {
    let delta = time.delta_secs();

    for (mut rock, mut transform) in &mut rocks {
        rock.time_alive += delta;

        if rock.sinking {
            let sink_progress =
                ((rock.time_alive - (rock.sink_deadline - ROCK_SINK_DURATION)) / ROCK_SINK_DURATION)
                    .clamp(0.0, 1.0);
            // Sink from rock_y down to underground
            let rock_y = (ROCK_HEIGHT / 2.0) * (1.0 - sink_progress);
            transform.translation.y = rock_y;
        }
    }
}

pub fn cleanup_sunk_rocks(
    mut commands: Commands,
    rocks: Query<(Entity, &ThrownRock)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, rock) in &rocks {
        if rock.sinking && rock.time_alive >= rock.sink_deadline {
            let obs_bounds = rock.obstacle_bounds();
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::circle(
                    Vec2::new(rock.center.x, rock.center.z),
                    rock.radius,
                )),
                rebuild: false,
            });

            commands.entity(entity).despawn();
        }
    }
}

pub fn destroy_dead_rocks(mut rocks: Query<(&mut ThrownRock, &ObstacleHealth)>) {
    for (mut rock, health) in &mut rocks {
        if health.is_dead() && !rock.sinking {
            rock.sinking = true;
            rock.sink_deadline = rock.time_alive + ROCK_SINK_DURATION;
        }
    }
}

/// Units with no valid path attack nearby rocks (same pattern as wall of stone).
pub fn units_attack_blocking_rocks(
    attack_cycle: Res<GlobalAttackCycle>,
    mut blocked_units: Query<
        (
            &Transform,
            &Hitbox,
            &FlowFieldVelocity,
            &mut AttackTiming,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (Without<Corpse>, Without<ThrownRock>),
    >,
    mut rocks: Query<(Entity, &ThrownRock, &mut ObstacleHealth)>,
) {
    let current_time = attack_cycle.current_time;
    let last_time = (current_time - crate::game::constants::APPROX_FRAME_TIME).max(0.0);

    for (transform, hitbox, flow_vel, mut attack_timing, _health, _temp_hp) in &mut blocked_units {
        // Only target rocks if this unit has no valid path
        if !flow_vel.pathfinding_distance.is_infinite() {
            continue;
        }

        let unit_pos = transform.translation;

        // Find nearest rock by distance to surface
        let mut nearest_rock_entity = None;
        let mut nearest_distance = f32::MAX;

        for (entity, rock, _) in rocks.iter() {
            if rock.sinking {
                continue;
            }
            let dist = rock.distance_to_surface(unit_pos);
            if dist < nearest_distance {
                nearest_distance = dist;
                nearest_rock_entity = Some(entity);
            }
        }

        // Deal damage if close enough to a rock
        let attack_range = hitbox.radius + ROCK_ATTACK_RANGE;
        if let Some(rock_entity) = nearest_rock_entity
            && nearest_distance <= attack_range
            && attack_timing.can_attack(current_time, last_time)
            && let Ok((_, _, mut rock_health)) = rocks.get_mut(rock_entity)
        {
            rock_health.take_damage(ROCK_DAMAGE_PER_HIT);
            attack_timing.record_attack(current_time);
        }
    }
}

/// Tints rock material from base color to damaged color based on remaining HP.
/// Clones the shared material on first damage so tinting one rock doesn't affect others.
pub fn update_rock_damage_tint(
    mut rocks: Query<(&ObstacleHealth, &mut MeshMaterial3d<StandardMaterial>), With<ThrownRock>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    rock_assets: Res<ThrownRockAssets>,
) {
    let base = ROCK_BASE_COLOR.to_srgba();
    let damaged = ROCK_DAMAGED_COLOR.to_srgba();

    for (health, mut material_handle) in &mut rocks {
        if health.current >= health.max {
            continue;
        }

        // Clone shared material into per-rock instance on first damage
        if material_handle.0 == rock_assets.material {
            let Some(shared_mat) = materials.get(&rock_assets.material) else {
                continue;
            };
            let cloned = shared_mat.clone();
            material_handle.0 = materials.add(cloned);
        }

        let Some(mat) = materials.get_mut(&material_handle.0) else {
            continue;
        };

        let fraction = health.fraction();
        let r = damaged.red + (base.red - damaged.red) * fraction;
        let g = damaged.green + (base.green - damaged.green) * fraction;
        let b = damaged.blue + (base.blue - damaged.blue) * fraction;
        mat.base_color = Color::srgba(r, g, b, 1.0);
    }
}

pub fn cleanup_rock_shadows(
    mut commands: Commands,
    shadows: Query<(Entity, &RockShadow)>,
    rocks: Query<Entity, With<ThrownRock>>,
) {
    for (shadow_entity, shadow) in &shadows {
        if rocks.get(shadow.owner).is_err() {
            commands.entity(shadow_entity).try_despawn();
        }
    }
}

/// Applies damage to rocks from spell AoE explosions (fireball, meteor, squall).
pub fn apply_spell_damage_to_rocks(
    fireball_explosions: Query<&FireballExplosion>,
    meteor_explosions: Query<&MeteorExplosion>,
    ice_explosions: Query<&IceExplosion>,
    mut rocks: Query<(&ThrownRock, &mut ObstacleHealth)>,
) {
    let xz_distance = |a: Vec3, b: Vec3| -> f32 {
        let dx = a.x - b.x;
        let dz = a.z - b.z;
        (dx * dx + dz * dz).sqrt()
    };

    for (rock, mut health) in &mut rocks {
        if rock.sinking || health.is_dead() {
            continue;
        }

        for explosion in &fireball_explosions {
            if explosion.damage_per_tick > 0.0
                && xz_distance(explosion.origin, rock.center) <= explosion.current_radius() + rock.radius
            {
                health.take_damage(explosion.damage_per_tick);
            }
        }

        for explosion in &meteor_explosions {
            if !explosion.damage_applied
                && xz_distance(explosion.origin, rock.center) <= explosion.max_radius + rock.radius
            {
                health.take_damage(explosion.damage);
            }
        }

        for explosion in &ice_explosions {
            if !explosion.damage_applied
                && xz_distance(explosion.origin, rock.center) <= explosion.max_radius + rock.radius
            {
                health.take_damage(explosion.damage);
            }
        }
    }
}
