//! Shared spell visual assets.
//!
//! Pre-allocates all meshes and materials used by spell effects.
//! Both local spell spawning and ghost/remote rendering use these handles,
//! ensuring a single source of truth for spell visuals.

use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};

use super::black_hole::constants::TORUS_MINOR_RADIUS;

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
    /// 2-plane cross cylinder (2 quads along Y axis, double-sided). Low-poly cylinder for beams.
    pub cross_plane_cylinder: Handle<Mesh>,
    /// Low-resolution torus (unit-scale) for black hole rings.
    pub black_hole_torus: Handle<Mesh>,

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
    pub disintegrate_glow: Handle<StandardMaterial>,
    pub disintegrate_flare: Handle<StandardMaterial>,
    pub disintegrate_particle: Handle<StandardMaterial>,

    // ── Fire VFX materials (shared by fireball, meteor, etc.) ─────────
    pub fire_glow: Handle<StandardMaterial>,
    pub fire_spark: Handle<StandardMaterial>,
    pub fire_smoke: Handle<StandardMaterial>,

    // ── Disintegrate smoke material ─────────────────────────────────────
    pub disintegrate_smoke: Handle<StandardMaterial>,

    // ── Finger of Death VFX materials ────────────────────────────────────
    pub necrotic_vein: Handle<StandardMaterial>,
    pub finger_of_death_glow: Handle<StandardMaterial>,
    pub necrotic_pulse: Handle<StandardMaterial>,

    // ── Crystal mini-spell materials ──────────────────────────────────────
    pub crystal_mini_missile: Handle<StandardMaterial>,
    pub crystal_range_indicator: Handle<StandardMaterial>,

    // ── Heat shimmer material (fire haze) ─────────────────────────────
    pub heat_shimmer: Handle<StandardMaterial>,

    // ── Magic missile VFX materials ────────────────────────────────────
    pub missile_glow: Handle<StandardMaterial>,
    pub missile_sparkle: Handle<StandardMaterial>,

    // ── Special meshes (fixed-size) ──────────────────────────────────────
    pub magic_missile_mesh: Handle<Mesh>,
    /// Flat quad mesh for particles (2 tris, double-sided).
    pub particle_quad: Handle<Mesh>,
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
        black_hole_billboard: materials.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            cull_mode: None,
            ..default()
        }),
        black_hole_ring: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.85, 0.8, 0.9),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(2.5, 1.5, 1.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        black_hole_accretion: materials.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            cull_mode: None,
            ..default()
        }),
        black_hole_accretion_ring: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.85, 0.8, 0.9),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(2.5, 1.5, 1.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
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
        wall_of_fire: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.5, 0.0, 0.4),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(2.0, 0.8, 0.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),

        // Explosion materials
        fireball_explosion: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.3, 0.0),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(3.0, 1.0, 0.0, 1.0),
            ..default()
        }),
        meteor_explosion: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.5, 0.1, 0.6),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(2.5, 1.0, 0.1, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        }),
        ice_explosion: materials.add(unlit(Color::srgb(0.3, 0.8, 1.0))),

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
            ..default()
        }),
        fire_smoke: materials.add(StandardMaterial {
            base_color: Color::srgba(0.05, 0.05, 0.05, 0.4),
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

        // Disintegrate smoke
        disintegrate_smoke: materials.add(StandardMaterial {
            base_color: Color::srgba(0.05, 0.05, 0.05, 0.4),
            unlit: true,
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
            base_color: Color::srgba(0.2, 0.0, 0.3, 0.15),
            unlit: true,
            emissive: bevy::color::LinearRgba::new(0.5, 0.0, 0.8, 1.0),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
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

        // Crystal mini-spell materials
        crystal_mini_missile: materials.add(unlit(Color::srgb(0.8, 0.3, 0.9))),
        crystal_range_indicator: materials.add(unlit_blend(Color::srgba(0.5, 0.2, 0.8, 0.15))),

        // Cross-plane sphere: 3 intersecting unit circles (XY, XZ, YZ), radius 1.0
        cross_plane_sphere: meshes.add(build_cross_plane_sphere(1.0)),
        // Cross-plane cylinder: 2 intersecting quads along Y axis, radius 0.5, height 1.0
        cross_plane_cylinder: meshes.add(build_cross_plane_cylinder(0.5, 1.0)),
        // Low-poly torus for black hole rings (unit-scale, scaled by Transform)
        black_hole_torus: meshes.add(
            Torus::new(1.0 - TORUS_MINOR_RADIUS, 1.0 + TORUS_MINOR_RADIUS)
                .mesh()
                .major_resolution(16)
                .minor_resolution(8),
        ),

        // Special meshes (magic missile radius = 5.0)
        magic_missile_mesh: meshes.add(build_cross_plane_sphere(5.0)),
        // Unit square in XY plane (2 tris, double-sided) for pixel-art particle effects.
        particle_quad: meshes.add({
            let h = 0.5_f32; // half-extent → 1x1 quad, scaled via Transform
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::default());
            // Front face (4 verts) + back face (4 verts)
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![
                [-h, -h, 0.0], [ h, -h, 0.0], [ h,  h, 0.0], [-h,  h, 0.0], // front
                [-h, -h, 0.0], [-h,  h, 0.0], [ h,  h, 0.0], [ h, -h, 0.0], // back
            ]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![
                [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
                [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
            ]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![
                [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
                [0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0],
            ]);
            mesh.insert_indices(Indices::U16(vec![
                0, 1, 2, 0, 2, 3, // front
                4, 5, 6, 4, 6, 7, // back
            ]));
            mesh
        }),
    });
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
        [f32; 3],                        // normal
    ); 3] = [
        (|c, s, r| [c * r, s * r, 0.0], [0.0, 0.0, 1.0]),  // XY plane
        (|c, s, r| [c * r, 0.0, s * r], [0.0, 1.0, 0.0]),  // XZ plane
        (|c, s, r| [0.0, c * r, s * r], [1.0, 0.0, 0.0]),  // YZ plane
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
            indices.push(base);            // center
            indices.push(base + 1 + i);    // current rim
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
        [-r, -h, 0.0], [r, -h, 0.0], [r, h, 0.0], [-r, h, 0.0],
        // ZY plane (4 verts)
        [0.0, -h, -r], [0.0, -h, r], [0.0, h, r], [0.0, h, -r],
    ];

    let normals: Vec<[f32; 3]> = vec![
        // XY plane normal
        [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
        // ZY plane normal
        [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
    ];

    let uvs: Vec<[f32; 2]> = vec![
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
    ];

    // Front + back faces for each plane
    let indices: Vec<u16> = vec![
        // XY plane front
        0, 1, 2, 0, 2, 3,
        // XY plane back
        0, 2, 1, 0, 3, 2,
        // ZY plane front
        4, 5, 6, 4, 6, 7,
        // ZY plane back
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
