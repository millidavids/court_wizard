use bevy::prelude::*;

use super::components::{Boulder, BoulderHeat, BoulderProjectile, BoulderShadow, ClonedMaterial};
use super::constants::*;
use super::messages::*;
use super::resources::BoulderAssets;
use crate::config::GameConfig;
use crate::game::components::{Billboard, ObstacleHealth, OnGameplayScreen};
use crate::game::pathfinding::FlowFieldVelocity;
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::plugin::GlobalAttackCycle;
use crate::game::shared_systems::ShadowAssets;
use crate::game::units::components::{
    AttackTiming, Corpse, Health, Hitbox, Teleportable, TemporaryHitPoints,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::meteor_fall::components::MeteorExplosion;
use crate::game::units::wizard::spells::squall::components::IceExplosion;

pub fn spawn_rock_projectile(
    mut commands: Commands,
    mut events: MessageReader<BoulderThrownMessage>,
    rock_assets: Res<BoulderAssets>,
) {
    for event in events.read() {
        let start_y = 20.0; // Launch from unit height
        let start = Vec3::new(event.origin.x, start_y, event.origin.z);
        let target = Vec3::new(event.target.x, 0.0, event.target.z);
        let idx = (event.sprite_index as usize).min(BOULDER_SPRITE_COUNT - 1);

        let projectile = BoulderProjectile {
            start,
            target,
            duration: ROCK_PROJECTILE_DURATION,
            elapsed: 0.0,
            arc_height: ROCK_PROJECTILE_ARC_HEIGHT,
            sprite_index: event.sprite_index,
        };

        let pos = projectile.current_position();

        commands.spawn((
            Mesh3d(rock_assets.mesh.clone()),
            MeshMaterial3d(rock_assets.materials[idx].clone()),
            Transform::from_translation(pos),
            projectile,
            OnGameplayScreen,
        ));
    }
}

/// Animates boulder projectiles along their parabolic arc and spawns landed boulders.
#[allow(clippy::too_many_arguments)]
pub fn animate_rock_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    rock_assets: Res<BoulderAssets>,
    shadow_assets: Res<ShadowAssets>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    mut projectiles: Query<(Entity, &mut BoulderProjectile, &mut Transform)>,
    existing_rocks: Query<&Boulder>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let delta = time.delta_secs();

    for (entity, mut projectile, mut transform) in &mut projectiles {
        projectile.elapsed += delta;
        let pos = projectile.current_position();
        transform.translation = pos;

        transform.rotate_z(BOULDER_SPIN_SPEED * delta);

        if projectile.is_landed() {
            // Despawn the projectile
            commands.entity(entity).despawn();

            // Determine landing position, resolving overlaps with existing boulders
            let mut land_pos = Vec3::new(projectile.target.x, 0.0, projectile.target.z);
            land_pos = resolve_overlap(land_pos, &existing_rocks);

            let rock_y = BOULDER_SPRITE_HEIGHT / 2.0 - BOULDER_GROUND_CLIP;
            let idx = (projectile.sprite_index as usize).min(BOULDER_SPRITE_COUNT - 1);

            // Spawn the permanent boulder obstacle
            let rock = Boulder {
                center: Vec3::new(land_pos.x, 0.0, land_pos.z),
                radius: ROCK_RADIUS,
                height: ROCK_HEIGHT,
                sinking: false,
                time_alive: 0.0,
                sink_deadline: f32::MAX,
                sprite_index: projectile.sprite_index,
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

            // Landed boulder spawns upright (no rotation carried over from spin)
            let rock_entity = commands
                .spawn((
                    Mesh3d(rock_assets.mesh.clone()),
                    MeshMaterial3d(rock_assets.materials[idx].clone()),
                    Transform::from_xyz(land_pos.x, rock_y, land_pos.z),
                    rock,
                    ObstacleHealth::new(ROCK_HEALTH),
                    Billboard,
                    Teleportable,
                    OnGameplayScreen,
                ))
                .id();

            // Spawn a shadow under the boulder
            commands.spawn((
                Mesh3d(shadow_assets.mesh.clone()),
                MeshMaterial3d(shadow_assets.material.clone()),
                Transform::from_xyz(land_pos.x, ROCK_SHADOW_Y, land_pos.z)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::splat(ROCK_SHADOW_SCALE)),
                BoulderShadow { owner: rock_entity },
                OnGameplayScreen,
            ));
        }
    }
}

/// Resolves overlap between a new boulder landing position and existing boulders.
/// Pushes the new boulder outward so it sits adjacent to any overlapping boulder.
fn resolve_overlap(mut pos: Vec3, existing_rocks: &Query<&Boulder>) -> Vec3 {
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

/// Ticks boulder lifetime and handles sinking animation.
pub fn tick_rock_lifetime(time: Res<Time>, mut rocks: Query<(&mut Boulder, &mut Transform)>) {
    let delta = time.delta_secs();

    for (mut rock, mut transform) in &mut rocks {
        rock.time_alive += delta;

        if rock.sinking {
            let sink_progress = ((rock.time_alive - (rock.sink_deadline - ROCK_SINK_DURATION))
                / ROCK_SINK_DURATION)
                .clamp(0.0, 1.0);
            // Sink from sprite position down to underground
            let base_y = BOULDER_SPRITE_HEIGHT / 2.0 - BOULDER_GROUND_CLIP;
            transform.translation.y = base_y * (1.0 - sink_progress);
        }
    }
}

pub fn cleanup_sunk_rocks(
    mut commands: Commands,
    rocks: Query<(Entity, &Boulder)>,
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

pub fn destroy_dead_rocks(mut rocks: Query<(&mut Boulder, &ObstacleHealth)>) {
    for (mut rock, health) in &mut rocks {
        if health.is_dead() && !rock.sinking {
            rock.sinking = true;
            rock.sink_deadline = rock.time_alive + ROCK_SINK_DURATION;
        }
    }
}

/// Units with no valid path attack nearby boulders (same pattern as wall of stone).
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
        (Without<Corpse>, Without<Boulder>),
    >,
    mut rocks: Query<(Entity, &Boulder, &mut ObstacleHealth)>,
) {
    let current_time = attack_cycle.current_time;
    let last_time = (current_time - crate::game::constants::APPROX_FRAME_TIME).max(0.0);

    for (transform, hitbox, flow_vel, mut attack_timing, _health, _temp_hp) in &mut blocked_units {
        // Only target boulders if this unit has no valid path
        if !flow_vel.pathfinding_distance.is_infinite() {
            continue;
        }

        let unit_pos = transform.translation;

        // Find nearest boulder by distance to surface
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

        // Deal damage if close enough to a boulder
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

/// Tints boulder sprite from white (full HP) toward a dark color (zero HP).
/// Clones the shared material on first damage so tinting one boulder doesn't affect others.
pub fn update_rock_damage_tint(
    mut commands: Commands,
    mut rocks: Query<
        (
            Entity,
            &ObstacleHealth,
            &mut MeshMaterial3d<StandardMaterial>,
            Has<ClonedMaterial>,
        ),
        With<Boulder>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let base = Color::WHITE.to_srgba();
    let damaged = ROCK_DAMAGED_COLOR.to_srgba();

    for (entity, health, mut material_handle, already_cloned) in &mut rocks {
        if health.current >= health.max {
            continue;
        }

        if !already_cloned {
            let Some(shared_mat) = materials.get(&material_handle.0) else {
                continue;
            };
            let cloned = shared_mat.clone();
            material_handle.0 = materials.add(cloned);
            commands.entity(entity).insert(ClonedMaterial);
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
    shadows: Query<(Entity, &BoulderShadow)>,
    rocks: Query<Entity, With<Boulder>>,
) {
    for (shadow_entity, shadow) in &shadows {
        if rocks.get(shadow.owner).is_err() {
            commands.entity(shadow_entity).try_despawn();
        }
    }
}

/// Applies damage to boulders from spell AoE explosions (fireball, meteor, squall).
pub fn apply_spell_damage_to_rocks(
    fireball_explosions: Query<&FireballExplosion>,
    meteor_explosions: Query<&MeteorExplosion>,
    ice_explosions: Query<&IceExplosion>,
    mut rocks: Query<(&Boulder, &mut ObstacleHealth)>,
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
                && xz_distance(explosion.origin, rock.center)
                    <= explosion.current_radius() + rock.radius
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

/// Bleeds off accumulated fire heat after `BOULDER_HEAT_DECAY_DELAY` seconds of no fire contribution.
/// Removes the `BoulderHeat` component when heat reaches zero.
pub fn tick_boulder_heat(
    mut commands: Commands,
    time: Res<Time>,
    mut boulders: Query<(Entity, &mut BoulderHeat)>,
) {
    let delta = time.delta_secs();
    for (entity, mut heat) in &mut boulders {
        if heat.decay_delay > 0.0 {
            heat.decay_delay = (heat.decay_delay - delta).max(0.0);
            continue;
        }
        heat.heat = (heat.heat - BOULDER_HEAT_DECAY_RATE * delta).max(0.0);
        if heat.heat <= 0.0 {
            commands.entity(entity).remove::<BoulderHeat>();
        }
    }
}

/// Detects boulders whose Transform was moved externally (e.g. by teleport) and
/// updates the Boulder.center + pathfinding grid to match.
pub fn sync_teleported_rocks(
    mut rocks: Query<(&mut Boulder, &Transform), Changed<Transform>>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (mut rock, transform) in &mut rocks {
        let new_center = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
        if (rock.center - new_center).length_squared() < 0.01 {
            continue;
        }

        // Remove obstacle at old position
        let old_bounds = rock.obstacle_bounds();
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(old_bounds[0], old_bounds[1], old_bounds[2], old_bounds[3]),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::circle(
                Vec2::new(rock.center.x, rock.center.z),
                rock.radius,
            )),
            rebuild: false,
        });

        // Update center
        rock.center = new_center;

        // Add obstacle at new position
        let new_bounds = rock.obstacle_bounds();
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(new_bounds[0], new_bounds[1], new_bounds[2], new_bounds[3]),
            obstacle_type: ObstacleType::Blocked,
            shape: Some(ObstacleShape::circle(
                Vec2::new(rock.center.x, rock.center.z),
                rock.radius,
            )),
            rebuild: false,
        });
    }
}

/// Spawns a pre-placed terrain boulder (identical to a landed thrown boulder).
/// Used by the terrain generation system, not by brute/ogre throws.
#[allow(clippy::too_many_arguments)]
pub(in crate::game) fn spawn_terrain_boulder(
    commands: &mut Commands,
    rock_assets: &BoulderAssets,
    shadow_assets: &ShadowAssets,
    x: f32,
    z: f32,
    scale: f32,
    sprite_index: u8,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    let radius = ROCK_RADIUS * scale;
    let rock_y = BOULDER_SPRITE_HEIGHT * scale / 2.0 - BOULDER_GROUND_CLIP;
    let idx = (sprite_index as usize).min(BOULDER_SPRITE_COUNT - 1);

    let rock = Boulder {
        center: Vec3::new(x, 0.0, z),
        radius,
        height: ROCK_HEIGHT * scale,
        sinking: false,
        time_alive: 0.0,
        sink_deadline: f32::MAX,
        sprite_index,
    };

    let center_xz = Vec2::new(x, z);
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_half_size(center_xz, Vec2::splat(radius)),
        obstacle_type: ObstacleType::Blocked,
        shape: Some(ObstacleShape::circle(center_xz, radius)),
        rebuild: false,
    });

    let rock_entity = commands
        .spawn((
            Mesh3d(rock_assets.mesh.clone()),
            MeshMaterial3d(rock_assets.materials[idx].clone()),
            Transform::from_xyz(x, rock_y, z).with_scale(Vec3::splat(scale)),
            rock,
            ObstacleHealth::new(ROCK_HEALTH * scale),
            Billboard,
            Teleportable,
            OnGameplayScreen,
        ))
        .id();

    commands.spawn((
        Mesh3d(shadow_assets.mesh.clone()),
        MeshMaterial3d(shadow_assets.material.clone()),
        Transform::from_xyz(x, ROCK_SHADOW_Y, z)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(ROCK_SHADOW_SCALE * scale)),
        BoulderShadow { owner: rock_entity },
        OnGameplayScreen,
    ));
}
