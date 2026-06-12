use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use super::super::button_systems::insert_material_background;
use super::super::constants::CONTENT_BG;

/// Marker for page content panels that should receive a parchment background.
#[derive(Component)]
pub(crate) struct ParchmentPanel;

/// Marker for overlay roots that should receive a frosted glass background.
#[derive(Component)]
pub(crate) struct FrostedGlassOverlay;

/// Procedural parchment/stone texture material for panel backgrounds.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(crate) struct ParchmentMaterial {
    #[uniform(0)]
    pub data: ParchmentData,
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub(crate) struct ParchmentData {
    pub base_color: LinearRgba,
    pub texture_strength: f32,
    pub vignette_strength: f32,
    pub noise_scale: f32,
    pub _padding: f32,
}

impl UiMaterial for ParchmentMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/parchment.wgsl".into()
    }
}

impl ParchmentMaterial {
    pub fn new(base_color: Color) -> Self {
        Self {
            data: ParchmentData {
                base_color: base_color.to_linear(),
                texture_strength: 0.45,
                vignette_strength: 0.4,
                noise_scale: 5.0,
                _padding: 0.0,
            },
        }
    }
}

/// Frosted glass overlay material for menu backgrounds.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(crate) struct FrostedGlassMaterial {
    #[uniform(0)]
    pub data: FrostedGlassData,
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub(crate) struct FrostedGlassData {
    pub tint_color: LinearRgba,
    pub frost_intensity: f32,
    pub noise_scale: f32,
    pub _padding1: f32,
    pub _padding2: f32,
}

impl UiMaterial for FrostedGlassMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/frosted_glass.wgsl".into()
    }
}

impl FrostedGlassMaterial {
    pub fn new() -> Self {
        Self {
            data: FrostedGlassData {
                tint_color: Color::hsla(20.0, 0.04, 0.12, 0.30).to_linear(),
                frost_intensity: 1.0,
                noise_scale: 6.0,
                _padding1: 0.0,
                _padding2: 0.0,
            },
        }
    }
}

impl Default for FrostedGlassMaterial {
    fn default() -> Self {
        Self::new()
    }
}

/// Scales a font size down based on text width to fit within a constrained area.
///
/// Returns `base_font` when `max_width <= min_chars`, scaling linearly down to
/// `base_font * min_scale` when `max_width >= max_chars`.
pub fn apply_parchment_backgrounds(
    mut commands: Commands,
    new_panels: Query<Entity, Added<ParchmentPanel>>,
    mut materials: ResMut<Assets<ParchmentMaterial>>,
) {
    for entity in &new_panels {
        let mat = materials.add(ParchmentMaterial::new(CONTENT_BG));
        insert_material_background(&mut commands, entity, mat);
    }
}

/// Applies a frosted glass material to newly spawned overlay roots.
pub fn apply_frosted_glass_overlays(
    mut commands: Commands,
    new_overlays: Query<Entity, Added<FrostedGlassOverlay>>,
    mut materials: ResMut<Assets<FrostedGlassMaterial>>,
) {
    for entity in &new_overlays {
        let mat = materials.add(FrostedGlassMaterial::new());
        insert_material_background(&mut commands, entity, mat);
    }
}
