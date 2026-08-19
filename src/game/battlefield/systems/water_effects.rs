use bevy::prelude::*;

use crate::game::battlefield::components::{WaterRipple, WaterRippleAssets};
use crate::game::battlefield::constants::*;
use crate::game::components::OnGameplayScreen;

/// Spawns growing, fading annulus ripples on the water pool.
pub fn emit_water_ripples(
    mut commands: Commands,
    ripple_assets: Res<WaterRippleAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < WATER_RIPPLE_INTERVAL {
        return;
    }
    *timer -= WATER_RIPPLE_INTERVAL;

    let t = time.elapsed_secs();

    // Spawn 1 ripple per interval at a random position within the pool
    let seed = t * 3.7;
    let angle = seed * 2.39;
    let dist_frac = (seed * 17.3).sin() * 0.5 + 0.5;
    let x = WATER_POOL_POSITION.x + angle.cos() * WATER_POOL_RADIUS * dist_frac * 0.5;
    let z = WATER_POOL_POSITION.z + angle.sin() * WATER_POOL_RADIUS * dist_frac * 0.5;

    let max_scale = WATER_RIPPLE_MAX_SCALE * (0.6 + 0.4 * ((seed * 41.7).sin() * 0.5 + 0.5));
    let lifetime = WATER_RIPPLE_LIFETIME * (0.8 + 0.4 * ((seed * 23.1).sin() * 0.5 + 0.5));

    // Each ripple gets its own material so we can fade alpha independently
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.9, 0.95, 1.0, WATER_RIPPLE_ALPHA),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Mesh3d(ripple_assets.mesh.clone()),
        MeshMaterial3d(mat),
        Transform::from_xyz(x, WATER_POOL_POSITION.y + 2.0, z)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(10.0)),
        WaterRipple {
            lifetime: 0.0,
            max_lifetime: lifetime,
            max_scale,
        },
        OnGameplayScreen,
    ));
}

/// Grows and fades water ripples over their lifetime.
pub fn update_water_ripples(
    mut commands: Commands,
    mut ripples: Query<(
        Entity,
        &mut WaterRipple,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();

    for (entity, mut ripple, mut transform, mesh_material) in &mut ripples {
        ripple.lifetime += delta;
        if ripple.lifetime >= ripple.max_lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let t = ripple.lifetime / ripple.max_lifetime;

        // Grow from small to max_scale
        let scale = 10.0 + t * ripple.max_scale;
        transform.scale = Vec3::splat(scale);

        // Fade alpha: ramp up quickly then fade out
        let alpha = if t < 0.1 {
            t / 0.1
        } else {
            1.0 - (t - 0.1) / 0.9
        } * WATER_RIPPLE_ALPHA;

        if let Some(mut mat) = materials.get_mut(&mesh_material.0) {
            mat.base_color = Color::srgba(0.9, 0.95, 1.0, alpha);
        }
    }
}
