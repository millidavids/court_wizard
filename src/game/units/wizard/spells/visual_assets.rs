//! Shared spell visual assets.
//!
//! Pre-allocates all meshes and materials used by spell effects.
//! Both local spell spawning and ghost/remote rendering use these handles,
//! ensuring a single source of truth for spell visuals.

use std::collections::HashMap;

use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

use crate::config::{GameConfig, WizardType};
use crate::game::units::constants::EXCREMAGE_BROWN;

use super::black_hole::constants::TORUS_MINOR_RADIUS;
use super::telekinesis::constants::{HARVEST_FLASH_COLOR, SHOCKWAVE_COLOR, SHOCKWAVE_TORUS_MINOR};
use super::wall_of_stone::wall_material::WallOfStoneMaterial;
use crate::game::battlefield::components::BattlefieldAssets;
use crate::game::battlefield::constants::{GROUND_NOISE_BASE, TILE_COUNT, TILE_WORLD_SIZE};
use crate::game::constants::{STONE_COLOR_DARK, STONE_COLOR_LIGHT};

/// Default bright yellow center color for fire explosion material.
const FIRE_EXPLOSION_INNER_COLOR: LinearRgba = LinearRgba::new(4.0, 2.5, 0.4, 1.0);
/// Default deep orange-red edge color for fire explosion material.
const FIRE_EXPLOSION_OUTER_COLOR: LinearRgba = LinearRgba::new(2.5, 0.4, 0.0, 1.0);
/// Bright white center color for ice explosion material.
const ICE_EXPLOSION_INNER_COLOR: LinearRgba = LinearRgba::new(3.0, 3.5, 4.0, 1.0);
/// Cool blue edge color for ice explosion material.
const ICE_EXPLOSION_OUTER_COLOR: LinearRgba = LinearRgba::new(0.3, 0.6, 2.5, 1.0);

/// Fresnel-based radial-gradient material for sphere explosion meshes.
/// Uses normal·view_dir instead of UVs for proper 3D sphere gradient.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct FireExplosionSphereMaterial {
    #[uniform(0)]
    pub inner_color: LinearRgba,
    #[uniform(0)]
    pub outer_color: LinearRgba,
    #[uniform(0)]
    pub opacity: f32,
}

impl Material for FireExplosionSphereMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/fire_explosion_sphere.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

/// Animated swirling-energy material for aura spheres.
/// Uses Fresnel edge glow + procedural noise interior patterns.
/// All instances share a global `time` uniform updated each frame.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct AuraSphereMaterial {
    #[uniform(0)]
    pub inner_color: LinearRgba,
    #[uniform(0)]
    pub outer_color: LinearRgba,
    #[uniform(0)]
    pub opacity: f32,
    #[uniform(0)]
    pub time: f32,
}

impl Material for AuraSphereMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/aura_sphere.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

// ── Aura color constants ────────────────────────────────────────────────
const KING_AURA_INNER: LinearRgba = LinearRgba::new(2.0, 1.8, 0.6, 1.0);
const KING_AURA_OUTER: LinearRgba = LinearRgba::new(0.4, 0.5, 1.5, 1.0);
const HEALING_AURA_INNER: LinearRgba = LinearRgba::new(0.3, 2.0, 0.5, 1.0);
const HEALING_AURA_OUTER: LinearRgba = LinearRgba::new(0.1, 1.0, 0.3, 1.0);
const GUARDIAN_AURA_INNER: LinearRgba = LinearRgba::new(0.5, 2.5, 3.0, 1.0);
const GUARDIAN_AURA_OUTER: LinearRgba = LinearRgba::new(0.8, 1.5, 2.5, 1.0);
const BATTLE_HYMN_AURA_INNER: LinearRgba = LinearRgba::new(2.5, 2.0, 0.4, 1.0);
const BATTLE_HYMN_AURA_OUTER: LinearRgba = LinearRgba::new(2.0, 1.2, 0.2, 1.0);
const HASTE_AURA_INNER: LinearRgba = LinearRgba::new(1.5, 2.5, 0.5, 1.0);
const HASTE_AURA_OUTER: LinearRgba = LinearRgba::new(2.0, 2.0, 0.3, 1.0);
const BERSERKER_AURA_INNER: LinearRgba = LinearRgba::new(2.5, 0.3, 0.2, 1.0);
const BERSERKER_AURA_OUTER: LinearRgba = LinearRgba::new(2.0, 0.8, 0.1, 1.0);
const SLEEP_AURA_INNER: LinearRgba = LinearRgba::new(2.5, 2.5, 3.0, 1.0);
const SLEEP_AURA_OUTER: LinearRgba = LinearRgba::new(1.5, 1.5, 2.0, 1.0);
const RAISE_DEAD_AURA_INNER: LinearRgba = LinearRgba::new(1.0, 0.3, 2.5, 1.0);
const RAISE_DEAD_AURA_OUTER: LinearRgba = LinearRgba::new(0.6, 0.2, 1.5, 1.0);
const COMMANDER_AURA_INNER: LinearRgba = LinearRgba::new(2.0, 1.5, 0.4, 1.0);
const COMMANDER_AURA_OUTER: LinearRgba = LinearRgba::new(2.0, 1.8, 0.6, 1.0);
const CRYSTAL_AURA_INNER: LinearRgba = LinearRgba::new(2.5, 0.8, 2.0, 1.0);
const CRYSTAL_AURA_OUTER: LinearRgba = LinearRgba::new(1.5, 0.3, 1.8, 1.0);
const TELEPORT_AURA_INNER: LinearRgba = LinearRgba::new(0.5, 1.5, 3.0, 1.0);
const TELEPORT_AURA_OUTER: LinearRgba = LinearRgba::new(0.3, 0.8, 2.5, 1.0);

/// Clones a sphere explosion material template into a unique per-entity handle.
pub fn clone_sphere_material(
    materials: &mut Assets<FireExplosionSphereMaterial>,
    template: &Handle<FireExplosionSphereMaterial>,
) -> Handle<FireExplosionSphereMaterial> {
    let mat = materials
        .get(template)
        .expect("sphere material template")
        .clone();
    materials.add(mat)
}

/// Shared fade fraction for all explosion sphere fade-out effects.
pub const EXPLOSION_FADE_FRACTION: f32 = 0.4;

/// Computes opacity for an explosion fade-out (1.0 → 0.0 over the last portion of lifetime).
pub fn explosion_fade_opacity(progress: f32) -> f32 {
    let remaining = 1.0 - progress.min(1.0);
    if remaining < EXPLOSION_FADE_FRACTION {
        remaining / EXPLOSION_FADE_FRACTION
    } else {
        1.0
    }
}

/// Pre-allocated meshes and materials for all spell visuals.
///
/// Initialized once at startup. Local spell spawn functions and multiplayer
/// ghost rendering both clone handles from this resource instead of creating
/// their own meshes/materials.
#[derive(Resource)]
pub struct SpellVisualAssets {
    // ── Base meshes (unit size, scaled by Transform) ──────────────────────
    pub unit_circle: Handle<Mesh>,
    pub unit_cuboid: Handle<Mesh>,
    pub unit_rect: Handle<Mesh>,
    /// 3-plane cross sphere (XY + XZ + YZ, double-sided). Low-poly sphere for 2D aesthetic.
    pub cross_plane_sphere: Handle<Mesh>,
    /// Icosphere mesh for spherical explosion effects (Fresnel-shaded).
    pub explosion_sphere: Handle<Mesh>,
    /// 2-plane cross cylinder (2 quads along Y axis, double-sided). Low-poly cylinder for beams.
    pub cross_plane_cylinder: Handle<Mesh>,
    /// 2-plane cross triangle (tapers from point at Y=0 to full width at Y=height, double-sided).
    pub cross_plane_triangle: Handle<Mesh>,
    /// Low-resolution torus (unit-scale) for black hole rings.
    pub black_hole_torus: Handle<Mesh>,
    /// Unit-scale torus for psychic shockwave ring.
    pub shockwave_torus: Handle<Mesh>,
    /// Flat annulus ring for entangle vine arches.
    pub entangle_vine_ring: Handle<Mesh>,
    // ── Zone materials (semi-transparent ground circles) ──────────────────
    pub spike_growth_zone: Handle<StandardMaterial>,
    pub healing_plume_zone: Handle<StandardMaterial>,
    pub entangle_zone: Handle<StandardMaterial>,
    pub fog_cloud_zone: Handle<StandardMaterial>,
    pub grease_zone: Handle<StandardMaterial>,
    pub grease_fire: Handle<StandardMaterial>,
    pub plague_wind_zone: Handle<StandardMaterial>,
    pub meteor_ground_fire: Handle<StandardMaterial>,
    /// Green semi-transparent material for entangle vine toruses.
    pub entangle_vine: Handle<StandardMaterial>,
    /// Dark green material for spike growth vine rings.
    pub spike_growth_vine: Handle<StandardMaterial>,
    /// Dark red material for spike growth spike rings.
    pub spike_growth_spike: Handle<StandardMaterial>,
    /// Darker red material for spike storm projectiles.
    pub spike_storm_projectile: Handle<StandardMaterial>,

    // ── Aura sphere materials (animated swirling energy) ──────────────────
    pub king_aura_sphere: Handle<AuraSphereMaterial>,
    pub healing_aura_sphere: Handle<AuraSphereMaterial>,
    pub guardian_aura_sphere: Handle<AuraSphereMaterial>,
    pub battle_hymn_aura_sphere: Handle<AuraSphereMaterial>,
    pub haste_aura_sphere: Handle<AuraSphereMaterial>,
    pub berserker_aura_sphere: Handle<AuraSphereMaterial>,
    pub sleep_aura_sphere: Handle<AuraSphereMaterial>,
    pub raise_dead_aura_sphere: Handle<AuraSphereMaterial>,
    pub commander_aura_sphere: Handle<AuraSphereMaterial>,
    pub crystal_aura_sphere: Handle<AuraSphereMaterial>,
    pub teleport_aura_sphere: Handle<AuraSphereMaterial>,

    // ── Casting indicator materials (translucent circles shown while aiming) ──
    pub haste_indicator: Handle<StandardMaterial>,
    pub battle_hymn_indicator: Handle<StandardMaterial>,
    pub berserker_rage_indicator: Handle<StandardMaterial>,
    pub sleep_indicator: Handle<StandardMaterial>,
    pub guardian_circle_indicator: Handle<StandardMaterial>,
    pub spike_growth_indicator: Handle<StandardMaterial>,
    pub healing_plume_indicator: Handle<StandardMaterial>,
    pub entangle_indicator: Handle<StandardMaterial>,
    pub fog_cloud_indicator: Handle<StandardMaterial>,
    pub grease_indicator: Handle<StandardMaterial>,
    pub plague_wind_indicator: Handle<StandardMaterial>,
    pub arcane_crystal_indicator: Handle<StandardMaterial>,
    pub lightning_rod_indicator: Handle<StandardMaterial>,
    pub meteor_fall_indicator: Handle<StandardMaterial>,
    pub squall_indicator: Handle<StandardMaterial>,
    pub fireball_indicator: Handle<StandardMaterial>,
    pub black_hole_indicator: Handle<StandardMaterial>,
    pub raise_the_dead_indicator: Handle<StandardMaterial>,
    pub teleport_destination: Handle<StandardMaterial>,
    pub teleport_source: Handle<StandardMaterial>,
    pub telekinesis_indicator: Handle<StandardMaterial>,
    pub mind_control_indicator: Handle<StandardMaterial>,

    // ── Object materials ─────────────────────────────────────────────────
    pub black_hole: Handle<StandardMaterial>,
    /// Pure-black billboard material for the black hole circle.
    pub black_hole_billboard: Handle<StandardMaterial>,
    /// White emissive material for the billboard torus ring.
    pub black_hole_ring: Handle<StandardMaterial>,
    /// Dark red material for the accretion disk circle.
    pub black_hole_accretion: Handle<StandardMaterial>,
    /// Warm-white emissive material for the accretion disk torus ring.
    pub black_hole_accretion_ring: Handle<StandardMaterial>,
    pub arcane_crystal: Handle<StandardMaterial>,
    pub lightning_rod: Handle<StandardMaterial>,

    // ── Wall materials ───────────────────────────────────────────────────
    pub wall_of_stone: Handle<WallOfStoneMaterial>,
    pub wall_of_fire: Handle<StandardMaterial>,

    // ── Explosion materials ──────────────────────────────────────────────
    pub fireball_explosion_sphere: Handle<FireExplosionSphereMaterial>,
    pub ice_explosion_sphere: Handle<FireExplosionSphereMaterial>,
    pub ice_explosion: Handle<StandardMaterial>,

    // ── Projectile materials ─────────────────────────────────────────────
    pub fireball_projectile: Handle<StandardMaterial>,
    pub ice_projectile: Handle<StandardMaterial>,
    pub meteor_projectile: Handle<StandardMaterial>,
    pub magic_missile: Handle<StandardMaterial>,
    /// Light gray bullet tracer material (speed-line look).
    pub bullet_tracer: Handle<StandardMaterial>,
    /// Bright white material for bullet hit flash overlay.
    pub bullet_hit_flash: Handle<StandardMaterial>,

    // ── Arc/Beam materials ───────────────────────────────────────────────
    pub chain_lightning_arc: Handle<StandardMaterial>,
    pub lightning_strike: Handle<StandardMaterial>,
    pub lightning_rod_arc: Handle<StandardMaterial>,
    pub crystal_beam: Handle<StandardMaterial>,
    pub crystal_arc: Handle<StandardMaterial>,
    pub finger_of_death_beam: Handle<StandardMaterial>,
    pub disintegrate_beam: Handle<StandardMaterial>,
    pub disintegrate_glow: Handle<StandardMaterial>,
    pub disintegrate_flare: Handle<StandardMaterial>,
    pub disintegrate_particle: Handle<StandardMaterial>,

    // ── Fire VFX materials (shared by fireball, meteor, etc.) ─────────
    pub fire_glow: Handle<StandardMaterial>,
    pub fire_spark: Handle<StandardMaterial>,
    pub fire_smoke: Handle<StandardMaterial>,
    /// White translucent smoke for ice/frost explosion effects.
    pub ice_smoke: Handle<StandardMaterial>,
    pub fire_black_smoke: Handle<StandardMaterial>,
    pub fire_orange_smoke: Handle<StandardMaterial>,
    /// Lighter, more yellow-orange variant for fire smoke.
    pub fire_orange_smoke_light: Handle<StandardMaterial>,
    /// Deeper, more red-orange variant for fire smoke.
    pub fire_orange_smoke_deep: Handle<StandardMaterial>,
    /// Bright emissive ember material for flickering base glow (additive).
    pub fire_ember: Handle<StandardMaterial>,
    /// Procedural fire particle material — noise + color ramp + dissolve.
    /// Shared by ALL fire smoke puffs for GPU batching (single draw call).
    pub fire_particle: Handle<super::vfx::fire_material::FireParticleMaterial>,
    /// Smoke particle material with circular radial falloff.
    /// Shared by ALL dark smoke puffs for GPU batching.
    pub smoke_particle: Handle<super::vfx::fire_material::SmokeParticleMaterial>,

    // ── Disintegrate smoke material ─────────────────────────────────────
    pub disintegrate_smoke: Handle<StandardMaterial>,

    // ── Disintegrate talent materials ────────────────────────────────────
    /// Bright white-gold material for searing finale detonation.
    pub searing_finale: Handle<StandardMaterial>,
    /// Dark glowing ground eclipse at beam impact point.
    pub disintegrate_eclipse: Handle<StandardMaterial>,

    // ── Finger of Death VFX materials ────────────────────────────────────
    pub necrotic_vein: Handle<StandardMaterial>,
    pub finger_of_death_glow: Handle<StandardMaterial>,
    pub finger_of_death_flare: Handle<StandardMaterial>,
    pub necrotic_pulse: Handle<StandardMaterial>,

    // ── Telekinesis talent materials ────────────────────────────────────────
    pub shockwave_material: Handle<StandardMaterial>,
    pub harvest_flash_material: Handle<StandardMaterial>,

    // ── Crystal mini-spell materials ──────────────────────────────────────
    pub crystal_mini_missile: Handle<StandardMaterial>,
    pub crystal_range_indicator: Handle<StandardMaterial>,

    // ── Dust smoke materials (wall of stone rising/sinking) ──────────
    pub dust_smoke: Handle<StandardMaterial>,
    pub dust_smoke_light: Handle<StandardMaterial>,
    pub dust_smoke_dark: Handle<StandardMaterial>,

    // ── Plague smoke material (poison cloud) ─────────────────────────
    pub plague_smoke: Handle<StandardMaterial>,

    // ── Fog smoke material (gray fog cloud puffs) ──────────────────
    pub fog_smoke: Handle<StandardMaterial>,

    // ── Snow particle material (squall storm) ─────────────────────────
    pub snow_particle: Handle<StandardMaterial>,

    // ── Heat shimmer material (fire haze) ─────────────────────────────
    pub heat_shimmer: Handle<StandardMaterial>,

    // ── Magic missile VFX materials ────────────────────────────────────
    pub missile_glow: Handle<StandardMaterial>,
    pub missile_sparkle: Handle<StandardMaterial>,

    // ── Grease VFX materials ─────────────────────────────────────────────
    /// Yellowish-brown fume wisp rising off grease zones.
    pub grease_fume: Handle<StandardMaterial>,
    /// Translucent dark brown bubble particle for grease zones.
    pub grease_bubble: Handle<StandardMaterial>,
    /// Dark brown splatter particle for grease zone edges.
    pub grease_splatter: Handle<StandardMaterial>,

    // ── Mark of Death materials ───────────────────────────────────────────
    pub mark_indicator: Handle<StandardMaterial>,

    // ── Ambient mote material (floating pollen/dust) ─────────────────────
    pub ambient_mote: Handle<StandardMaterial>,

    // ── Special meshes (fixed-size) ──────────────────────────────────────
    pub magic_missile_mesh: Handle<Mesh>,
    /// Flat quad mesh for particles (2 tris, double-sided).
    pub particle_quad: Handle<Mesh>,
    /// Arrow mesh for directional indicators (triangle, double-sided, points along +Y).
    pub arrow_mesh: Handle<Mesh>,

    // ── Arrow indicator material ──────────────────────────────────────────
    pub plague_wind_arrow: Handle<StandardMaterial>,

    // ── Banishment VFX material ─────────────────────────────────────────
    /// Semi-transparent dark purple lensing sphere for banishment effect.
    pub banishment_lens: Handle<StandardMaterial>,
    pub banishment_spark: Handle<StandardMaterial>,
    pub dispel_spark: Handle<StandardMaterial>,

    // ── Cast flare materials (per spell school) ─────────────────────────
    /// Bright orange/yellow glow for fire school spells.
    pub flare_fire_glow: Handle<StandardMaterial>,
    pub flare_fire_spark: Handle<StandardMaterial>,
    /// White/electric blue for lightning school spells.
    pub flare_lightning_glow: Handle<StandardMaterial>,
    pub flare_lightning_spark: Handle<StandardMaterial>,
    /// Purple/violet for arcane school spells.
    pub flare_arcane_glow: Handle<StandardMaterial>,
    pub flare_arcane_spark: Handle<StandardMaterial>,
    /// Green for nature school spells.
    pub flare_nature_glow: Handle<StandardMaterial>,
    pub flare_nature_spark: Handle<StandardMaterial>,
    /// Gold/white for holy/buff school spells.
    pub flare_holy_glow: Handle<StandardMaterial>,
    pub flare_holy_spark: Handle<StandardMaterial>,
    /// Dark purple for dark school spells.
    pub flare_dark_glow: Handle<StandardMaterial>,
    pub flare_dark_spark: Handle<StandardMaterial>,
    /// Light blue for force school spells.
    pub flare_force_glow: Handle<StandardMaterial>,
    pub flare_force_spark: Handle<StandardMaterial>,
    /// Amber/brown for transmutation school spells.
    pub flare_transmutation_glow: Handle<StandardMaterial>,
    pub flare_transmutation_spark: Handle<StandardMaterial>,

    // ── Floating mote materials ──────────────────────────────────────────
    /// Soft blue/purple motes for sleep zones.
    pub sleep_mote: Handle<StandardMaterial>,
    /// Golden sparkle motes for healing zones.
    pub healing_mote: Handle<StandardMaterial>,
    /// Gold shimmer motes for buff zones (haste, guardian circle, battle hymn).
    pub buff_mote: Handle<StandardMaterial>,
    /// Green sparkle motes for entangle/nature zones.
    pub nature_mote: Handle<StandardMaterial>,

    // ── Smoke poof materials ─────────────────────────────────────────────
    /// Amber/brown smoke for polymorph transformation.
    pub polymorph_poof: Handle<StandardMaterial>,
    /// Dark purple smoke for banishment void.
    pub banishment_poof: Handle<StandardMaterial>,
}

/// Initializes the shared spell visual assets resource.
#[allow(clippy::too_many_arguments)]
pub fn init_spell_visual_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fire_explosion_sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    mut fire_particle_materials: ResMut<Assets<super::vfx::fire_material::FireParticleMaterial>>,
    mut smoke_particle_materials: ResMut<Assets<super::vfx::fire_material::SmokeParticleMaterial>>,
    mut wall_of_stone_materials: ResMut<Assets<WallOfStoneMaterial>>,
    mut aura_sphere_materials: ResMut<Assets<AuraSphereMaterial>>,
    battlefield_assets: Res<BattlefieldAssets>,
) {
    let unlit = |color: Color| StandardMaterial {
        base_color: color,
        unlit: true,
        ..default()
    };

    let unlit_blend = |color: Color| StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    };

    // Same as unlit_blend but with negative depth_bias so indicators render behind units
    let indicator_mat = |color: Color| StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        depth_bias: -100.0,
        ..default()
    };

    commands.insert_resource(SpellVisualAssets {
        // Base meshes
        unit_circle: meshes.add(Circle::new(1.0)),
        unit_cuboid: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        unit_rect: meshes.add(Rectangle::new(1.0, 1.0)),

        // Zone materials (persistent ground effects)
        spike_growth_zone: materials.add(unlit_blend(Color::srgba(0.15, 0.4, 0.05, 0.4))),
        healing_plume_zone: materials.add(unlit_blend(Color::srgba(0.1, 0.7, 0.2, 0.4))),
        entangle_zone: materials.add(unlit_blend(Color::srgba(0.1, 0.6, 0.15, 0.35))),
        fog_cloud_zone: materials.add(unlit_blend(Color::srgba(0.6, 0.65, 0.7, 0.35))),
        grease_zone: materials.add(unlit_blend(Color::srgba(0.45, 0.4, 0.05, 0.4))),
        grease_fire: materials.add(unlit_blend(Color::srgba(0.9, 0.3, 0.05, 0.55))),
        plague_wind_zone: materials.add(unlit_blend(Color::srgba(0.2, 0.6, 0.1, 0.4))),
        meteor_ground_fire: materials.add(unlit_blend(Color::srgba(0.9, 0.25, 0.05, 0.5))),
        entangle_vine: materials.add(unlit_blend(Color::srgba(0.05, 0.3, 0.05, 0.75))),
        spike_growth_vine: materials.add(unlit_blend(Color::srgba(0.08, 0.25, 0.05, 0.7))),
        spike_growth_spike: materials.add(unlit_blend(Color::srgba(0.5, 0.08, 0.08, 0.75))),
        spike_storm_projectile: materials.add(unlit_blend(Color::srgba(0.6, 0.1, 0.1, 0.9))),

        // Casting indicator materials (translucent circles shown while aiming, depth_bias pushes behind units)
        haste_indicator: materials.add(indicator_mat(Color::srgba(1.0, 0.85, 0.0, 0.3))),
        battle_hymn_indicator: materials.add(indicator_mat(Color::srgba(1.0, 0.85, 0.0, 0.3))),
        berserker_rage_indicator: materials.add(indicator_mat(Color::srgba(0.9, 0.15, 0.1, 0.3))),
        sleep_indicator: materials.add(indicator_mat(Color::srgba(0.4, 0.3, 0.7, 0.3))),
        guardian_circle_indicator: materials.add(indicator_mat(Color::srgba(0.0, 0.8, 1.0, 0.3))),
        spike_growth_indicator: materials.add(indicator_mat(Color::srgba(0.2, 0.45, 0.1, 0.3))),
        healing_plume_indicator: materials.add(indicator_mat(Color::srgba(0.2, 0.8, 0.3, 0.3))),
        entangle_indicator: materials.add(indicator_mat(Color::srgba(0.1, 0.7, 0.2, 0.3))),
        fog_cloud_indicator: materials.add(indicator_mat(Color::srgba(0.7, 0.75, 0.8, 0.3))),
        grease_indicator: materials.add(indicator_mat(Color::srgba(0.25, 0.2, 0.05, 0.45))),
        plague_wind_indicator: materials.add(indicator_mat(Color::srgba(0.3, 0.8, 0.1, 0.3))),
        arcane_crystal_indicator: materials.add(indicator_mat(Color::srgba(0.5, 0.2, 0.8, 0.3))),
        lightning_rod_indicator: materials.add(indicator_mat(Color::srgba(0.7, 0.85, 1.0, 0.4))),
        meteor_fall_indicator: materials.add(indicator_mat(Color::srgba(0.9, 0.3, 0.1, 0.25))),
        squall_indicator: materials.add(indicator_mat(Color::srgba(0.3, 0.8, 1.0, 0.4))),
        fireball_indicator: materials.add(indicator_mat(Color::srgba(1.0, 0.4, 0.1, 0.3))),
        black_hole_indicator: materials.add(indicator_mat(Color::srgba(0.4, 0.1, 0.6, 0.3))),
        raise_the_dead_indicator: materials.add(indicator_mat(Color::srgba(0.3, 0.9, 0.2, 0.3))),
        teleport_destination: materials.add(indicator_mat(Color::srgba(0.0, 0.6, 1.0, 0.25))),
        teleport_source: materials.add(indicator_mat(Color::srgba(0.0, 0.8, 1.0, 0.35))),
        telekinesis_indicator: materials.add(indicator_mat(Color::srgba(0.6, 0.9, 1.0, 0.7))),
        mind_control_indicator: materials.add(indicator_mat(
            super::mind_control::constants::INDICATOR_COLOR,
        )),

        // Object materials
        black_hole: materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.0, 0.1),
            emissive: bevy::color::LinearRgba::new(0.2, 0.0, 0.4, 1.0),
            depth_bias: 100.0,
            ..default()
        }),
        black_hole_billboard: materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.0, 0.0, 0.99),
            unlit: true,
            cull_mode: None,
            alpha_mode: AlphaMode::Blend,
            depth_bias: 100.0,
            ..default()
        }),
        black_hole_ring: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.85, 0.8, 0.9),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(2.5, 1.5, 1.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            depth_bias: 100.0,
            ..default()
        }),
        black_hole_accretion: materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.0, 0.0, 0.99),
            unlit: true,
            cull_mode: None,
            alpha_mode: AlphaMode::Blend,
            depth_bias: 100.0,
            ..default()
        }),
        black_hole_accretion_ring: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.85, 0.8, 0.9),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(2.5, 1.5, 1.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            depth_bias: 100.0,
            ..default()
        }),
        arcane_crystal: materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.1, 0.9),
            emissive: bevy::color::LinearRgba::new(0.4, 0.05, 0.6, 1.0),
            ..default()
        }),
        lightning_rod: materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.6, 0.65),
            metallic: 0.8,
            perceptual_roughness: 0.3,
            ..default()
        }),

        // Wall materials
        wall_of_stone: {
            let dark = STONE_COLOR_DARK.to_srgba();
            let light = STONE_COLOR_LIGHT.to_srgba();
            wall_of_stone_materials.add(WallOfStoneMaterial {
                tile_params: Vec4::new(
                    TILE_WORLD_SIZE,
                    TILE_COUNT as f32,
                    0.0,
                    GROUND_NOISE_BASE,
                ),
                damage_tint: Vec4::ZERO,
                side_dark: Vec4::new(dark.red, dark.green, dark.blue, 1.0),
                side_light: Vec4::new(light.red, light.green, light.blue, 1.0),
                base_texture: battlefield_assets.battlefield_tiles.clone(),
            })
        },
        wall_of_fire: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.5, 0.0, 0.4),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(2.0, 0.8, 0.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),

        // Explosion materials
        fireball_explosion_sphere: fire_explosion_sphere_materials.add(
            FireExplosionSphereMaterial {
                inner_color: FIRE_EXPLOSION_INNER_COLOR,
                outer_color: FIRE_EXPLOSION_OUTER_COLOR,
                opacity: 1.0,
            },
        ),
        ice_explosion_sphere: fire_explosion_sphere_materials.add(
            FireExplosionSphereMaterial {
                inner_color: ICE_EXPLOSION_INNER_COLOR,
                outer_color: ICE_EXPLOSION_OUTER_COLOR,
                opacity: 1.0,
            },
        ),
        ice_explosion: materials.add(unlit(Color::srgb(0.3, 0.8, 1.0))),

        // Aura sphere materials (swirling energy)
        king_aura_sphere: aura_mat(&mut aura_sphere_materials, KING_AURA_INNER, KING_AURA_OUTER),
        healing_aura_sphere: aura_mat(&mut aura_sphere_materials, HEALING_AURA_INNER, HEALING_AURA_OUTER),
        guardian_aura_sphere: aura_mat(&mut aura_sphere_materials, GUARDIAN_AURA_INNER, GUARDIAN_AURA_OUTER),
        battle_hymn_aura_sphere: aura_mat(&mut aura_sphere_materials, BATTLE_HYMN_AURA_INNER, BATTLE_HYMN_AURA_OUTER),
        haste_aura_sphere: aura_mat(&mut aura_sphere_materials, HASTE_AURA_INNER, HASTE_AURA_OUTER),
        berserker_aura_sphere: aura_mat(&mut aura_sphere_materials, BERSERKER_AURA_INNER, BERSERKER_AURA_OUTER),
        sleep_aura_sphere: aura_mat(&mut aura_sphere_materials, SLEEP_AURA_INNER, SLEEP_AURA_OUTER),
        raise_dead_aura_sphere: aura_mat(&mut aura_sphere_materials, RAISE_DEAD_AURA_INNER, RAISE_DEAD_AURA_OUTER),
        commander_aura_sphere: aura_mat(&mut aura_sphere_materials, COMMANDER_AURA_INNER, COMMANDER_AURA_OUTER),
        crystal_aura_sphere: aura_mat(&mut aura_sphere_materials, CRYSTAL_AURA_INNER, CRYSTAL_AURA_OUTER),
        teleport_aura_sphere: aura_sphere_materials.add(AuraSphereMaterial {
            inner_color: TELEPORT_AURA_INNER, outer_color: TELEPORT_AURA_OUTER, opacity: 1.0, time: 0.0,
        }),

        // Projectile materials
        fireball_projectile: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.5, 0.0),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(3.0, 1.5, 0.0, 1.0),
            ..default()
        }),
        ice_projectile: materials.add(unlit(Color::srgb(0.7, 0.9, 1.0))),
        meteor_projectile: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.4, 0.1),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(3.0, 1.2, 0.1, 1.0),
            ..default()
        }),
        magic_missile: materials.add(unlit(Color::srgb(1.0, 0.4, 0.8))),
        bullet_tracer: materials.add(unlit(Color::srgb(0.75, 0.75, 0.75))),
        bullet_hit_flash: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, 0.8),
            emissive: LinearRgba::new(5.0, 5.0, 5.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),

        // Arc/Beam materials
        chain_lightning_arc: materials.add(unlit(Color::srgb(0.7, 0.85, 1.0))),
        lightning_strike: materials.add(unlit(Color::srgb(0.8, 0.9, 1.0))),
        lightning_rod_arc: materials.add(unlit(Color::srgb(0.7, 0.85, 1.0))),
        crystal_beam: materials.add(unlit(Color::srgb(1.0, 0.6, 0.1))),
        crystal_arc: materials.add(unlit(Color::srgba(0.6, 0.4, 1.0, 0.9))),
        finger_of_death_beam: materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.0, 0.8),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(1.5, 0.0, 2.5, 1.0),
            ..default()
        }),
        disintegrate_beam: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.6, 0.1),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(3.0, 1.5, 0.2, 1.0),
            ..default()
        }),
        disintegrate_glow: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.5, 0.1, 0.25),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(1.5, 0.7, 0.1, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        disintegrate_flare: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.95, 0.7),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(5.0, 4.0, 2.0, 1.0),
            ..default()
        }),
        disintegrate_particle: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.6, 0.2),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(2.0, 1.0, 0.2, 1.0),
            ..default()
        }),

        // Fire VFX materials (shared by fireball, meteor, etc.)
        fire_glow: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.4, 0.0, 0.2),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(2.0, 0.8, 0.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        fire_spark: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.8, 0.3),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(4.0, 3.0, 1.0, 1.0),
            alpha_mode: AlphaMode::Add,
            cull_mode: None,
            ..default()
        }),
        fire_smoke: materials.add(StandardMaterial {
            base_color: Color::srgba(0.05, 0.05, 0.05, 0.2),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        ice_smoke: materials.add(StandardMaterial {
            base_color: Color::srgba(0.9, 0.93, 1.0, 0.25),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        fire_black_smoke: materials.add(StandardMaterial {
            base_color: Color::srgba(0.02, 0.02, 0.02, 0.55),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        fire_orange_smoke: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.45, 0.05, 0.3),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(1.5, 0.5, 0.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        fire_orange_smoke_light: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.6, 0.1, 0.28),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(2.0, 0.8, 0.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        fire_orange_smoke_deep: materials.add(StandardMaterial {
            base_color: Color::srgba(0.9, 0.3, 0.02, 0.35),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(1.2, 0.3, 0.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        fire_ember: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.7, 0.2, 0.9),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(5.0, 3.0, 0.5, 1.0),
            alpha_mode: AlphaMode::Add,
            cull_mode: None,
            ..default()
        }),
        fire_particle: fire_particle_materials.add(
            super::vfx::fire_material::FireParticleMaterial { time: 0.0 },
        ),
        smoke_particle: smoke_particle_materials.add(
            super::vfx::fire_material::SmokeParticleMaterial {
                color: bevy::color::LinearRgba::new(0.02, 0.02, 0.02, 0.55),
            },
        ),

        // Dust smoke (earthy brown puffs for wall of stone rising/sinking)
        dust_smoke: materials.add(StandardMaterial {
            base_color: Color::srgba(0.55, 0.40, 0.25, 0.35),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        dust_smoke_light: materials.add(StandardMaterial {
            base_color: Color::srgba(0.65, 0.50, 0.30, 0.30),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        dust_smoke_dark: materials.add(StandardMaterial {
            base_color: Color::srgba(0.40, 0.28, 0.15, 0.40),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),

        // Plague smoke (sickly green poison cloud puffs)
        plague_smoke: materials.add(StandardMaterial {
            base_color: Color::srgba(0.15, 0.45, 0.08, 0.35),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),

        // Fog smoke (gray fog cloud puffs — denser than plague smoke)
        fog_smoke: materials.add(StandardMaterial {
            base_color: Color::srgba(0.55, 0.6, 0.65, 0.5),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),

        // Snow particle (white translucent swirling flakes)
        snow_particle: materials.add(StandardMaterial {
            base_color: Color::srgba(0.9, 0.95, 1.0, 0.4),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),

        // Heat shimmer (subtle warm haze near fire)
        heat_shimmer: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.9, 0.7, 0.1),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),

        // Grease fume wisps (yellowish-brown vapor rising off grease)
        grease_fume: materials.add(unlit_blend(Color::srgba(0.4, 0.35, 0.1, 0.35))),
        grease_bubble: materials.add(unlit_blend(Color::srgba(0.35, 0.3, 0.08, 0.45))),
        grease_splatter: materials.add(unlit_blend(Color::srgba(0.3, 0.22, 0.05, 0.5))),

        // Mark of Death indicator (purple circle above marked unit)
        mark_indicator: materials.add(StandardMaterial {
            base_color: Color::srgba(0.6, 0.0, 0.8, 0.7),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(1.5, 0.0, 2.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),

        // Ambient mote (soft pale green, very low alpha for subtle floating particles)
        ambient_mote: materials.add(StandardMaterial {
            base_color: Color::srgba(0.7, 0.85, 0.6, 0.18),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),

        // Disintegrate smoke
        disintegrate_smoke: materials.add(StandardMaterial {
            base_color: Color::srgba(0.05, 0.05, 0.05, 0.4),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),

        // Disintegrate talent materials
        searing_finale: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.95, 0.7, 0.8),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(5.0, 4.5, 2.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        disintegrate_eclipse: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.5, 0.0, 0.4),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(2.5, 1.0, 0.1, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),

        // Finger of Death VFX
        necrotic_vein: materials.add(StandardMaterial {
            base_color: Color::srgba(0.6, 0.0, 0.8, 0.8),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(1.5, 0.0, 2.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        finger_of_death_glow: materials.add(StandardMaterial {
            base_color: Color::srgba(0.4, 0.0, 0.6, 0.25),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(0.8, 0.0, 1.2, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        finger_of_death_flare: materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.7, 1.0),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(3.0, 1.0, 5.0, 1.0),
            ..default()
        }),
        necrotic_pulse: materials.add(StandardMaterial {
            base_color: Color::srgba(0.4, 0.0, 0.6, 0.5),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(0.8, 0.0, 1.2, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),

        // Magic missile VFX
        missile_glow: materials.add(StandardMaterial {
            base_color: Color::srgba(0.8, 0.3, 1.0, 0.2),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(2.0, 0.8, 3.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        missile_sparkle: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 1.0),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(4.0, 3.5, 5.0, 1.0),
            ..default()
        }),

        // Shockwave material (light blue, emissive, alpha blend)
        shockwave_material: materials.add(StandardMaterial {
            base_color: SHOCKWAVE_COLOR,
            unlit: true,
            emissive: bevy::color::LinearRgba::new(1.0, 2.0, 3.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        // Harvest flash material (bright light blue circle overlay)
        harvest_flash_material: materials.add(unlit_blend(HARVEST_FLASH_COLOR)),

        // Crystal mini-spell materials
        crystal_mini_missile: materials.add(unlit(Color::srgb(0.8, 0.3, 0.9))),
        crystal_range_indicator: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.4, 0.7, 0.2),
            emissive: LinearRgba::new(1.5, 0.4, 0.8, 1.0),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            depth_bias: -500.0,
            ..default()
        }),

        // Cross-plane sphere: 3 intersecting unit circles (XY, XZ, YZ), radius 1.0
        cross_plane_sphere: meshes.add(build_cross_plane_sphere(1.0)),
        // Icosphere for spherical explosion effects (unit radius, scaled by Transform)
        explosion_sphere: meshes.add(Sphere::new(1.0).mesh().ico(2).expect("icosphere mesh")),
        // Cross-plane cylinder: 2 intersecting quads along Y axis, radius 0.5, height 1.0
        cross_plane_cylinder: meshes.add(build_cross_plane_cylinder(0.5, 1.0)),
        // Cross-plane triangle: 2 intersecting triangles (point at Y=0, widens to radius at Y=height)
        cross_plane_triangle: meshes.add(build_cross_plane_triangle(0.5, 1.0)),
        // Low-poly torus for black hole rings (unit-scale, scaled by Transform)
        black_hole_torus: meshes.add(
            Torus::new(1.0 - TORUS_MINOR_RADIUS, 1.0 + TORUS_MINOR_RADIUS)
                .mesh()
                .major_resolution(16)
                .minor_resolution(8),
        ),
        // Torus for psychic shockwave ring (thicker tube than black hole)
        shockwave_torus: meshes.add(
            Torus::new(1.0 - SHOCKWAVE_TORUS_MINOR, 1.0 + SHOCKWAVE_TORUS_MINOR)
                .mesh()
                .major_resolution(24)
                .minor_resolution(8),
        ),
        // Flat annulus ring for entangle vine arches (inner 0.5, outer 1.0 = 50% ring width)
        entangle_vine_ring: meshes.add(Annulus::new(0.5, 1.0).mesh().resolution(16)),
        // Special meshes (magic missile radius = 5.0)
        magic_missile_mesh: meshes.add(build_cross_plane_sphere(5.0)),
        // Unit square in XY plane (2 tris, double-sided) for pixel-art particle effects.
        particle_quad: meshes.add({
            let h = 0.5_f32; // half-extent → 1x1 quad, scaled via Transform
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                bevy::asset::RenderAssetUsages::default(),
            );
            // Front face (4 verts) + back face (4 verts)
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_POSITION,
                vec![
                    [-h, -h, 0.0],
                    [h, -h, 0.0],
                    [h, h, 0.0],
                    [-h, h, 0.0], // front
                    [-h, -h, 0.0],
                    [-h, h, 0.0],
                    [h, h, 0.0],
                    [h, -h, 0.0], // back
                ],
            );
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                vec![
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0, -1.0],
                    [0.0, 0.0, -1.0],
                    [0.0, 0.0, -1.0],
                    [0.0, 0.0, -1.0],
                ],
            );
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_UV_0,
                vec![
                    [0.0, 1.0],
                    [1.0, 1.0],
                    [1.0, 0.0],
                    [0.0, 0.0],
                    [0.0, 1.0],
                    [0.0, 0.0],
                    [1.0, 0.0],
                    [1.0, 1.0],
                ],
            );
            mesh.insert_indices(Indices::U16(vec![
                0, 1, 2, 0, 2, 3, // front
                4, 5, 6, 4, 6, 7, // back
            ]));
            mesh
        }),
        // Arrow triangle pointing along +Y (from base at Y=0 to tip at Y=1), double-sided.
        arrow_mesh: meshes.add({
            let hw = 0.5_f32; // half-width at base
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                bevy::asset::RenderAssetUsages::default(),
            );
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_POSITION,
                vec![
                    [-hw, 0.0, 0.0],
                    [hw, 0.0, 0.0],
                    [0.0, 1.0, 0.0], // front
                    [-hw, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [hw, 0.0, 0.0], // back
                ],
            );
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                vec![
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0, -1.0],
                    [0.0, 0.0, -1.0],
                    [0.0, 0.0, -1.0],
                ],
            );
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_UV_0,
                vec![
                    [0.0, 1.0],
                    [1.0, 1.0],
                    [0.5, 0.0],
                    [0.0, 1.0],
                    [0.5, 0.0],
                    [1.0, 1.0],
                ],
            );
            mesh.insert_indices(Indices::U16(vec![0, 1, 2, 3, 4, 5]));
            mesh
        }),
        plague_wind_arrow: materials.add(unlit_blend(Color::srgba(0.3, 0.8, 0.1, 0.5))),
        banishment_lens: materials.add(unlit_blend(Color::srgba(0.1, 0.5, 0.2, 0.6))),
        banishment_spark: materials.add(unlit_blend(Color::srgba(0.2, 0.9, 0.3, 0.8))),
        dispel_spark: materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.9, 1.0),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(4.0, 4.0, 5.0, 1.0),
            ..default()
        }),

        // ── Cast flare materials ──────────────────────────────────────
        // Fire school (orange/yellow)
        flare_fire_glow: materials.add(unlit_blend(Color::srgba(1.0, 0.7, 0.1, 0.6))),
        flare_fire_spark: materials.add(unlit(Color::srgb(1.0, 0.6, 0.1))),
        // Lightning school (white/electric blue)
        flare_lightning_glow: materials.add(unlit_blend(Color::srgba(0.8, 0.9, 1.0, 0.7))),
        flare_lightning_spark: materials.add(unlit(Color::srgb(0.8, 0.9, 1.0))),
        // Arcane school (purple/violet)
        flare_arcane_glow: materials.add(unlit_blend(Color::srgba(0.6, 0.2, 0.9, 0.6))),
        flare_arcane_spark: materials.add(unlit(Color::srgb(0.6, 0.2, 0.9))),
        // Nature school (green)
        flare_nature_glow: materials.add(unlit_blend(Color::srgba(0.2, 0.8, 0.2, 0.6))),
        flare_nature_spark: materials.add(unlit(Color::srgb(0.2, 0.8, 0.2))),
        // Holy/buff school (gold/white)
        flare_holy_glow: materials.add(unlit_blend(Color::srgba(1.0, 0.9, 0.4, 0.7))),
        flare_holy_spark: materials.add(unlit(Color::srgb(1.0, 0.9, 0.4))),
        // Dark school (dark purple)
        flare_dark_glow: materials.add(unlit_blend(Color::srgba(0.3, 0.0, 0.4, 0.6))),
        flare_dark_spark: materials.add(unlit(Color::srgb(0.4, 0.1, 0.5))),
        // Force school (light blue)
        flare_force_glow: materials.add(unlit_blend(Color::srgba(0.4, 0.7, 1.0, 0.6))),
        flare_force_spark: materials.add(unlit(Color::srgb(0.4, 0.7, 1.0))),
        // Transmutation school (amber/brown)
        flare_transmutation_glow: materials.add(unlit_blend(Color::srgba(0.8, 0.5, 0.1, 0.6))),
        flare_transmutation_spark: materials.add(unlit(Color::srgb(0.8, 0.5, 0.1))),

        // ── Floating mote materials ───────────────────────────────────
        sleep_mote: materials.add(unlit_blend(Color::srgba(0.5, 0.4, 0.9, 0.5))),
        healing_mote: materials.add(unlit_blend(Color::srgba(1.0, 0.9, 0.3, 0.5))),
        buff_mote: materials.add(unlit_blend(Color::srgba(1.0, 0.85, 0.2, 0.4))),
        nature_mote: materials.add(unlit_blend(Color::srgba(0.3, 0.9, 0.3, 0.4))),

        // ── Smoke poof materials ──────────────────────────────────────
        polymorph_poof: materials.add(unlit_blend(Color::srgba(0.7, 0.5, 0.2, 0.6))),
        banishment_poof: materials.add(unlit_blend(Color::srgba(0.2, 0.0, 0.3, 0.6))),
    });
}

impl SpellVisualAssets {
    /// Returns references to all material handles for batch operations.
    fn all_material_handles(&self) -> Vec<&Handle<StandardMaterial>> {
        vec![
            // Zones
            &self.spike_growth_zone,
            &self.healing_plume_zone,
            &self.entangle_zone,
            &self.fog_cloud_zone,
            &self.grease_zone,
            &self.grease_fire,
            &self.plague_wind_zone,
            &self.meteor_ground_fire,
            &self.entangle_vine,
            &self.spike_growth_vine,
            &self.spike_growth_spike,
            &self.spike_storm_projectile,
            // Indicators
            &self.haste_indicator,
            &self.battle_hymn_indicator,
            &self.berserker_rage_indicator,
            &self.sleep_indicator,
            &self.guardian_circle_indicator,
            &self.spike_growth_indicator,
            &self.healing_plume_indicator,
            &self.entangle_indicator,
            &self.fog_cloud_indicator,
            &self.grease_indicator,
            &self.plague_wind_indicator,
            &self.arcane_crystal_indicator,
            &self.lightning_rod_indicator,
            &self.meteor_fall_indicator,
            &self.squall_indicator,
            &self.teleport_destination,
            &self.teleport_source,
            &self.telekinesis_indicator,
            &self.mind_control_indicator,
            // Objects
            &self.black_hole,
            &self.black_hole_billboard,
            &self.black_hole_ring,
            &self.black_hole_accretion,
            &self.black_hole_accretion_ring,
            &self.arcane_crystal,
            &self.lightning_rod,
            // Walls (wall_of_stone uses WallOfStoneMaterial, handled separately)
            &self.wall_of_fire,
            // Explosions (sphere materials handled separately in refresh_spell_visuals_for_wizard)
            &self.ice_explosion,
            // Projectiles
            &self.fireball_projectile,
            &self.ice_projectile,
            &self.meteor_projectile,
            &self.magic_missile,
            // Arcs/Beams
            &self.chain_lightning_arc,
            &self.lightning_strike,
            &self.lightning_rod_arc,
            &self.crystal_beam,
            &self.crystal_arc,
            &self.finger_of_death_beam,
            &self.disintegrate_beam,
            &self.disintegrate_glow,
            &self.disintegrate_flare,
            &self.disintegrate_particle,
            // Fire VFX
            &self.fire_glow,
            &self.fire_spark,
            &self.fire_smoke,
            &self.fire_orange_smoke,
            &self.fire_orange_smoke_light,
            &self.fire_orange_smoke_deep,
            &self.fire_black_smoke,
            // Disintegrate VFX
            &self.disintegrate_smoke,
            &self.searing_finale,
            &self.disintegrate_eclipse,
            // Finger of Death VFX
            &self.necrotic_vein,
            &self.finger_of_death_glow,
            &self.finger_of_death_flare,
            &self.necrotic_pulse,
            // Magic Missile VFX
            &self.missile_glow,
            &self.missile_sparkle,
            // Telekinesis VFX
            &self.shockwave_material,
            &self.harvest_flash_material,
            // Crystal VFX
            &self.crystal_mini_missile,
            &self.crystal_range_indicator,
            // Heat shimmer
            &self.heat_shimmer,
            // Snow particle
            &self.snow_particle,
            // Dust smoke
            &self.dust_smoke,
            &self.dust_smoke_light,
            &self.dust_smoke_dark,
            // Grease VFX
            &self.grease_fume,
            &self.grease_bubble,
            &self.grease_splatter,
            // Mark of Death
            &self.mark_indicator,
            // Ambient motes
            &self.ambient_mote,
            // Arrow indicator
            &self.plague_wind_arrow,
            // Banishment
            &self.banishment_lens,
            &self.banishment_spark,
            // Dispel
            &self.dispel_spark,
            // Cast flares
            &self.flare_fire_glow,
            &self.flare_fire_spark,
            &self.flare_lightning_glow,
            &self.flare_lightning_spark,
            &self.flare_arcane_glow,
            &self.flare_arcane_spark,
            &self.flare_nature_glow,
            &self.flare_nature_spark,
            &self.flare_holy_glow,
            &self.flare_holy_spark,
            &self.flare_dark_glow,
            &self.flare_dark_spark,
            &self.flare_force_glow,
            &self.flare_force_spark,
            &self.flare_transmutation_glow,
            &self.flare_transmutation_spark,
            // Floating motes
            &self.sleep_mote,
            &self.healing_mote,
            &self.buff_mote,
            &self.nature_mote,
            // Smoke poofs
            &self.polymorph_poof,
            &self.banishment_poof,
        ]
    }
}

/// Stores original material colors so they can be restored when switching wizard types.
#[derive(Resource)]
pub struct OriginalSpellColors {
    entries: HashMap<AssetId<StandardMaterial>, (Color, bevy::color::LinearRgba)>,
    /// Whether materials are currently overridden (e.g. brown for Excremage).
    currently_overridden: bool,
}

/// Captures original material colors at startup for later restoration.
pub fn capture_original_spell_colors(
    assets: Res<SpellVisualAssets>,
    materials: Res<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let mut entries = HashMap::new();
    for handle in assets.all_material_handles() {
        if let Some(mat) = materials.get(handle) {
            entries.insert(handle.id(), (mat.base_color, mat.emissive));
        }
    }
    commands.insert_resource(OriginalSpellColors {
        entries,
        currently_overridden: false,
    });
}

/// Recolors all spell materials to brown for Excremage, or restores originals.
pub fn refresh_spell_visuals_for_wizard(
    assets: Res<SpellVisualAssets>,
    mut originals: ResMut<OriginalSpellColors>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fire_explosion_sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    config: Res<GameConfig>,
) {
    let is_excremage = config.wizard_type == WizardType::Excremage;

    // Skip if materials are already in the correct state
    if is_excremage == originals.currently_overridden {
        return;
    }
    originals.currently_overridden = is_excremage;

    for handle in assets.all_material_handles() {
        let Some((orig_color, orig_emissive)) = originals.entries.get(&handle.id()) else {
            continue;
        };
        let Some(mat) = materials.get_mut(handle) else {
            continue;
        };

        if is_excremage {
            mat.base_color = excremage_color(*orig_color);
            mat.emissive = excremage_emissive(*orig_emissive);
        } else {
            mat.base_color = *orig_color;
            mat.emissive = *orig_emissive;
        }
    }

    // Handle fire explosion sphere material (Excremage override)
    if let Some(mat) = fire_explosion_sphere_materials.get_mut(&assets.fireball_explosion_sphere) {
        if is_excremage {
            let brown = EXCREMAGE_BROWN.to_linear();
            mat.inner_color =
                LinearRgba::new(brown.red * 3.0, brown.green * 3.0, brown.blue * 1.5, 1.0);
            mat.outer_color =
                LinearRgba::new(brown.red * 2.0, brown.green * 0.5, brown.blue * 0.1, 1.0);
        } else {
            mat.inner_color = FIRE_EXPLOSION_INNER_COLOR;
            mat.outer_color = FIRE_EXPLOSION_OUTER_COLOR;
        }
    }
}

/// Converts a color to a brown Excremage variant, preserving alpha.
/// Creates an AuraSphereMaterial with default opacity and time.
fn aura_mat(
    materials: &mut Assets<AuraSphereMaterial>,
    inner: LinearRgba,
    outer: LinearRgba,
) -> Handle<AuraSphereMaterial> {
    materials.add(AuraSphereMaterial {
        inner_color: inner,
        outer_color: outer,
        opacity: 1.0,
        time: 0.0,
    })
}

fn excremage_color(original: Color) -> Color {
    let alpha = original.to_srgba().alpha;
    let brown = EXCREMAGE_BROWN.to_srgba();
    Color::srgba(brown.red, brown.green, brown.blue, alpha)
}

/// Converts emissive to a brown glow, scaling intensity from the original.
/// Derives ratios from `EXCREMAGE_BROWN` so the tint stays consistent.
fn excremage_emissive(original: bevy::color::LinearRgba) -> bevy::color::LinearRgba {
    let brown = EXCREMAGE_BROWN.to_linear();
    let intensity = (original.red + original.green + original.blue) / 3.0;
    // Normalize brown channels so the brightest channel maps to 1.0,
    // then scale by original intensity to preserve brightness.
    let max_c = brown.red.max(brown.green).max(brown.blue).max(0.001);
    bevy::color::LinearRgba::new(
        intensity * brown.red / max_c,
        intensity * brown.green / max_c,
        intensity * brown.blue / max_c,
        original.alpha,
    )
}

/// Builds a cross-plane sphere mesh: 3 intersecting circles (XY, XZ, YZ planes).
///
/// Each circle is approximated as a polygon with 16 segments, double-sided.
/// Total: 3 planes × 16 segments × 2 sides = 192 triangles.
fn build_cross_plane_sphere(radius: f32) -> Mesh {
    let segments = 16_u16;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    // Generate a filled circle fan on each plane
    let planes: [(
        fn(f32, f32, f32) -> [f32; 3], // vertex position from (cos, sin, radius)
        [f32; 3],                      // normal
    ); 3] = [
        (|c, s, r| [c * r, s * r, 0.0], [0.0, 0.0, 1.0]), // XY plane
        (|c, s, r| [c * r, 0.0, s * r], [0.0, 1.0, 0.0]), // XZ plane
        (|c, s, r| [0.0, c * r, s * r], [1.0, 0.0, 0.0]), // YZ plane
    ];

    for (pos_fn, normal) in &planes {
        let base = positions.len() as u16;

        // Center vertex
        let center_pos = pos_fn(0.0, 0.0, 0.0);
        positions.push(center_pos);
        normals.push(*normal);
        uvs.push([0.5, 0.5]);

        // Rim vertices
        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let c = angle.cos();
            let s = angle.sin();
            positions.push(pos_fn(c, s, radius));
            normals.push(*normal);
            uvs.push([0.5 + c * 0.5, 0.5 + s * 0.5]);
        }

        // Front-face triangles (fan from center)
        for i in 0..segments {
            let next = (i + 1) % segments;
            indices.push(base); // center
            indices.push(base + 1 + i); // current rim
            indices.push(base + 1 + next); // next rim
        }

        // Back-face triangles (reversed winding)
        for i in 0..segments {
            let next = (i + 1) % segments;
            indices.push(base);
            indices.push(base + 1 + next);
            indices.push(base + 1 + i);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U16(indices));
    mesh
}

/// Builds a cross-plane triangle mesh: 2 perpendicular triangles that taper from a point
/// at Y=0 to full `base_radius` width at Y=`height`. Used for disintegrate beam (narrow at
/// origin, wide at tip). Double-sided rendering (8 triangles total).
fn build_cross_plane_triangle(base_radius: f32, height: f32) -> Mesh {
    let r = base_radius;
    let h = height;

    // XY plane: point at origin, widens along X at Y=height
    // ZY plane: point at origin, widens along Z at Y=height
    // Each triangle is doubled for back-face rendering
    let positions: Vec<[f32; 3]> = vec![
        // XY plane front (3 verts)
        [0.0, 0.0, 0.0],
        [-r, h, 0.0],
        [r, h, 0.0],
        // XY plane back (3 verts, reversed winding)
        [0.0, 0.0, 0.0],
        [r, h, 0.0],
        [-r, h, 0.0],
        // ZY plane front (3 verts)
        [0.0, 0.0, 0.0],
        [0.0, h, -r],
        [0.0, h, r],
        // ZY plane back (3 verts, reversed winding)
        [0.0, 0.0, 0.0],
        [0.0, h, r],
        [0.0, h, -r],
    ];

    let normals: Vec<[f32; 3]> = vec![
        // XY plane front
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        // XY plane back
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        // ZY plane front
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        // ZY plane back
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
    ];

    let uvs: Vec<[f32; 2]> = vec![
        [0.5, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [0.5, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [0.5, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [0.5, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ];

    let indices: Vec<u16> = vec![
        0, 1, 2, // XY front
        3, 4, 5, // XY back
        6, 7, 8, // ZY front
        9, 10, 11, // ZY back
    ];

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U16(indices));
    mesh
}

/// Builds a cross-plane cylinder mesh: 2 intersecting rectangular quads along the Y axis.
///
/// The quads are in the XY and ZY planes, each `width = radius * 2` and `height`.
/// Double-sided rendering (8 triangles total).
fn build_cross_plane_cylinder(radius: f32, height: f32) -> Mesh {
    let h = height / 2.0; // half height
    let r = radius;

    // Plane 1: XY plane (extends along X)
    // Plane 2: ZY plane (extends along Z)
    let positions: Vec<[f32; 3]> = vec![
        // XY plane (4 verts)
        [-r, -h, 0.0],
        [r, -h, 0.0],
        [r, h, 0.0],
        [-r, h, 0.0],
        // ZY plane (4 verts)
        [0.0, -h, -r],
        [0.0, -h, r],
        [0.0, h, r],
        [0.0, h, -r],
    ];

    let normals: Vec<[f32; 3]> = vec![
        // XY plane normal
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        // ZY plane normal
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
    ];

    let uvs: Vec<[f32; 2]> = vec![
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
    ];

    // Front + back faces for each plane
    let indices: Vec<u16> = vec![
        // XY plane front
        0, 1, 2, 0, 2, 3, // XY plane back
        0, 2, 1, 0, 3, 2, // ZY plane front
        4, 5, 6, 4, 6, 7, // ZY plane back
        4, 6, 5, 4, 7, 6,
    ];

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U16(indices));
    mesh
}
