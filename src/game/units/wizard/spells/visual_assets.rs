//! Shared spell visual assets.
//!
//! Pre-allocates all meshes and materials used by spell effects.
//! Both local spell spawning and ghost/remote rendering use these handles,
//! ensuring a single source of truth for spell visuals.

use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;

/// Pre-allocated meshes and materials for all spell visuals.
///
/// Initialized once at startup. Local spell spawn functions and multiplayer
/// ghost rendering both clone handles from this resource instead of creating
/// their own meshes/materials.
#[derive(Resource)]
pub struct SpellVisualAssets {
    // ── Base meshes (unit size, scaled by Transform) ──────────────────────
    pub unit_circle: Handle<Mesh>,
    pub unit_sphere: Handle<Mesh>,
    pub unit_cylinder: Handle<Mesh>,
    pub unit_cuboid: Handle<Mesh>,
    pub unit_rect: Handle<Mesh>,

    // ── Zone materials (semi-transparent ground circles) ──────────────────
    pub spike_growth_zone: Handle<StandardMaterial>,
    pub healing_plume_zone: Handle<StandardMaterial>,
    pub entangle_zone: Handle<StandardMaterial>,
    pub fog_cloud_zone: Handle<StandardMaterial>,
    pub grease_zone: Handle<StandardMaterial>,
    pub grease_fire: Handle<StandardMaterial>,
    pub plague_wind_zone: Handle<StandardMaterial>,
    pub meteor_ground_fire: Handle<StandardMaterial>,

    // ── Casting indicator materials (translucent circles shown while aiming) ──
    pub haste_indicator: Handle<StandardMaterial>,
    pub battle_hymn_indicator: Handle<StandardMaterial>,
    pub berserker_rage_indicator: Handle<StandardMaterial>,
    pub sleep_indicator: Handle<StandardMaterial>,
    pub guardian_circle_indicator: Handle<StandardMaterial>,
    pub hypnotic_pattern_indicator: Handle<StandardMaterial>,
    pub phantasmal_force_indicator: Handle<StandardMaterial>,
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
    pub teleport_destination: Handle<StandardMaterial>,
    pub teleport_source: Handle<StandardMaterial>,
    pub telekinesis_indicator: Handle<StandardMaterial>,

    // ── Object materials ─────────────────────────────────────────────────
    pub black_hole: Handle<StandardMaterial>,
    pub arcane_crystal: Handle<StandardMaterial>,
    pub lightning_rod: Handle<StandardMaterial>,

    // ── Wall materials ───────────────────────────────────────────────────
    pub wall_of_stone: Handle<StandardMaterial>,
    pub wall_of_fire: Handle<StandardMaterial>,

    // ── Explosion materials ──────────────────────────────────────────────
    pub fireball_explosion: Handle<StandardMaterial>,
    pub meteor_explosion: Handle<StandardMaterial>,
    pub ice_explosion: Handle<StandardMaterial>,

    // ── Projectile materials ─────────────────────────────────────────────
    pub fireball_projectile: Handle<StandardMaterial>,
    pub ice_projectile: Handle<StandardMaterial>,
    pub meteor_projectile: Handle<StandardMaterial>,
    pub magic_missile: Handle<StandardMaterial>,

    // ── Arc/Beam materials ───────────────────────────────────────────────
    pub chain_lightning_arc: Handle<StandardMaterial>,
    pub lightning_strike: Handle<StandardMaterial>,
    pub lightning_rod_arc: Handle<StandardMaterial>,
    pub crystal_beam: Handle<StandardMaterial>,
    pub crystal_arc: Handle<StandardMaterial>,
    pub finger_of_death_beam: Handle<StandardMaterial>,
    pub disintegrate_beam: Handle<StandardMaterial>,

    // ── Crystal mini-spell materials ──────────────────────────────────────
    pub crystal_mini_missile: Handle<StandardMaterial>,
    pub crystal_range_indicator: Handle<StandardMaterial>,

    // ── Special meshes (fixed-size) ──────────────────────────────────────
    pub magic_missile_mesh: Handle<Mesh>,
}

/// Initializes the shared spell visual assets resource.
pub fn init_spell_visual_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    commands.insert_resource(SpellVisualAssets {
        // Base meshes
        unit_circle: meshes.add(Circle::new(1.0)),
        unit_sphere: meshes.add(Sphere::new(1.0)),
        unit_cylinder: meshes.add(Cylinder::new(0.5, 1.0)),
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

        // Casting indicator materials (translucent circles shown while aiming)
        haste_indicator: materials.add(unlit_blend(Color::srgba(1.0, 0.85, 0.0, 0.3))),
        battle_hymn_indicator: materials.add(unlit_blend(Color::srgba(1.0, 0.85, 0.0, 0.3))),
        berserker_rage_indicator: materials.add(unlit_blend(Color::srgba(0.9, 0.15, 0.1, 0.3))),
        sleep_indicator: materials.add(unlit_blend(Color::srgba(0.4, 0.3, 0.7, 0.3))),
        guardian_circle_indicator: materials.add(unlit_blend(Color::srgba(0.0, 0.8, 1.0, 0.3))),
        hypnotic_pattern_indicator: materials.add(unlit_blend(Color::srgba(0.6, 0.3, 0.8, 0.3))),
        phantasmal_force_indicator: materials.add(unlit_blend(Color::srgba(0.5, 0.5, 0.8, 0.3))),
        spike_growth_indicator: materials.add(unlit_blend(Color::srgba(0.2, 0.45, 0.1, 0.3))),
        healing_plume_indicator: materials.add(unlit_blend(Color::srgba(0.2, 0.8, 0.3, 0.3))),
        entangle_indicator: materials.add(unlit_blend(Color::srgba(0.1, 0.7, 0.2, 0.3))),
        fog_cloud_indicator: materials.add(unlit_blend(Color::srgba(0.7, 0.75, 0.8, 0.3))),
        grease_indicator: materials.add(unlit_blend(Color::srgba(0.5, 0.45, 0.1, 0.3))),
        plague_wind_indicator: materials.add(unlit_blend(Color::srgba(0.3, 0.8, 0.1, 0.3))),
        arcane_crystal_indicator: materials.add(unlit_blend(Color::srgba(0.5, 0.2, 0.8, 0.3))),
        lightning_rod_indicator: materials.add(unlit_blend(Color::srgba(0.7, 0.85, 1.0, 0.4))),
        meteor_fall_indicator: materials.add(unlit_blend(Color::srgba(0.9, 0.3, 0.1, 0.25))),
        squall_indicator: materials.add(unlit_blend(Color::srgba(0.3, 0.8, 1.0, 0.4))),
        teleport_destination: materials.add(unlit_blend(Color::srgba(0.0, 0.6, 1.0, 0.25))),
        teleport_source: materials.add(unlit_blend(Color::srgba(0.0, 0.8, 1.0, 0.35))),
        telekinesis_indicator: materials.add(unlit_blend(Color::srgba(0.6, 0.9, 1.0, 0.7))),

        // Object materials
        black_hole: materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.0, 0.1),
            emissive: bevy::color::LinearRgba::new(0.2, 0.0, 0.4, 1.0),
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
        wall_of_stone: materials.add(StandardMaterial {
            base_color: Color::srgba(0.75, 0.6, 0.45, 1.0),
            ..default()
        }),
        wall_of_fire: materials.add(unlit_blend(Color::srgba(1.0, 0.5, 0.0, 0.4))),

        // Explosion materials
        fireball_explosion: materials.add(unlit(Color::srgb(1.0, 0.3, 0.0))),
        meteor_explosion: materials.add(unlit_blend(Color::srgba(1.0, 0.5, 0.1, 0.6))),
        ice_explosion: materials.add(unlit(Color::srgb(0.3, 0.8, 1.0))),

        // Projectile materials
        fireball_projectile: materials.add(unlit(Color::srgb(1.0, 0.5, 0.0))),
        ice_projectile: materials.add(unlit(Color::srgb(0.7, 0.9, 1.0))),
        meteor_projectile: materials.add(unlit(Color::srgb(1.0, 0.4, 0.1))),
        magic_missile: materials.add(unlit(Color::srgb(1.0, 0.4, 0.8))),

        // Arc/Beam materials
        chain_lightning_arc: materials.add(unlit(Color::srgb(0.7, 0.85, 1.0))),
        lightning_strike: materials.add(unlit(Color::srgb(0.8, 0.9, 1.0))),
        lightning_rod_arc: materials.add(unlit(Color::srgb(0.7, 0.85, 1.0))),
        crystal_beam: materials.add(unlit(Color::srgb(1.0, 0.6, 0.1))),
        crystal_arc: materials.add(unlit(Color::srgba(0.6, 0.4, 1.0, 0.9))),
        finger_of_death_beam: materials.add(StandardMaterial {
            base_color: Color::srgba(0.6, 0.0, 0.8, 0.0),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        disintegrate_beam: materials.add(unlit(Color::srgb(1.0, 0.6, 0.1))),

        // Crystal mini-spell materials
        crystal_mini_missile: materials.add(unlit(Color::srgb(0.8, 0.3, 0.9))),
        crystal_range_indicator: materials.add(unlit_blend(Color::srgba(0.5, 0.2, 0.8, 0.15))),

        // Special meshes (magic missile radius = 5.0, from magic_missile/styles.rs)
        magic_missile_mesh: meshes.add(Sphere::new(5.0)),
    });
}
