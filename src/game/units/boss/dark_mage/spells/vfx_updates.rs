use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::resources::DarkMageAssets;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Updates falling meteor projectiles -- moves them downward and despawns on impact.
pub fn update_meteor_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Transform, &DarkMageMeteorProjectile)>,
) {
    let delta = time.delta_secs();

    for (entity, mut transform, projectile) in &mut projectiles {
        transform.translation.y -= projectile.velocity * delta;

        // Despawn when it reaches the ground
        if transform.translation.y <= projectile.target_pos.y + 5.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Updates lightning bolt visuals -- despawns after lifetime expires.
pub fn update_lightning_bolts(
    time: Res<Time>,
    mut commands: Commands,
    mut bolts: Query<(Entity, &mut DarkMageLightningBolt)>,
) {
    let delta = time.delta_secs();

    for (entity, mut bolt) in &mut bolts {
        bolt.lifetime -= delta;
        if bolt.lifetime <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Updates temporary visual effects (plague puffs, etc.) -- despawns after lifetime expires.
pub fn update_visual_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut effects: Query<(Entity, &mut DarkMageVisualEffect)>,
) {
    let delta = time.delta_secs();

    for (entity, mut effect) in &mut effects {
        effect.lifetime -= delta;
        if effect.lifetime <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns floating cloud puffs periodically for active plague clouds.
pub fn update_plague_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    assets: Res<DarkMageAssets>,
    spell_assets: Res<SpellVisualAssets>,
    mut clouds: Query<(&Transform, &mut DarkMagePlagueCloud)>,
) {
    let delta = time.delta_secs();

    for (transform, mut cloud) in &mut clouds {
        cloud.particle_timer -= delta;
        if cloud.particle_timer <= 0.0 {
            cloud.particle_timer += 0.4;

            let center = transform.translation;
            let angle = game_rng.0.random::<f32>() * std::f32::consts::TAU;
            let offset_r = cloud.radius * 0.6 * game_rng.0.random::<f32>();
            let px = center.x + angle.cos() * offset_r;
            let pz = center.z + angle.sin() * offset_r;
            let py = 15.0 + game_rng.0.random::<f32>() * 30.0;

            commands.spawn((
                Mesh3d(spell_assets.cross_plane_sphere.clone()),
                MeshMaterial3d(assets.plague_zone_material.clone()),
                Transform::from_translation(Vec3::new(px, py, pz))
                    .with_scale(Vec3::splat(30.0 + game_rng.0.random::<f32>() * 25.0)),
                DarkMageVisualEffect {
                    lifetime: 2.0 + game_rng.0.random::<f32>() * 1.5,
                },
                OnGameplayScreen,
            ));
        }
    }
}
