//! Generic "hit flash" visual: a brief white overlay that pops on a unit
//! when it takes a discrete hit (bullet, sword swing, etc). Insert
//! `HitFlash` on the target entity; `update_hit_flashes` spawns the overlay
//! `HitFlashVfx` on the next frame and `update_hit_flash_vfx` fades it out.

use bevy::prelude::*;

use super::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::components::OnGameplayScreen;

/// Marker for the unit-side flash. The `timer` field gates how long the
/// `HitFlash` component stays attached to the unit; pick whatever value the
/// caller likes (it doesn't affect VFX duration).
#[derive(Component)]
pub struct HitFlash {
    pub timer: f32,
}

/// The visible white overlay entity that gets spawned alongside the hit
/// unit. Always uses `HIT_FLASH_VFX_DURATION` for its fade.
#[derive(Component)]
pub struct HitFlashVfx {
    pub timer: f32,
}

/// Lifetime of the white overlay in seconds.
pub const HIT_FLASH_VFX_DURATION: f32 = 0.08;

/// Initial scale of the overlay sphere (in world units).
const VFX_SCALE: f32 = 30.0;

/// Spawns the overlay on the frame `HitFlash` is added, then ticks the
/// flag's timer and removes it once expired.
pub fn update_hit_flashes(
    time: Res<Time>,
    mut commands: Commands,
    mut flashes: Query<(Entity, &mut HitFlash, &Transform)>,
    visual_assets: Res<SpellVisualAssets>,
) {
    for (entity, mut flash, transform) in &mut flashes {
        if flash.is_added() {
            commands.spawn((
                Mesh3d(visual_assets.cross_plane_sphere.clone()),
                MeshMaterial3d(visual_assets.hit_flash_material.clone()),
                Transform::from_translation(transform.translation)
                    .with_scale(Vec3::splat(VFX_SCALE)),
                HitFlashVfx {
                    timer: HIT_FLASH_VFX_DURATION,
                },
                OnGameplayScreen,
            ));
        }
        flash.timer -= time.delta_secs();
        if flash.timer <= 0.0 {
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}

/// Fades and despawns the overlay sphere.
pub fn update_hit_flash_vfx(
    time: Res<Time>,
    mut commands: Commands,
    mut flashes: Query<(Entity, &mut HitFlashVfx, &mut Transform)>,
) {
    for (entity, mut flash, mut transform) in &mut flashes {
        flash.timer -= time.delta_secs();
        if flash.timer <= 0.0 {
            commands.entity(entity).try_despawn();
        } else {
            let alpha = flash.timer / HIT_FLASH_VFX_DURATION;
            transform.scale = Vec3::splat(VFX_SCALE * alpha);
        }
    }
}
