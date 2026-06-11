use bevy::prelude::*;
use rand::Rng;

use super::super::components::{MeteorExplosion, MeteorFallStorm, MeteorProjectile};
use super::super::constants::*;
use super::super::meteor::find_nearest_non_defender_xz;
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::components::Team;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, clone_sphere_material,
};
use crate::networking::snapshot::SpellEffectKind;

/// Talent flags to apply to a meteor projectile at spawn time.
pub(crate) struct MeteorProjectileTalentFlags {
    pub aftershock: bool,
    pub volcanic_eruption: bool,
    pub ground_fire_duration_mult: f32,
    pub ground_fire_damage_mult: f32,
    pub ground_fire_radius_mult: f32,
    pub tracking: bool,
    pub is_extinction: bool,
}

impl Default for MeteorProjectileTalentFlags {
    fn default() -> Self {
        Self {
            aftershock: false,
            volcanic_eruption: false,
            ground_fire_duration_mult: 1.0,
            ground_fire_damage_mult: 1.0,
            ground_fire_radius_mult: 1.0,
            tracking: false,
            is_extinction: false,
        }
    }
}

/// Spawns meteor projectiles periodically from active storms.
///
/// Projectiles spawn at random positions within the storm radius, high above the battlefield.
pub(crate) fn spawn_meteor_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut storms: Query<(Entity, &mut MeteorFallStorm)>,
    enemies: Query<(&Transform, &Team)>,
) {
    let rng = &mut game_rng.0;

    for (storm_entity, mut storm) in storms.iter_mut() {
        storm.update_timers(time.delta_secs());

        // Check if fixed-duration storm has expired (Extinction Event)
        if let Some(duration) = storm.duration
            && storm.time_alive >= duration
        {
            commands.entity(storm_entity).try_despawn();
            continue;
        }

        // Check if it's time to spawn another meteor
        if storm.time_since_spawn >= storm.spawn_interval {
            storm.reset_spawn_timer();

            // Random position within storm circle (on XZ plane)
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let distance = rng.random_range(0.0..storm.radius);
            let offset = Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);

            let spawn_pos = Vec3::new(
                storm.position.x + offset.x,
                METEOR_SPAWN_HEIGHT,
                storm.position.z + offset.z,
            );

            // Spawn projectile with talent-modified values
            let damage = METEOR_DAMAGE * storm.empowerment * storm.damage_mult;
            let explosion_radius =
                EXPLOSION_RADIUS * storm.empowerment * storm.explosion_radius_mult;
            let mesh_radius = METEOR_MESH_RADIUS * storm.mesh_radius_mult;

            let entity = spawn_meteor_projectile_entity(
                &mut commands,
                &visual_assets,
                spawn_pos,
                Vec3::new(0.0, METEOR_INITIAL_VELOCITY, 0.0),
                damage,
                explosion_radius,
                storm.empowerment,
                mesh_radius,
                MeteorProjectileTalentFlags {
                    aftershock: storm.aftershock,
                    volcanic_eruption: storm.volcanic_eruption,
                    ground_fire_duration_mult: storm.ground_fire_duration_mult,
                    ground_fire_damage_mult: storm.ground_fire_damage_mult,
                    ground_fire_radius_mult: storm.ground_fire_radius_mult,
                    tracking: storm.tracking,
                    is_extinction: false,
                },
            );

            // For tracking meteors, bias spawn position toward nearest enemy
            if storm.tracking {
                let storm_center_xz = Vec2::new(storm.position.x, storm.position.z);
                if let Some((enemy_xz, _)) = find_nearest_non_defender_xz(
                    enemies
                        .iter()
                        .map(|(t, team)| (Vec2::new(t.translation.x, t.translation.z), *team)),
                    storm_center_xz,
                    Some(storm.radius),
                ) {
                    // Bias 50% toward nearest enemy
                    let biased_x = spawn_pos.x * 0.5 + enemy_xz.x * 0.5;
                    let biased_z = spawn_pos.z * 0.5 + enemy_xz.y * 0.5;
                    commands.entity(entity).insert(
                        Transform::from_translation(Vec3::new(
                            biased_x,
                            METEOR_SPAWN_HEIGHT,
                            biased_z,
                        ))
                        .with_scale(Vec3::splat(mesh_radius)),
                    );
                }
            }
        }
    }
}

/// Spawns a raw meteor projectile entity with explicit parameters.
///
/// Used by both storm spawning and crystal absorption/auto-cast.
/// Pass `MeteorProjectileTalentFlags::default()` for non-talented projectiles.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_meteor_projectile_entity(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    spawn_pos: Vec3,
    velocity: Vec3,
    damage: f32,
    explosion_radius: f32,
    empowerment: f32,
    mesh_radius: f32,
    talent_flags: MeteorProjectileTalentFlags,
) -> Entity {
    let mut projectile =
        MeteorProjectile::new(velocity, damage, explosion_radius, empowerment, mesh_radius);
    projectile.aftershock = talent_flags.aftershock;
    projectile.volcanic_eruption = talent_flags.volcanic_eruption;
    projectile.ground_fire_duration_mult = talent_flags.ground_fire_duration_mult;
    projectile.ground_fire_damage_mult = talent_flags.ground_fire_damage_mult;
    projectile.ground_fire_radius_mult = talent_flags.ground_fire_radius_mult;
    projectile.tracking = talent_flags.tracking;
    projectile.is_extinction = talent_flags.is_extinction;

    commands
        .spawn((
            projectile,
            Mesh3d(assets.cross_plane_sphere.clone()),
            MeshMaterial3d(assets.meteor_projectile.clone()),
            Transform::from_translation(spawn_pos).with_scale(Vec3::splat(mesh_radius)),
            OnGameplayScreen,
        ))
        .id()
}

/// Spawns a meteor explosion visual entity.
pub(crate) fn spawn_explosion_entity(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
    pos: Vec3,
    radius: f32,
    damage: f32,
    networked: bool,
) {
    let mat_handle = clone_sphere_material(sphere_materials, &assets.fireball_explosion_sphere);

    let mut entity = commands.spawn((
        Mesh3d(assets.explosion_sphere.clone()),
        MeshMaterial3d(mat_handle),
        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z)).with_scale(Vec3::splat(0.1)),
        MeteorExplosion::new(pos, radius, damage),
        OnGameplayScreen,
    ));
    if networked {
        entity.insert(NetworkedSpellEffect {
            kind: SpellEffectKind::MeteorExplosion,
        });
    }
}
