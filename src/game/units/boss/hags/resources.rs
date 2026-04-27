use bevy::prelude::*;

use super::constants::*;
use crate::game::units::boss::utils::EYE_FRAME_UV;
use crate::game::units::systems::create_sprite_material;

/// Resource that controls when eyes transfer between hags.
#[derive(Resource)]
pub struct EyeTransferTimer {
    pub time_remaining: f32,
}

/// Resource tracking permanently dead hags.
#[derive(Resource)]
pub struct HagDeathTracker {
    pub permanent_deaths: u32,
}

impl HagDeathTracker {
    pub fn new() -> Self {
        Self {
            permanent_deaths: 0,
        }
    }
}

/// Pre-loaded meshes and materials for the hag bosses.
#[derive(Resource)]
pub struct HagAssets {
    /// Quad mesh for the hag walking sprite.
    pub sprite_mesh: Handle<Mesh>,
    /// Walking sprite sheet texture (shared across all hags).
    pub walking_texture: Handle<Image>,
    /// Attack sprite sheet texture (used by Josephina's melee animation).
    pub attacking_texture: Handle<Image>,
    /// Casting sprite sheet texture (used when any hag casts a spell).
    pub casting_texture: Handle<Image>,
    pub justina_material: Handle<StandardMaterial>,
    pub martina_material: Handle<StandardMaterial>,
    pub josephina_material: Handle<StandardMaterial>,
    /// Quad mesh for eye sprites.
    pub eye_sprite_mesh: Handle<Mesh>,
    pub invulnerability_eye_material: Handle<StandardMaterial>,
    pub ability_eye_material: Handle<StandardMaterial>,
    pub mind_control_aura_mesh: Handle<Mesh>,
    pub mind_control_aura_material: Handle<StandardMaterial>,
}

/// System to pre-load hag assets at startup.
pub(super) fn preload_hag_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let walking_texture = asset_server.load("images/sprite_sheets/hag-walking_4-frames.png");
    let attacking_texture = asset_server.load("images/sprite_sheets/hag-attacking_4-frames.png");
    let casting_texture = asset_server.load("images/sprite_sheets/hag-casting_4-frames.png");
    let eye_texture = asset_server.load("images/sprite_sheets/eye-pulsing_4-frames.png");

    let eye_frame_uv = EYE_FRAME_UV;

    let assets = HagAssets {
        sprite_mesh: meshes.add(Rectangle::new(HAG_SPRITE_WIDTH, HAG_SPRITE_HEIGHT)),
        walking_texture: walking_texture.clone(),
        attacking_texture,
        casting_texture,
        justina_material: create_sprite_material(
            &mut materials,
            walking_texture.clone(),
            JUSTINA_COLOR,
            HAG_FRAME_UV,
            Vec2::ZERO,
        ),
        martina_material: create_sprite_material(
            &mut materials,
            walking_texture.clone(),
            MARTINA_COLOR,
            HAG_FRAME_UV,
            Vec2::ZERO,
        ),
        josephina_material: create_sprite_material(
            &mut materials,
            walking_texture,
            JOSEPHINA_COLOR,
            HAG_FRAME_UV,
            Vec2::ZERO,
        ),
        eye_sprite_mesh: meshes.add(Rectangle::new(EYE_SPRITE_SIZE, EYE_SPRITE_SIZE)),
        invulnerability_eye_material: create_sprite_material(
            &mut materials,
            eye_texture.clone(),
            INVULNERABILITY_EYE_COLOR,
            eye_frame_uv,
            Vec2::ZERO,
        ),
        ability_eye_material: create_sprite_material(
            &mut materials,
            eye_texture,
            ABILITY_EYE_COLOR,
            eye_frame_uv,
            Vec2::ZERO,
        ),
        mind_control_aura_mesh: meshes.add(Circle::new(MIND_CONTROL_AURA_RADIUS)),
        mind_control_aura_material: materials.add(StandardMaterial {
            base_color: MIND_CONTROL_AURA_COLOR,
            unlit: true,
            alpha_mode: bevy::prelude::AlphaMode::Blend,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
