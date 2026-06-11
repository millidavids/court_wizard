use super::super::super::vfx;
use super::super::components::BanishmentVfx;
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// Spawns a shrinking lensing sphere VFX and burst of sparks at the given position.
pub(super) fn spawn_banishment_vfx(
    commands: &mut Commands,
    position: Vec3,
    visual_assets: &SpellVisualAssets,
    pending: &mut crate::game::multiplayer::spell_sync::PendingCastEvents,
) {
    let start_radius = constants::VFX_START_RADIUS;
    commands.spawn((
        BanishmentVfx {
            time_alive: 0.0,
            lifetime: constants::VFX_LIFETIME,
            start_radius,
        },
        Mesh3d(visual_assets.cross_plane_sphere.clone()),
        MeshMaterial3d(visual_assets.banishment_lens.clone()),
        Transform::from_translation(position).with_scale(Vec3::splat(start_radius)),
        OnGameplayScreen,
    ));
    vfx::systems::emit_banishment_lens_event(
        pending,
        position,
        start_radius,
        constants::VFX_LIFETIME,
    );

    // Spawn exploding spark particles (reuses FireSpark component + update system)
    vfx::systems::spawn_sparks_with_material_synced(
        commands,
        visual_assets,
        pending,
        crate::networking::snapshot::SparkMaterial::Banishment,
        visual_assets.banishment_spark.clone(),
        position,
        constants::SPARK_COUNT,
        0.0,
    );
}

/// Animates banishment lensing VFX: shrinks from start radius to zero, then despawns.
pub fn update_banishment_vfx(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BanishmentVfx, &mut Transform)>,
) {
    let delta = time.delta_secs();
    for (entity, mut vfx, mut transform) in &mut query {
        vfx.time_alive += delta;
        if vfx.time_alive >= vfx.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = vfx.time_alive / vfx.lifetime;
        // Quadratic ease-in for accelerating collapse
        let remaining = 1.0 - progress * progress;
        let radius = vfx.start_radius * remaining;
        transform.scale = Vec3::splat(radius.max(0.01));
    }
}
