use super::super::components::*;
use super::super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::wizard::spells::arcane_crystal::components::CrystalSpawn;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, clone_sphere_material,
};
use bevy::prelude::*;

/// Spawns a fireball explosion with talent effects at the given position.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_explosion_with_talents(
    rng: &mut dyn rand::RngCore,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
    position: Vec3,
    fireball: &Fireball,
    _time_secs: f32,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    crystal_spawn: Option<&CrystalSpawn>,
) {
    let explosion_duration = if fireball.explosion_duration > 0.0 {
        fireball.explosion_duration
    } else {
        constants::EXPLOSION_DURATION
    };

    let mut explosion = FireballExplosion::new(
        position,
        fireball.explosion_radius,
        fireball.damage,
        constants::DAMAGE_TYPE,
        fireball.empowerment,
    );
    explosion.duration = explosion_duration;
    explosion.chain_ignition = fireball.chain_ignition;

    // Per-entity material clone so each explosion can fade independently
    let mat_handle = clone_sphere_material(sphere_materials, &assets.fireball_explosion_sphere);

    // Pre-generate all sub-explosion bubbles with distance-based sizes
    use rand::Rng;
    let max_r = fireball.explosion_radius;
    let pending: Vec<PendingBubble> = (0..constants::EXPLOSION_BUBBLE_COUNT)
        .map(|_| {
            let direction = Vec3::new(
                rng.random_range(-1.0..1.0_f32),
                rng.random_range(0.0..1.0_f32),
                rng.random_range(-1.0..1.0_f32),
            )
            .normalize_or(Vec3::Y);
            let offset_frac = rng.random_range(
                constants::BUBBLE_OFFSET_FRACTION_MIN..constants::BUBBLE_OFFSET_FRACTION_MAX,
            );
            let distance = max_r * offset_frac;
            // Size so bubble never reaches more than 10% past main explosion edge
            let radius = max_r * (constants::BUBBLE_OVERSHOOT - offset_frac);
            PendingBubble {
                direction,
                distance,
                radius,
            }
        })
        .collect();

    let entity = commands
        .spawn((
            Mesh3d(assets.explosion_sphere.clone()),
            MeshMaterial3d(mat_handle),
            Transform::from_translation(position).with_scale(Vec3::splat(0.1)),
            ExplosionBubbleSpawner { pending },
            explosion,
            NetworkedSpellEffect {
                kind: crate::networking::snapshot::SpellEffectKind::FireballExplosion,
            },
            OnGameplayScreen,
        ))
        .id();

    if let Some(cs) = crystal_spawn {
        commands.entity(entity).insert(CrystalSpawn {
            origin: cs.origin,
            max_range: cs.max_range,
            lifetime: None,
        });
    }

    // Sparks, smoke, and heat shimmer are spawned by update_explosions on first frame

    // Impact sound effect
    audio::play_impact_sfx(commands, &sfx.fireball_impact, position, game_config, sfx);

    // Cluster Bomb: spawn 3 mini-fireballs in random directions
    if fireball.cluster_bomb {
        spawn_cluster_bombs(rng, commands, assets, position, fireball);
    }

    // Scorched Earth: spawn persistent burning ground circle
    if fireball.scorched_earth {
        let scorched_pos = Vec3::new(position.x, 1.5, position.z);
        let mut scorched = FireballExplosion::new(
            scorched_pos,
            fireball.explosion_radius * 0.8,
            fireball.damage * 0.3,
            constants::DAMAGE_TYPE,
            fireball.empowerment,
        );
        scorched.duration = 5.0;
        scorched.skip_growth = true;
        scorched.chain_ignition = fireball.chain_ignition;

        commands.spawn((
            Transform::from_translation(scorched_pos),
            Visibility::default(),
            scorched,
            ScorchedEarthFire,
            crate::game::multiplayer::components::NetworkedSpellEffect {
                kind: crate::networking::snapshot::SpellEffectKind::ScorchedEarthFire,
            },
            OnGameplayScreen,
        ));
    }
}

/// Spawns 3 mini-fireballs for the Cluster Bomb talent.
fn spawn_cluster_bombs(
    rng: &mut dyn rand::RngCore,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    parent_fireball: &Fireball,
) {
    use rand::Rng;

    for _ in 0..3 {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let distance = rng.random_range(100.0..300.0);
        let flight_time = 0.4;
        let horizontal_speed = distance / flight_time;
        let vy = -(origin.y.max(5.0)) / flight_time;
        let velocity = Vec3::new(
            angle.cos() * horizontal_speed,
            vy,
            angle.sin() * horizontal_speed,
        );

        let mut mini = Fireball::new(
            velocity,
            parent_fireball.damage * 0.5,
            constants::DAMAGE_TYPE,
            parent_fireball.explosion_radius * 0.5,
            parent_fireball.radius * 0.5,
            parent_fireball.empowerment,
        );
        mini.scorched_earth = parent_fireball.scorched_earth;
        mini.chain_ignition = parent_fireball.chain_ignition;

        let visual_radius = 15.0 * parent_fireball.empowerment;

        let entity = commands
            .spawn((
                Mesh3d(assets.cross_plane_sphere.clone()),
                MeshMaterial3d(assets.fireball_projectile.clone()),
                Transform::from_translation(origin).with_scale(Vec3::splat(visual_radius)),
                mini,
                OnGameplayScreen,
            ))
            .id();

        vfx::systems::spawn_fire_glow(
            commands,
            assets,
            entity,
            origin,
            visual_radius,
            OnGameplayScreen,
        );
    }
}
