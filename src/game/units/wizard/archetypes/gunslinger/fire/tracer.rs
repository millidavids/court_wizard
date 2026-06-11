use bevy::prelude::*;

use super::super::components::*;
use super::super::constants;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Move bullet tracers in a straight line.
pub fn move_tracers(time: Res<Time>, mut tracers: Query<(&mut Transform, &BulletTracer)>) {
    for (mut transform, tracer) in &mut tracers {
        transform.translation += tracer.velocity * time.delta_secs();
    }
}

/// Despawn tracers that have traveled beyond their range.
pub fn despawn_distant_tracers(
    mut commands: Commands,
    tracers: Query<(Entity, &Transform, &BulletTracer)>,
) {
    for (entity, transform, tracer) in &tracers {
        if transform.translation.distance(tracer.origin) > tracer.max_range {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Update and despawn muzzle flashes.
pub fn update_muzzle_flashes(
    time: Res<Time>,
    mut commands: Commands,
    mut flashes: Query<(Entity, &mut MuzzleFlash, &mut Transform)>,
) {
    for (entity, mut flash, mut transform) in &mut flashes {
        flash.timer -= time.delta_secs();
        if flash.timer <= 0.0 {
            commands.entity(entity).try_despawn();
        } else {
            let scale =
                (flash.timer / constants::MUZZLE_FLASH_DURATION) * constants::MUZZLE_FLASH_SIZE;
            transform.scale = Vec3::splat(scale);
        }
    }
}

pub(crate) fn spawn_tracer(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    velocity: Vec3,
    max_range: f32,
) -> Entity {
    // Orient the cylinder along the velocity direction for a bullet-line look
    let dir = velocity.normalize_or_zero();
    let rotation = Quat::from_rotation_arc(Vec3::Y, dir);
    commands
        .spawn((
            Mesh3d(assets.cross_plane_cylinder.clone()),
            MeshMaterial3d(assets.bullet_tracer.clone()),
            Transform::from_translation(position)
                .with_rotation(rotation)
                .with_scale(Vec3::new(
                    constants::BULLET_RADIUS,
                    constants::BULLET_LENGTH,
                    constants::BULLET_RADIUS,
                )),
            BulletTracer {
                velocity,
                max_range,
                origin: position,
            },
            crate::game::components::OnGameplayScreen,
        ))
        .id()
}

pub(crate) fn spawn_muzzle_flash(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    gun: GunType,
) -> Entity {
    commands
        .spawn((
            Mesh3d(assets.cross_plane_sphere.clone()),
            MeshMaterial3d(assets.fireball_projectile.clone()),
            Transform::from_translation(position)
                .with_scale(Vec3::splat(constants::MUZZLE_FLASH_SIZE)),
            MuzzleFlash {
                timer: constants::MUZZLE_FLASH_DURATION,
                gun,
            },
            crate::game::components::OnGameplayScreen,
        ))
        .id()
}
