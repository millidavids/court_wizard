use bevy::prelude::*;

use crate::game::constants::KING_CORPSE_COLOR;

use super::constants::*;

/// Pre-loaded meshes and materials for the king unit.
#[derive(Resource)]
pub struct KingAssets {
    /// Circle mesh (used for corpses).
    pub mesh: Handle<Mesh>,
    /// Rectangle mesh for sprite rendering.
    pub sprite_mesh: Handle<Mesh>,
    /// Directional sprite textures: [forward, back, left, right].
    /// Reuses infantry textures.
    pub sprite_textures: [Handle<Image>; 4],
    pub corpse_material: Handle<StandardMaterial>,
}

/// System to pre-load king assets at startup.
pub(super) fn preload_king_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // King reuses infantry sprite textures
    let sprite_textures = [
        asset_server.load("images/sprite_sheets/infantry-walking-forward_64x64_4-frames.png"),
        asset_server.load("images/sprite_sheets/infantry-walking-back_64x64_4-frames.png"),
        asset_server.load("images/sprite_sheets/infantry-walking-left_64x64_4-frames.png"),
        asset_server.load("images/sprite_sheets/infantry-walking-right_64x64_4-frames.png"),
    ];

    let assets = KingAssets {
        mesh: meshes.add(Circle::new(KING_RADIUS)),
        sprite_mesh: meshes.add(Rectangle::new(KING_SPRITE_WIDTH, KING_SPRITE_HEIGHT)),
        sprite_textures,
        corpse_material: materials.add(StandardMaterial {
            base_color: KING_CORPSE_COLOR,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
