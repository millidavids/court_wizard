use bevy::prelude::*;

use crate::game::constants::{
    ATTACKER_CORPSE_COLOR, DEFENDER_CORPSE_COLOR, UNDEAD_CORPSE_COLOR,
};

use super::constants::ARROW_WIDTH;
use super::styles::*;

/// Pre-loaded meshes and materials for archer units.
#[derive(Resource)]
pub struct ArcherAssets {
    /// Circle mesh (used for corpses).
    pub mesh: Handle<Mesh>,
    /// Rectangle mesh for sprite rendering.
    pub sprite_mesh: Handle<Mesh>,
    /// Directional sprite textures: [forward, back, left, right].
    pub sprite_textures: [Handle<Image>; 4],
    pub arrow_mesh: Handle<Mesh>,
    pub arrow_material: Handle<StandardMaterial>,
    pub defender_corpse_material: Handle<StandardMaterial>,
    pub attacker_corpse_material: Handle<StandardMaterial>,
    pub undead_corpse_material: Handle<StandardMaterial>,
}

/// System to pre-load archer assets at startup.
pub(super) fn preload_archer_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let sprite_textures = [
        asset_server.load("images/sprite_sheets/archer-walking-forward_64x64_4-frames.png"),
        asset_server.load("images/sprite_sheets/archer-walking-back_64x64_4-frames.png"),
        asset_server.load("images/sprite_sheets/archer-walking-left_64x64_4-frames.png"),
        asset_server.load("images/sprite_sheets/archer-walking-right_64x64_4-frames.png"),
    ];

    let assets = ArcherAssets {
        mesh: meshes.add(Circle::new(ARCHER_RADIUS)),
        sprite_mesh: meshes.add(Rectangle::new(ARCHER_SPRITE_WIDTH, ARCHER_SPRITE_HEIGHT)),
        sprite_textures,
        arrow_mesh: meshes.add(Circle::new(ARROW_WIDTH)),
        arrow_material: materials.add(StandardMaterial {
            base_color: ARROW_COLOR,
            unlit: true,
            ..default()
        }),
        defender_corpse_material: materials.add(StandardMaterial {
            base_color: DEFENDER_CORPSE_COLOR,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        attacker_corpse_material: materials.add(StandardMaterial {
            base_color: ATTACKER_CORPSE_COLOR,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        undead_corpse_material: materials.add(StandardMaterial {
            base_color: UNDEAD_CORPSE_COLOR,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
