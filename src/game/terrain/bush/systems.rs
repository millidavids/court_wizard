use bevy::prelude::*;

use super::components::{BurningBush, Bush};
use super::constants::*;
use super::resources::BushAssets;
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::shared_systems::{ShadowAssets, spawn_terrain_shadow};
use crate::game::terrain::utils::{
    BURNING_VEGETATION_TINT, apply_heat_zone_fire_dot, emit_burning_vfx, should_ignite_from_fire,
};
use crate::game::terrain::wind_sway::material::WindSwayMaterial;
use crate::game::units::components::{Corpse, FireDoT, RoughTerrainModifier};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Spawns a single bush at the given position with a size multiplier.
#[allow(clippy::too_many_arguments)]
pub(in crate::game) fn spawn_single_bush(
    commands: &mut Commands,
    assets: &BushAssets,
    shadow_assets: &ShadowAssets,
    x: f32,
    z: f32,
    scale: f32,
    sprite_index: u8,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    let center = Vec3::new(x, 0.0, z);
    let radius = BUSH_RADIUS * scale;
    let idx = (sprite_index as usize).min(BUSH_SPRITE_COUNT - 1);

    commands.spawn((
        Mesh3d(assets.mesh.clone()),
        MeshMaterial3d(assets.materials[idx].clone()),
        Transform::from_xyz(x, BUSH_SPRITE_HEIGHT * scale / 2.0 - BUSH_GROUND_CLIP, z)
            .with_scale(Vec3::splat(scale)),
        Bush {
            center,
            radius,
            sprite_index,
        },
        Billboard,
        OnGameplayScreen,
    ));

    spawn_terrain_shadow(commands, shadow_assets, x, z, BUSH_SHADOW_SCALE * scale);

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

/// Ignites bushes hit by fire spell explosions or disintegrate beams.
#[allow(clippy::too_many_arguments)]
pub fn ignite_bushes_from_fire(
    mut commands: Commands,
    bushes: Query<(Entity, &Bush), Without<BurningBush>>,
    explosions: Query<&crate::game::units::wizard::spells::fireball::components::FireballExplosion>,
    meteor_explosions: Query<
        &crate::game::units::wizard::spells::meteor_fall::components::MeteorExplosion,
    >,
    beams: Query<&crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam>,
    walls: Query<&crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect>,
    ground_fires: Query<
        &crate::game::units::wizard::spells::meteor_fall::components::MeteorGroundFire,
    >,
    bush_assets: Res<BushAssets>,
    mut materials: ResMut<Assets<WindSwayMaterial>>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, bush) in &bushes {
        if !should_ignite_from_fire(
            bush.center,
            bush.radius,
            &explosions,
            &meteor_explosions,
            &beams,
            &walls,
            &ground_fires,
        ) {
            continue;
        }

        commands
            .entity(entity)
            .insert(BurningBush { tick_timer: 0.0 });

        let idx = (bush.sprite_index as usize).min(BUSH_SPRITE_COUNT - 1);
        if let Some(shared_mat) = materials.get(&bush_assets.materials[idx]) {
            let mut tinted = shared_mat.clone();
            tinted.base_color = BURNING_VEGETATION_TINT.to_linear();
            let new_mat = materials.add(tinted);
            commands.entity(entity).insert(MeshMaterial3d(new_mat));
        }

        let center_xz = Vec2::new(bush.center.x, bush.center.z);
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_half_size(center_xz, Vec2::splat(bush.radius)),
            obstacle_type: ObstacleType::SlowTerrain(BURNING_BUSH_FLOW_COST),
            shape: Some(ObstacleShape::circle(center_xz, bush.radius)),
            rebuild: false,
        });
    }
}

/// Burning bushes deal periodic fire damage to units inside them.
pub fn apply_burning_bush_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut burning_bushes: Query<(&Bush, &mut BurningBush)>,
    mut units: Query<
        (Entity, &Transform, Option<&mut FireDoT>, Has<SpellShield>),
        (
            Without<Corpse>,
            // Staging attackers are spell-immune, and terrain fire spread by
            // spells must not burn them either (lava is the only sanctioned
            // environmental damage during staging).
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (bush, mut burning) in &mut burning_bushes {
        burning.tick_timer += delta;
        if burning.tick_timer < BURNING_BUSH_TICK_INTERVAL {
            continue;
        }
        burning.tick_timer -= BURNING_BUSH_TICK_INTERVAL;

        let heat_radius = bush.radius * BURNING_BUSH_HEAT_RADIUS_MULTIPLIER;
        let heat_radius_sq = heat_radius * heat_radius;

        for (entity, transform, fire_dot, has_shield) in &mut units {
            let dx = transform.translation.x - bush.center.x;
            let dz = transform.translation.z - bush.center.z;
            if dx * dx + dz * dz <= heat_radius_sq {
                apply_heat_zone_fire_dot(
                    &mut commands,
                    entity,
                    fire_dot,
                    has_shield,
                    BURNING_BUSH_HEAT_SPELL_DAMAGE,
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
        emit_burning_vfx(
            &mut commands,
            &visual_assets,
            bush.center.x,
            bush.center.z,
            BUSH_SPRITE_HEIGHT,
            bush.radius,
            2,
            2,
            t,
            emit_smoke,
            emit_sparks,
        );
    }
}
