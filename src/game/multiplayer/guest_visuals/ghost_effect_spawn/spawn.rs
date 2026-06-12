use bevy::prelude::*;

use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets,
};
use crate::networking::snapshot::{SpellEffectKind, SpellEffectSnapshot};

use super::boulders::{spawn_boulder_obstacle, spawn_boulder_projectile};
use super::explosions::{
    spawn_dispel_impact, spawn_fireball_explosion, spawn_flame_ground_fire, spawn_ice_explosion,
    spawn_meteor_explosion, spawn_napalm_trail, spawn_scorched_earth_fire, spawn_squall_storm,
};
use super::objects::{spawn_arcane_crystal, spawn_black_hole, spawn_lightning_rod};
use super::walls::{spawn_wall_of_fire, spawn_wall_of_stone};
use super::zones::{
    spawn_entangle_ground, spawn_fog_cloud_zone, spawn_grease_fire, spawn_grease_zone,
    spawn_healing_plume_zone, spawn_meteor_ground_fire, spawn_plague_wind_cloud,
    spawn_spike_growth_zone,
};

/// Spawns a persistent spell effect entity with real components.
///
/// Returns the spawned entity, or `None` if the kind is unknown.
/// Zone fade systems modify materials directly, so zones get unique material clones.
/// Some effects (black hole, lightning rod) allocate meshes at spawn since they need
/// specific sizes; these are rare entities so the cost is negligible.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_spell_effect(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
    boulder_assets: &crate::game::terrain::boulder::resources::BoulderAssets,
) -> Option<Entity> {
    let kind = SpellEffectKind::try_from(effect.kind).ok()?;
    let pos = Vec3::new(effect.x, effect.y, effect.z);

    // Rotation for flat circles (face up on the ground plane)
    let flat_rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);

    match kind {
        // ── Zones (flat circles, real components, unique materials for fading) ──
        SpellEffectKind::SpikeGrowthZone => spawn_spike_growth_zone(commands, effect, pos),
        SpellEffectKind::HealingPlumeZone => {
            spawn_healing_plume_zone(commands, effect, pos, assets, materials, flat_rotation)
        }
        SpellEffectKind::EntangleGround => {
            spawn_entangle_ground(commands, effect, pos, assets, materials)
        }
        SpellEffectKind::FogCloudZone => {
            spawn_fog_cloud_zone(commands, effect, pos, assets, materials, flat_rotation)
        }
        SpellEffectKind::GreaseZone => {
            spawn_grease_zone(commands, effect, pos, assets, materials, flat_rotation)
        }
        SpellEffectKind::GreaseFire => {
            spawn_grease_fire(commands, effect, pos, assets, materials, flat_rotation)
        }
        SpellEffectKind::PlagueWindCloud => spawn_plague_wind_cloud(commands, effect, pos),
        SpellEffectKind::MeteorGroundFire => spawn_meteor_ground_fire(commands, effect, pos),

        // ── Objects (3D meshes, shared materials) ──
        SpellEffectKind::BlackHole => spawn_black_hole(commands, effect, pos, assets),
        SpellEffectKind::ArcaneCrystal => spawn_arcane_crystal(commands, effect, pos, assets),
        SpellEffectKind::LightningRod => spawn_lightning_rod(commands, effect, pos, assets),

        // ── Walls ──
        SpellEffectKind::WallOfStone => spawn_wall_of_stone(commands, effect, pos, assets),
        SpellEffectKind::WallOfFire => spawn_wall_of_fire(commands, effect, pos, assets, materials),

        // ── Explosions (sphere meshes, scale-driven animation) ──
        SpellEffectKind::FireballExplosion => {
            spawn_fireball_explosion(commands, effect, pos, assets, sphere_materials)
        }
        SpellEffectKind::MeteorExplosion => {
            spawn_meteor_explosion(commands, effect, pos, assets, sphere_materials)
        }
        SpellEffectKind::IceExplosion => {
            spawn_ice_explosion(commands, effect, pos, assets, sphere_materials)
        }
        SpellEffectKind::DispelImpact => spawn_dispel_impact(commands, effect, pos, assets),
        SpellEffectKind::SquallStorm => spawn_squall_storm(commands, effect, pos),
        SpellEffectKind::ScorchedEarthFire => {
            spawn_scorched_earth_fire(commands, effect, pos, assets, sphere_materials)
        }
        SpellEffectKind::NapalmTrail => {
            spawn_napalm_trail(commands, effect, pos, assets, sphere_materials)
        }
        SpellEffectKind::FlameGroundFire => spawn_flame_ground_fire(commands, effect, pos),

        // ── Boulders ──
        SpellEffectKind::BoulderProjectileEffect => {
            spawn_boulder_projectile(commands, effect, pos, boulder_assets)
        }
        SpellEffectKind::BoulderObstacle => {
            spawn_boulder_obstacle(commands, effect, pos, boulder_assets)
        }
    }
}
