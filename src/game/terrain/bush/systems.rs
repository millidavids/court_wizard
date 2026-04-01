use bevy::prelude::*;

use super::components::{BurningBush, Bush};
use super::constants::*;
use super::resources::BushAssets;
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::shared_systems::{ShadowAssets, spawn_terrain_shadow};
use crate::game::units::components::{Corpse, Health, RoughTerrainModifier};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Spawns a single bush at the given position with a size multiplier.
pub(in crate::game) fn spawn_single_bush(
    commands: &mut Commands,
    assets: &BushAssets,
    shadow_assets: &ShadowAssets,
    x: f32,
    z: f32,
    scale: f32,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    let center = Vec3::new(x, 0.0, z);
    let radius = BUSH_RADIUS * scale;

    commands.spawn((
        Mesh3d(assets.mesh.clone()),
        MeshMaterial3d(assets.material.clone()),
        Transform::from_xyz(x, BUSH_HEIGHT * scale / 2.0 + 1.0, z).with_scale(Vec3::splat(scale)),
        Bush { center, radius },
        Billboard,
        OnGameplayScreen,
    ));

    spawn_terrain_shadow(commands, shadow_assets, x, z, 0.9 * scale);

    let center_xz = Vec2::new(x, z);
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_half_size(center_xz, Vec2::splat(radius)),
        obstacle_type: ObstacleType::SlowTerrain(BUSH_FLOW_COST),
        shape: Some(ObstacleShape::circle(center_xz, radius)),
        rebuild: false,
    });
}

/// Applies speed slow to units inside non-burning bushes.
pub fn apply_bush_slow(
    bushes: Query<&Bush, Without<BurningBush>>,
    mut units: Query<(&Transform, &mut RoughTerrainModifier), Without<Corpse>>,
) {
    for (transform, mut terrain_mod) in &mut units {
        let unit_xz = Vec2::new(transform.translation.x, transform.translation.z);
        let in_bush = bushes.iter().any(|bush| {
            let bush_xz = Vec2::new(bush.center.x, bush.center.z);
            unit_xz.distance_squared(bush_xz) <= bush.radius * bush.radius
        });
        if in_bush && terrain_mod.0 > BUSH_SPEED_MODIFIER {
            terrain_mod.0 = BUSH_SPEED_MODIFIER;
        }
    }
}

/// Ignites bushes hit by fire spell explosions or disintegrate beams. Once burning, they stay on fire.
pub fn ignite_bushes_from_fire(
    mut commands: Commands,
    bushes: Query<(Entity, &Bush), Without<BurningBush>>,
    explosions: Query<&crate::game::units::wizard::spells::fireball::components::FireballExplosion>,
    meteor_explosions: Query<
        &crate::game::units::wizard::spells::meteor_fall::components::MeteorExplosion,
    >,
    beams: Query<&crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let xz_distance = |a: Vec3, b: Vec3| -> f32 {
        let dx = a.x - b.x;
        let dz = a.z - b.z;
        (dx * dx + dz * dz).sqrt()
    };

    for (entity, bush) in &bushes {
        let mut should_ignite = false;

        // Check fireball explosions
        for explosion in &explosions {
            if explosion.damage_per_tick > 0.0
                && xz_distance(explosion.origin, bush.center)
                    <= explosion.current_radius() + bush.radius
            {
                should_ignite = true;
                break;
            }
        }

        // Check meteor explosions
        if !should_ignite {
            for explosion in &meteor_explosions {
                if !explosion.damage_applied
                    && xz_distance(explosion.origin, bush.center)
                        <= explosion.max_radius + bush.radius
                {
                    should_ignite = true;
                    break;
                }
            }
        }

        // Check disintegrate beams
        if !should_ignite {
            for beam in &beams {
                if beam.contains_point_with_radius(bush.center, bush.radius) {
                    should_ignite = true;
                    break;
                }
            }
        }

        if should_ignite {
            // Add burning component
            commands
                .entity(entity)
                .insert(BurningBush { tick_timer: 0.0 });

            let new_mat = materials.add(StandardMaterial {
                base_color: BURNING_BUSH_COLOR,
                unlit: true,
                ..default()
            });
            commands.entity(entity).insert(MeshMaterial3d(new_mat));

            let center_xz = Vec2::new(bush.center.x, bush.center.z);
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_half_size(center_xz, Vec2::splat(bush.radius)),
                obstacle_type: ObstacleType::SlowTerrain(BURNING_BUSH_FLOW_COST),
                shape: Some(ObstacleShape::circle(center_xz, bush.radius)),
                rebuild: false,
            });
        }
    }
}

/// Burning bushes deal periodic fire damage to units inside them.
pub fn apply_burning_bush_damage(
    time: Res<Time>,
    mut burning_bushes: Query<(&Bush, &mut BurningBush)>,
    mut units: Query<
        (
            &Transform,
            &mut Health,
            Option<&mut crate::game::units::components::TemporaryHitPoints>,
        ),
        Without<Corpse>,
    >,
) {
    let delta = time.delta_secs();

    for (bush, mut burning) in &mut burning_bushes {
        burning.tick_timer += delta;
        if burning.tick_timer < BURNING_BUSH_TICK_INTERVAL {
            continue;
        }
        burning.tick_timer -= BURNING_BUSH_TICK_INTERVAL;

        let damage = BURNING_BUSH_DPS * BURNING_BUSH_TICK_INTERVAL;

        for (transform, mut health, mut temp_hp) in &mut units {
            let dx = transform.translation.x - bush.center.x;
            let dz = transform.translation.z - bush.center.z;
            if dx * dx + dz * dz <= bush.radius * bush.radius {
                crate::game::units::components::apply_damage_to_unit(
                    &mut health,
                    temp_hp.as_deref_mut(),
                    damage,
                );
            }
        }
    }
}

/// Emits fire smoke and spark particles from burning bushes.
pub fn emit_burning_bush_vfx(
    mut commands: Commands,
    burning_bushes: Query<&Bush, With<BurningBush>>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut smoke_timer: Local<f32>,
    mut spark_timer: Local<f32>,
) {
    let delta = time.delta_secs();
    let t = time.elapsed_secs();

    *smoke_timer += delta;
    *spark_timer += delta;

    let emit_smoke = *smoke_timer >= BURNING_BUSH_SMOKE_INTERVAL;
    let emit_sparks = *spark_timer >= BURNING_BUSH_SPARK_INTERVAL;

    if emit_smoke {
        *smoke_timer -= BURNING_BUSH_SMOKE_INTERVAL;
    }
    if emit_sparks {
        *spark_timer -= BURNING_BUSH_SPARK_INTERVAL;
    }

    for bush in &burning_bushes {
        if emit_smoke {
            vfx::systems::spawn_fire_orange_smoke(
                &mut commands,
                &visual_assets,
                Vec3::new(bush.center.x, 0.0, bush.center.z),
                bush.radius,
                2,
                t + bush.center.x * 0.1,
            );
        }
        if emit_sparks {
            vfx::systems::spawn_fire_sparks(
                &mut commands,
                &visual_assets,
                Vec3::new(bush.center.x, 5.0, bush.center.z),
                3,
                t + bush.center.z * 0.1,
            );
        }
    }
}
