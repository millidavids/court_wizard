//! The visible half of hit feedback: a brief element-colored overlay that pops
//! on a unit when something connects with it.
//!
//! Insert [`HitFlash`] on the target; [`update_hit_flashes`] spawns the overlay
//! [`HitFlashVfx`] on the next frame and [`update_hit_flash_vfx`] fades it out.
//! Spells go through `PendingSpellHit` rather than inserting `HitFlash`
//! directly — see `spell_hits.rs`.

use bevy::prelude::*;

use super::super::damage::DamageType;
use super::super::wizard::spells::visual_assets::SpellVisualAssets;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;

/// Marker for the unit-side flash. The `timer` field gates how long the
/// `HitFlash` component stays attached to the unit; pick whatever value the
/// caller likes (it doesn't affect VFX duration).
#[derive(Component)]
pub struct HitFlash {
    pub timer: f32,
    /// Element whose color the overlay is tinted with.
    pub damage_type: DamageType,
    /// Overlay size, resolved from the target's `Hitbox` when this was
    /// requested. Captured here rather than looked up at spawn time because
    /// `convert_dead_to_corpses` strips `Hitbox` on death — reading it later
    /// would give every killing blow the infantry-sized fallback.
    pub base_scale: f32,
}

/// The visible overlay entity that gets spawned alongside the hit unit. Always
/// uses `HIT_FLASH_VFX_DURATION` for its fade.
#[derive(Component)]
pub struct HitFlashVfx {
    pub timer: f32,
    /// World-space size at full strength. Derived from the target's hitbox so
    /// a boss doesn't get an infantry-sized dot.
    pub base_scale: f32,
}

/// Lifetime of the overlay in seconds.
pub const HIT_FLASH_VFX_DURATION: f32 = 0.08;

/// Fallback overlay scale for targets with no `Hitbox` (roughly infantry-sized:
/// `UNIT_RADIUS` is `8 * UNIT_SCALE` = 32).
pub(super) const VFX_SCALE: f32 = 30.0;

/// Spawns the overlay on the frame `HitFlash` is added, then ticks the flag's
/// timer and removes it once expired.
///
/// Note the spawn is gated on `is_added()`. Re-inserting a component that is
/// already present marks it *changed*, not *added*, so a second `HitFlash`
/// landing while one is live produces no new overlay. `SpellHitCooldown`
/// (0.22s) is longer than this VFX (0.08s), which keeps that from happening for
/// spell hits.
pub fn update_hit_flashes(
    time: Res<Time>,
    mut commands: Commands,
    mut flashes: Query<(Entity, &mut HitFlash, &Transform)>,
    visual_assets: Res<SpellVisualAssets>,
    config: Res<GameConfig>,
) {
    for (entity, mut flash, transform) in &mut flashes {
        if flash.is_added() {
            let palette = if config.reduce_flashes {
                &visual_assets.hit_flash_materials_dim
            } else {
                &visual_assets.hit_flash_materials
            };
            let material = palette[flash.damage_type.to_u8() as usize].clone();
            let base_scale = flash.base_scale;

            commands.spawn((
                Mesh3d(visual_assets.cross_plane_sphere.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(transform.translation)
                    .with_scale(Vec3::splat(base_scale)),
                HitFlashVfx {
                    timer: HIT_FLASH_VFX_DURATION,
                    base_scale,
                },
                OnGameplayScreen,
            ));
        }
        flash.timer -= time.delta_secs();
        if flash.timer <= 0.0 {
            commands.entity(entity).try_remove::<HitFlash>();
        }
    }
}

/// Shrinks and despawns the overlay sphere.
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
            let remaining = flash.timer / HIT_FLASH_VFX_DURATION;
            transform.scale = Vec3::splat(flash.base_scale * remaining);
        }
    }
}
