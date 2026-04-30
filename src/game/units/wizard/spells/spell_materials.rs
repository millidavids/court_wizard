//! Spell material types and helpers (FireExplosionSphereMaterial, AuraSphereMaterial).

use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

/// Default bright yellow center color for fire explosion material.
pub(super) const FIRE_EXPLOSION_INNER_COLOR: LinearRgba = LinearRgba::new(4.0, 2.5, 0.4, 1.0);
/// Default deep orange-red edge color for fire explosion material.
pub(super) const FIRE_EXPLOSION_OUTER_COLOR: LinearRgba = LinearRgba::new(2.5, 0.4, 0.0, 1.0);
/// Bright white center color for ice explosion material.
pub(super) const ICE_EXPLOSION_INNER_COLOR: LinearRgba = LinearRgba::new(3.0, 3.5, 4.0, 1.0);
/// Cool blue edge color for ice explosion material.
pub(super) const ICE_EXPLOSION_OUTER_COLOR: LinearRgba = LinearRgba::new(0.3, 0.6, 2.5, 1.0);

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
pub(super) const KING_AURA_INNER: LinearRgba = LinearRgba::new(2.0, 1.8, 0.6, 1.0);
pub(super) const KING_AURA_OUTER: LinearRgba = LinearRgba::new(0.4, 0.5, 1.5, 1.0);
pub(super) const HEALING_AURA_INNER: LinearRgba = LinearRgba::new(0.3, 2.0, 0.5, 1.0);
pub(super) const HEALING_AURA_OUTER: LinearRgba = LinearRgba::new(0.1, 1.0, 0.3, 1.0);
pub(super) const GUARDIAN_AURA_INNER: LinearRgba = LinearRgba::new(0.5, 2.5, 3.0, 1.0);
pub(super) const GUARDIAN_AURA_OUTER: LinearRgba = LinearRgba::new(0.8, 1.5, 2.5, 1.0);
pub(super) const BATTLE_HYMN_AURA_INNER: LinearRgba = LinearRgba::new(2.5, 2.0, 0.4, 1.0);
pub(super) const BATTLE_HYMN_AURA_OUTER: LinearRgba = LinearRgba::new(2.0, 1.2, 0.2, 1.0);
pub(super) const HASTE_AURA_INNER: LinearRgba = LinearRgba::new(1.5, 2.5, 0.5, 1.0);
pub(super) const HASTE_AURA_OUTER: LinearRgba = LinearRgba::new(2.0, 2.0, 0.3, 1.0);
pub(super) const BERSERKER_AURA_INNER: LinearRgba = LinearRgba::new(2.5, 0.3, 0.2, 1.0);
pub(super) const BERSERKER_AURA_OUTER: LinearRgba = LinearRgba::new(2.0, 0.8, 0.1, 1.0);
pub(super) const SLEEP_AURA_INNER: LinearRgba = LinearRgba::new(2.5, 2.5, 3.0, 1.0);
pub(super) const SLEEP_AURA_OUTER: LinearRgba = LinearRgba::new(1.5, 1.5, 2.0, 1.0);
pub(super) const RAISE_DEAD_AURA_INNER: LinearRgba = LinearRgba::new(1.0, 0.3, 2.5, 1.0);
pub(super) const RAISE_DEAD_AURA_OUTER: LinearRgba = LinearRgba::new(0.6, 0.2, 1.5, 1.0);
pub(super) const COMMANDER_AURA_INNER: LinearRgba = LinearRgba::new(2.0, 1.5, 0.4, 1.0);
pub(super) const COMMANDER_AURA_OUTER: LinearRgba = LinearRgba::new(2.0, 1.8, 0.6, 1.0);
pub(super) const CRYSTAL_AURA_INNER: LinearRgba = LinearRgba::new(2.5, 0.8, 2.0, 1.0);
pub(super) const CRYSTAL_AURA_OUTER: LinearRgba = LinearRgba::new(1.5, 0.3, 1.8, 1.0);
pub(super) const TELEPORT_AURA_INNER: LinearRgba = LinearRgba::new(0.5, 1.5, 3.0, 1.0);
pub(super) const TELEPORT_AURA_OUTER: LinearRgba = LinearRgba::new(0.3, 0.8, 2.5, 1.0);

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
