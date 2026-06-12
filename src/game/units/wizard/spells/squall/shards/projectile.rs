//! Ice projectile spawning, physics, collision, and frozen-ground patch helper.

use bevy::prelude::*;
use rand::Rng;

use super::super::components::{FrozenGround, IceExplosion, IceProjectile, SquallStorm};
use super::super::constants::*;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, clone_sphere_material,
};
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::networking::snapshot::SpellEffectKind;

/// Spawns ice projectiles from active squall storms.
pub(crate) fn spawn_ice_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    // Host-only — guest's ghost SquallStorm must NOT independently spawn
    // ice / apply CC; the host's authoritative storm drives gameplay and
    // CRDT carries the result.
    mut storms: Query<
        &mut SquallStorm,
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
) {
    let rng = &mut game_rng.0;

    for mut storm in storms.iter_mut() {
        storm.update_timers(time.delta_secs());

        // Apply spawn rate talent modifier
        let spawn_interval = ICE_SPAWN_INTERVAL * storm.talent_params.spawn_rate_mult;

        // Check if it's time to spawn another projectile
        if storm.time_since_spawn >= spawn_interval {
            storm.reset_spawn_timer();

            // Random position within storm circle (on XZ plane)
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let distance = rng.random_range(0.0..storm.radius);
            let offset = Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);

            let spawn_pos = Vec3::new(
                storm.position.x + offset.x,
                ICE_SPAWN_HEIGHT,
                storm.position.z + offset.z,
            );

            // Determine if this is a hailstone (Tier 2 talent)
            let is_hailstone =
                storm.talent_params.hailstones && rng.random_range(0.0..1.0) < HAILSTONE_CHANCE;

            // Calculate damage with talent modifiers
            let base_damage = FROST_DAMAGE * storm.empowerment * storm.talent_params.damage_mult;
            let damage = if is_hailstone {
                base_damage * HAILSTONE_DAMAGE_MULT
            } else {
                base_damage
            };
            let explosion_radius = EXPLOSION_RADIUS * storm.empowerment;
            let mesh_scale = if is_hailstone {
                ICE_PROJECTILE_MESH_RADIUS * HAILSTONE_MESH_SCALE
            } else {
                ICE_PROJECTILE_MESH_RADIUS
            };

            commands.spawn((
                IceProjectile::new(
                    Vec3::new(0.0, ICE_INITIAL_VELOCITY, 0.0),
                    damage,
                    explosion_radius,
                    storm.empowerment,
                    is_hailstone,
                    storm.talent_params.ice_age,
                ),
                Mesh3d(visual_assets.cross_plane_sphere.clone()),
                MeshMaterial3d(visual_assets.ice_projectile.clone()),
                Transform::from_translation(spawn_pos).with_scale(Vec3::splat(mesh_scale)),
                OnGameplayScreen,
            ));
        }
    }
}

/// Updates ice projectile physics - applies gravity and moves projectiles.
pub(crate) fn update_ice_projectiles(
    time: Res<Time>,
    mut projectiles: Query<(&mut Transform, &mut IceProjectile)>,
) {
    let delta = time.delta_secs();

    for (mut transform, mut projectile) in projectiles.iter_mut() {
        // Apply gravity
        projectile.velocity.y += ICE_GRAVITY * delta;

        // Move projectile
        transform.translation += projectile.velocity * delta;
    }
}

/// Checks for ice projectile collisions with ground or walls, spawns explosions.
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_ice_projectile_collisions(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    projectiles: Query<(Entity, &Transform, &IceProjectile)>,
    walls: Query<&WallOfStone>,
    rocks: Query<&crate::game::terrain::boulder::components::Boulder>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    for (entity, transform, projectile) in projectiles.iter() {
        let projectile_pos = transform.translation;

        // Check wall collision
        let mut hit_wall = false;
        for wall in &walls {
            if wall.contains_point_xz(projectile_pos) && projectile_pos.y <= wall.height {
                // Hit wall - spawn explosion at wall surface
                let explosion_pos = Vec3::new(projectile_pos.x, wall.height, projectile_pos.z);
                spawn_ice_explosion(
                    &mut commands,
                    &visual_assets,
                    &mut sphere_materials,
                    explosion_pos,
                    projectile.explosion_radius,
                    projectile.damage,
                    projectile.empowerment,
                );
                let sfx_scale = if projectile.is_hailstone { 0.5 } else { 0.3 };
                audio::play_impact_sfx_scaled(
                    &mut commands,
                    &sfx.squall_impact,
                    explosion_pos,
                    &game_config,
                    &sfx,
                    sfx_scale,
                );
                commands.entity(entity).try_despawn();
                hit_wall = true;
                break;
            }
        }
        if hit_wall {
            continue;
        }

        // Check collision with rocks
        let mut hit_rock = false;
        for rock in &rocks {
            if rock.blocks_projectile(projectile_pos) {
                let explosion_pos = Vec3::new(projectile_pos.x, rock.height, projectile_pos.z);
                spawn_ice_explosion(
                    &mut commands,
                    &visual_assets,
                    &mut sphere_materials,
                    explosion_pos,
                    projectile.explosion_radius,
                    projectile.damage,
                    projectile.empowerment,
                );
                let sfx_scale = if projectile.is_hailstone { 0.5 } else { 0.3 };
                audio::play_impact_sfx_scaled(
                    &mut commands,
                    &sfx.squall_impact,
                    explosion_pos,
                    &game_config,
                    &sfx,
                    sfx_scale,
                );
                commands.entity(entity).try_despawn();
                hit_rock = true;
                break;
            }
        }
        if hit_rock {
            continue;
        }

        // Check ground collision (Y <= 0)
        if projectile_pos.y <= 0.0 {
            // Hit ground - spawn explosion at ground level
            let explosion_pos = Vec3::new(projectile_pos.x, 0.0, projectile_pos.z);
            spawn_ice_explosion(
                &mut commands,
                &visual_assets,
                &mut sphere_materials,
                explosion_pos,
                projectile.explosion_radius,
                projectile.damage,
                projectile.empowerment,
            );
            // Ice Age: spawn frozen ground at impact point
            if projectile.ice_age {
                spawn_frozen_ground_patch(
                    &mut commands,
                    &visual_assets,
                    explosion_pos,
                    projectile.empowerment,
                );
            }
            let sfx_scale = if projectile.is_hailstone { 0.5 } else { 0.3 };
            audio::play_impact_sfx_scaled(
                &mut commands,
                &sfx.squall_impact,
                explosion_pos,
                &game_config,
                &sfx,
                sfx_scale,
            );
            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns an ice explosion at the given position.
fn spawn_ice_explosion(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
    position: Vec3,
    max_radius: f32,
    damage: f32,
    empowerment: f32,
) {
    let explosion_pos = Vec3::new(position.x, 1.0, position.z);

    let mat_handle = clone_sphere_material(sphere_materials, &assets.ice_explosion_sphere);

    commands.spawn((
        Mesh3d(assets.explosion_sphere.clone()),
        MeshMaterial3d(mat_handle),
        Transform::from_translation(explosion_pos).with_scale(Vec3::splat(0.1)),
        IceExplosion::new(position, max_radius, damage, empowerment),
        NetworkedSpellEffect {
            kind: SpellEffectKind::IceExplosion,
        },
        OnGameplayScreen,
    ));
}

/// Spawns a frozen ground patch at an impact point (Ice Age talent).
pub(crate) fn spawn_frozen_ground_patch(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    empowerment: f32,
) {
    let patch_pos = Vec3::new(position.x, 0.05, position.z);
    let patch_radius = ICE_AGE_PATCH_RADIUS * empowerment;

    commands.spawn((
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(assets.ice_explosion.clone()),
        Transform::from_translation(patch_pos)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(patch_radius)),
        FrozenGround::new(patch_pos, patch_radius, ICE_AGE_GROUND_DURATION),
        OnGameplayScreen,
    ));
}
