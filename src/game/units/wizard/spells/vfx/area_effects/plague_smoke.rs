use bevy::prelude::*;

use super::super::components::PlagueSmoke;
use super::super::constants;
use crate::game::components::Billboard;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

// ── Plague smoke (poison cloud particles) ─────────────────────────────

/// Spawns plague smoke puffs scattered within a cloud volume.
/// Each puff is a billboard that drifts upward with gentle swirling.
/// Tunable parameters for smoke puff spawning.
pub struct SmokePuffParams {
    pub rise_speed: f32,
    pub swirl_speed: f32,
    pub size: f32,
    pub lifetime: f32,
    /// Height base multiplier (fraction of rise_speed for minimum height).
    pub height_base: f32,
    /// Height range multiplier (fraction of rise_speed added by randomness).
    pub height_range: f32,
}

/// Plague wind smoke parameters (standard cloud).
pub const PLAGUE_SMOKE_PARAMS: SmokePuffParams = SmokePuffParams {
    rise_speed: constants::PLAGUE_SMOKE_RISE_SPEED,
    swirl_speed: constants::PLAGUE_SMOKE_SWIRL_SPEED,
    size: constants::PLAGUE_SMOKE_SIZE,
    lifetime: constants::PLAGUE_SMOKE_LIFETIME,
    height_base: 0.3,
    height_range: 0.5,
};

/// Fog cloud smoke parameters (denser, ground-hugging).
pub const FOG_SMOKE_PARAMS: SmokePuffParams = SmokePuffParams {
    rise_speed: constants::FOG_SMOKE_RISE_SPEED,
    swirl_speed: constants::FOG_SMOKE_SWIRL_SPEED,
    size: constants::FOG_SMOKE_SIZE,
    lifetime: constants::FOG_SMOKE_LIFETIME,
    height_base: 0.2,
    height_range: 0.3,
};

/// Spawns plague smoke puffs (green).
pub fn spawn_plague_smoke_puffs(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    center: Vec3,
    cloud_radius: f32,
    count: usize,
    time_secs: f32,
) {
    spawn_smoke_puffs(
        commands,
        assets,
        &assets.plague_smoke,
        &PLAGUE_SMOKE_PARAMS,
        center,
        cloud_radius,
        count,
        time_secs,
    );
}

/// Spawns fog smoke puffs (gray, denser and ground-hugging).
pub fn spawn_fog_smoke_puffs(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    center: Vec3,
    cloud_radius: f32,
    count: usize,
    time_secs: f32,
) {
    spawn_smoke_puffs(
        commands,
        assets,
        &assets.fog_smoke,
        &FOG_SMOKE_PARAMS,
        center,
        cloud_radius,
        count,
        time_secs,
    );
}

/// Spawns smoke puffs with configurable material and parameters.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_smoke_puffs(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    material: &Handle<StandardMaterial>,
    params: &SmokePuffParams,
    center: Vec3,
    cloud_radius: f32,
    count: usize,
    time_secs: f32,
) {
    for i in 0..count {
        let seed = i as f32 * 1.618_034 + time_secs * 7.1;
        let angle = seed * 2.39 + (seed * 13.7).sin() * 1.5;

        // Scatter within cloud radius (weighted toward edges for more volume)
        let r_frac = 0.2 + 0.8 * ((seed * 23.1).sin() * 0.5 + 0.5);
        let r = cloud_radius * r_frac * 0.7;
        let x = center.x + angle.cos() * r;
        let z = center.z + angle.sin() * r;

        // Random height within cloud volume
        let height_frac = (seed * 31.3).cos() * 0.5 + 0.5;
        let y = params.rise_speed * params.height_base
            + height_frac * params.rise_speed * params.height_range;

        // Gentle upward drift with swirl
        let rise_variation = 0.7 + 0.3 * ((seed * 17.3).cos() * 0.5 + 0.5);
        let swirl_x = angle.sin() * params.swirl_speed;
        let swirl_z = -angle.cos() * params.swirl_speed;
        let velocity = Vec3::new(swirl_x, params.rise_speed * rise_variation, swirl_z);

        let size_variation = 0.6 + 0.4 * ((seed * 41.7).sin() * 0.5 + 0.5);
        let base_size = params.size * size_variation;

        let lifetime_variation = 0.8 + 0.4 * ((seed * 53.3).cos() * 0.5 + 0.5);

        commands.spawn((
            PlagueSmoke {
                velocity,
                time_alive: 0.0,
                lifetime: params.lifetime * lifetime_variation,
                base_size,
                phase: seed,
                spawn_y: 0.0,
            },
            Mesh3d(assets.particle_quad.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(Vec3::new(x, y, z))
                .with_scale(Vec3::splat(base_size * 0.3)),
            Billboard,
            OnGameplayScreen,
        ));
    }
}

/// Updates plague smoke puffs: drift, sway, grow, then fade.
pub fn update_plague_smoke(
    mut commands: Commands,
    mut smoke_query: Query<(Entity, &mut PlagueSmoke, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for (entity, mut smoke, mut transform) in smoke_query.iter_mut() {
        smoke.time_alive += dt;

        if smoke.time_alive >= smoke.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = smoke.time_alive / smoke.lifetime;

        // Drift upward + swirl
        transform.translation += smoke.velocity * dt;

        // Height-based effects for fire puffs (spawn_y > 0 means this is a fire puff)
        let is_fire = smoke.spawn_y > 0.0;
        let height_factor = if is_fire {
            let height_risen = (transform.translation.y - smoke.spawn_y).max(0.0);
            (height_risen / constants::FIRE_HEIGHT_SCALE_RANGE).clamp(0.0, 2.0)
        } else {
            0.0
        };

        // Lateral sway — fire puffs widen as they rise
        let sway_mult = if is_fire {
            1.0 + height_factor * constants::FIRE_HEIGHT_SWAY_MULT
        } else {
            1.0
        };
        let sway_amp = constants::PLAGUE_SMOKE_SWAY_AMPLITUDE * sway_mult;
        let sway_x = (t * constants::PLAGUE_SMOKE_SWAY_FREQUENCY * std::f32::consts::TAU
            + smoke.phase)
            .sin()
            * sway_amp
            * dt;
        transform.translation.x += sway_x;

        // Fire puffs also get Z-axis sway for 3D billowing
        if is_fire {
            let sway_z = (t * constants::PLAGUE_SMOKE_SWAY_FREQUENCY * 0.7 * std::f32::consts::TAU
                + smoke.phase
                + 1.5)
                .cos()
                * sway_amp
                * 0.6
                * dt;
            transform.translation.z += sway_z;
        }

        // Scale: grow quickly in first 30%, hold, then shrink in last 30%
        let base_scale = if progress < 0.3 {
            let grow = progress / 0.3;
            smoke.base_size * (0.3 + 0.7 * grow)
        } else if progress > 0.7 {
            let shrink = 1.0 - (progress - 0.7) / 0.3;
            smoke.base_size * shrink
        } else {
            smoke.base_size
        };

        // Fire puffs grow larger as they rise
        let height_scale = if is_fire {
            1.0 + height_factor * constants::FIRE_HEIGHT_SIZE_MULT
        } else {
            1.0
        };
        transform.scale = Vec3::splat(base_scale * height_scale);
    }
}
