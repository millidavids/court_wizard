use super::super::components::*;
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, clone_sphere_material,
};
use bevy::prelude::*;

/// Spawns smoke trail wisps behind flying fireballs.
pub fn spawn_fireball_smoke_trail(
    mut commands: Commands,
    fireballs: Query<&Transform, With<Fireball>>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < vfx::constants::SMOKE_SPAWN_INTERVAL {
        return;
    }
    *timer -= vfx::constants::SMOKE_SPAWN_INTERVAL;

    let t = time.elapsed_secs();

    for transform in fireballs.iter() {
        vfx::systems::spawn_fire_smoke_wisps(
            &mut commands,
            &visual_assets,
            transform.translation,
            vfx::constants::SMOKE_COUNT_PER_SPAWN,
            t,
            vfx::constants::SMOKE_LIFETIME,
            vfx::constants::SMOKE_SIZE,
            vfx::constants::SMOKE_RISE_SPEED,
            vfx::constants::SMOKE_SPREAD_SPEED,
        );

        vfx::systems::spawn_heat_shimmer(
            &mut commands,
            &visual_assets,
            transform.translation,
            1,
            t,
        );
    }
}

/// Napalm talent: fireballs with napalm leave small burning zones behind them.
pub fn update_napalm_trails(
    mut commands: Commands,
    time: Res<Time>,
    visual_assets: Res<SpellVisualAssets>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    mut fireballs: Query<
        (&Transform, &mut Fireball),
        Without<crate::game::multiplayer::components::GhostSpellProjectile>,
    >,
) {
    for (transform, mut fireball) in &mut fireballs {
        if !fireball.napalm {
            continue;
        }
        fireball.napalm_timer += time.delta_secs();
        if fireball.napalm_timer >= 0.1 {
            fireball.napalm_timer = 0.0;

            let pos = Vec3::new(transform.translation.x, 3.0, transform.translation.z);
            let mut trail_explosion = FireballExplosion::new(
                pos,
                30.0 * fireball.empowerment,
                fireball.damage * 0.2,
                constants::DAMAGE_TYPE,
                fireball.empowerment,
            );
            trail_explosion.duration = 1.0;

            let mat_handle = clone_sphere_material(
                &mut sphere_materials,
                &visual_assets.fireball_explosion_sphere,
            );

            commands.spawn((
                Mesh3d(visual_assets.explosion_sphere.clone()),
                MeshMaterial3d(mat_handle),
                Transform::from_translation(pos)
                    .with_scale(Vec3::splat(30.0 * fireball.empowerment)),
                trail_explosion,
                crate::game::multiplayer::components::NetworkedSpellEffect {
                    kind: crate::networking::snapshot::SpellEffectKind::NapalmTrail,
                },
                OnGameplayScreen,
            ));
        }
    }
}

/// Spawns wall-of-fire-style orange and black smoke puffs over Scorched Earth fire zones.
pub fn spawn_scorched_earth_fire_smoke(
    mut commands: Commands,
    zones: Query<&FireballExplosion, With<ScorchedEarthFire>>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < 0.25 {
        return;
    }
    *timer -= 0.25;

    let t = time.elapsed_secs();

    for explosion in zones.iter() {
        // Don't emit smoke in the last 0.5s (fade-out)
        let remaining = explosion.duration - explosion.time_alive;
        if remaining < 0.5 {
            continue;
        }

        vfx::systems::spawn_fire_orange_smoke(
            &mut commands,
            &visual_assets,
            Vec3::new(explosion.origin.x, 0.0, explosion.origin.z),
            explosion.max_radius,
            9,
            t,
        );
    }
}
