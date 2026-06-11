use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::DarkMageAssets;
use crate::game::components::OnGameplayScreen;
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::boss::utils::indicator_rotation;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, clone_sphere_material,
};

/// Marker component to track that a plague cloud has already broadcast its hazard.
#[derive(Component)]
pub struct PlagueHazardBroadcast;

/// System that broadcasts plague cloud hazards to the flow field when they spawn.
/// Runs once per cloud entity that doesn't yet have the marker.
pub fn broadcast_plague_hazards(
    mut commands: Commands,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    clouds: Query<(Entity, &Transform, &DarkMagePlagueCloud), Without<PlagueHazardBroadcast>>,
) {
    for (entity, transform, cloud) in &clouds {
        let center_xz = Vec2::new(transform.translation.x, transform.translation.z);
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(center_xz, Vec2::splat(cloud.radius * 2.0)),
            obstacle_type: ObstacleType::Hazard(PLAGUE_HAZARD_COST),
            shape: Some(ObstacleShape::circle(center_xz, cloud.radius)),
            rebuild: false,
        });
        commands.entity(entity).insert(PlagueHazardBroadcast);
    }
}

/// Spawns telegraph indicator entities for a spell.
pub(crate) fn spawn_telegraph_indicators(
    commands: &mut Commands,
    assets: &DarkMageAssets,
    materials: &mut Assets<StandardMaterial>,
    spell_type: DarkMageSpellType,
    target_pos: Vec3,
    direction: Option<Vec3>,
) -> DarkMageIndicators {
    let fill_material = materials.add(StandardMaterial {
        base_color: INDICATOR_BASE_COLOR,
        emissive: bevy::color::LinearRgba::new(0.0, 0.0, 0.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        ..default()
    });

    let mut entities = Vec::new();

    match spell_type {
        DarkMageSpellType::DarkMeteor => {
            // Circle indicator
            let entity = commands
                .spawn((
                    Mesh3d(assets.circle_mesh.clone()),
                    MeshMaterial3d(fill_material.clone()),
                    Transform::from_translation(target_pos.with_y(INDICATOR_Y))
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                        .with_scale(Vec3::splat(METEOR_RADIUS)),
                    DarkMageIndicator,
                    OnGameplayScreen,
                ))
                .id();
            entities.push(entity);
        }

        DarkMageSpellType::ShadowLightning => {
            if let Some(dir) = direction {
                // Rectangle corridor indicator
                let rotation = indicator_rotation(dir);
                let entity = commands
                    .spawn((
                        Mesh3d(assets.rect_mesh.clone()),
                        MeshMaterial3d(fill_material.clone()),
                        Transform::from_translation(target_pos.with_y(INDICATOR_Y))
                            .with_rotation(rotation)
                            .with_scale(Vec3::new(
                                LIGHTNING_CORRIDOR_WIDTH,
                                LIGHTNING_CORRIDOR_LENGTH,
                                1.0,
                            )),
                        DarkMageIndicator,
                        OnGameplayScreen,
                    ))
                    .id();
                entities.push(entity);
            }
        }

        DarkMageSpellType::PlagueCloud => {
            // Circle indicator (like meteor but different radius)
            let entity = commands
                .spawn((
                    Mesh3d(assets.circle_mesh.clone()),
                    MeshMaterial3d(fill_material.clone()),
                    Transform::from_translation(target_pos.with_y(INDICATOR_Y))
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                        .with_scale(Vec3::splat(PLAGUE_RADIUS)),
                    DarkMageIndicator,
                    OnGameplayScreen,
                ))
                .id();
            entities.push(entity);
        }
    }

    DarkMageIndicators {
        entities,
        fill_material,
    }
}

/// Spawns the meteor explosion effect entity. Uses the same Fresnel-shader
/// sphere + impact VFX as the wizard's fireball/meteor_fall, plus procedural
/// fire-orange smoke for the lingering flame look.
pub(crate) fn spawn_meteor_explosion(
    commands: &mut Commands,
    spell_assets: &SpellVisualAssets,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
    target_pos: Vec3,
) {
    let pos = Vec3::new(target_pos.x, 0.0, target_pos.z);
    // Deterministic per-position pseudo-time for VFX seeding (matches meteor_fall).
    let t = pos.x * 0.01 + pos.z * 0.01;

    // Fresnel sphere explosion — cloned per-entity so its opacity fade is independent.
    let mat_handle =
        clone_sphere_material(sphere_materials, &spell_assets.fireball_explosion_sphere);
    commands.spawn((
        Mesh3d(spell_assets.explosion_sphere.clone()),
        MeshMaterial3d(mat_handle),
        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z)).with_scale(Vec3::splat(0.1)),
        DarkMageMeteorExplosion {
            time_alive: 0.0,
            radius: METEOR_RADIUS,
            damage_applied: false,
            damage: METEOR_DAMAGE,
        },
        OnGameplayScreen,
    ));

    // Impact VFX — sparks, smoke, heat shimmer, dark smoke (the "fireball effect").
    vfx::systems::spawn_fire_sparks(commands, spell_assets, pos, vfx::constants::SPARK_COUNT, t);
    vfx::systems::spawn_explosion_smoke(commands, spell_assets, pos, t);
    vfx::systems::spawn_heat_shimmer(
        commands,
        spell_assets,
        pos,
        vfx::constants::EXPLOSION_SHIMMER_COUNT,
        t,
    );
    vfx::systems::spawn_explosion_dark_smoke(commands, spell_assets, pos, t);

    // Procedural fire-orange smoke around the impact (the "new particle effects"
    // for the meteor's fire visual).
    vfx::systems::spawn_fire_orange_smoke(
        commands,
        spell_assets,
        pos,
        METEOR_RADIUS,
        METEOR_FIRE_PARTICLE_COUNT,
        t,
    );

    // Falling meteor projectile from the sky.
    commands.spawn((
        Mesh3d(spell_assets.cross_plane_sphere.clone()),
        MeshMaterial3d(spell_assets.meteor_projectile.clone()),
        Transform::from_translation(Vec3::new(target_pos.x, METEOR_FALL_HEIGHT, target_pos.z))
            .with_scale(Vec3::splat(METEOR_PROJECTILE_RADIUS)),
        DarkMageMeteorProjectile {
            target_pos,
            velocity: METEOR_FALL_HEIGHT / 0.3, // Reaches ground in ~0.3s
        },
        OnGameplayScreen,
    ));
}

/// Spawns the lightning strike effect entity.
pub(crate) fn spawn_lightning_strike(
    commands: &mut Commands,
    assets: &DarkMageAssets,
    spell_assets: &SpellVisualAssets,
    target_pos: Vec3,
    direction: Vec3,
) {
    let rotation = indicator_rotation(direction);

    // Ground corridor damage zone
    commands.spawn((
        Mesh3d(assets.rect_mesh.clone()),
        MeshMaterial3d(assets.lightning_strike_material.clone()),
        Transform::from_translation(target_pos.with_y(INDICATOR_Y + 1.0))
            .with_rotation(rotation)
            .with_scale(Vec3::new(
                LIGHTNING_CORRIDOR_WIDTH,
                LIGHTNING_CORRIDOR_LENGTH,
                1.0,
            )),
        DarkMageLightningStrike {
            lifetime: LIGHTNING_STRIKE_DURATION,
            half_width: LIGHTNING_CORRIDOR_WIDTH / 2.0,
            half_length: LIGHTNING_CORRIDOR_LENGTH / 2.0,
            direction,
            damage_applied: false,
            damage: LIGHTNING_DAMAGE,
        },
        OnGameplayScreen,
    ));

    // Vertical lightning bolt striking down from sky along the corridor
    // Spawn several bolts along the corridor length for visual impact
    let bolt_count = 3;
    let step = LIGHTNING_CORRIDOR_LENGTH / (bolt_count as f32 + 1.0);
    let corridor_start = target_pos - direction * (LIGHTNING_CORRIDOR_LENGTH / 2.0);

    for i in 1..=bolt_count {
        let pos = corridor_start + direction * (step * i as f32);
        commands.spawn((
            Mesh3d(spell_assets.cross_plane_cylinder.clone()),
            MeshMaterial3d(assets.lightning_strike_material.clone()),
            Transform::from_translation(Vec3::new(pos.x, LIGHTNING_BOLT_HEIGHT / 2.0, pos.z))
                .with_scale(Vec3::new(8.0, LIGHTNING_BOLT_HEIGHT, 8.0)),
            DarkMageLightningBolt {
                lifetime: LIGHTNING_STRIKE_DURATION,
            },
            OnGameplayScreen,
        ));
    }
}

/// Spawns the persistent plague cloud entity and broadcasts hazard to flow field.
pub(crate) fn spawn_plague_cloud(
    rng: &mut impl Rng,
    commands: &mut Commands,
    assets: &DarkMageAssets,
    spell_assets: &SpellVisualAssets,
    target_pos: Vec3,
) {
    // Ground zone circle
    commands.spawn((
        Mesh3d(assets.circle_mesh.clone()),
        MeshMaterial3d(assets.plague_zone_material.clone()),
        Transform::from_translation(target_pos.with_y(INDICATOR_Y + 0.5))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(PLAGUE_RADIUS)),
        DarkMagePlagueCloud {
            lifetime: PLAGUE_DURATION,
            tick_timer: 0.0,
            particle_timer: 0.0,
            radius: PLAGUE_RADIUS,
            damage: PLAGUE_DAMAGE_PER_TICK,
        },
        OnGameplayScreen,
    ));

    // Spawn initial cloud puffs (cross-plane spheres floating above the zone)
    for i in 0..5 {
        let angle = (i as f32 / 5.0) * std::f32::consts::TAU;
        let offset_r = PLAGUE_RADIUS * 0.5 * rng.random::<f32>();
        let px = target_pos.x + angle.cos() * offset_r;
        let pz = target_pos.z + angle.sin() * offset_r;
        let py = 20.0 + rng.random::<f32>() * 40.0;

        commands.spawn((
            Mesh3d(spell_assets.cross_plane_sphere.clone()),
            MeshMaterial3d(assets.plague_zone_material.clone()),
            Transform::from_translation(Vec3::new(px, py, pz))
                .with_scale(Vec3::splat(40.0 + rng.random::<f32>() * 30.0)),
            DarkMageVisualEffect {
                lifetime: PLAGUE_DURATION,
            },
            OnGameplayScreen,
        ));
    }
}
