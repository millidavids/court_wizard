use bevy::prelude::*;

use crate::game::constants::{
    ATTACKER_CORPSE_COLOR, DEFENDER_CORPSE_COLOR, UNDEAD_CORPSE_COLOR,
};

use super::styles::*;

/// Pre-loaded meshes and materials for infantry units.
#[derive(Resource)]
pub struct InfantryAssets {
    /// Circle mesh (used for corpses and boss mesh swap).
    pub mesh: Handle<Mesh>,
    /// Rectangle mesh for sprite rendering.
    pub sprite_mesh: Handle<Mesh>,
    /// Directional sprite textures: [forward, back, left, right].
    pub sprite_textures: [Handle<Image>; 4],
    pub defender_corpse_material: Handle<StandardMaterial>,
    pub attacker_corpse_material: Handle<StandardMaterial>,
    pub undead_corpse_material: Handle<StandardMaterial>,
}

/// System to pre-load infantry assets at startup.
pub(super) fn preload_infantry_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let sprite_textures = [
        asset_server.load("images/sprite_sheets/infantry-walking-forward_64x64_4-frames.png"),
        asset_server.load("images/sprite_sheets/infantry-walking-back_64x64_4-frames.png"),
        asset_server.load("images/sprite_sheets/infantry-walking-left_64x64_4-frames.png"),
        asset_server.load("images/sprite_sheets/infantry-walking-right_64x64_4-frames.png"),
    ];

    let assets = InfantryAssets {
        mesh: meshes.add(Circle::new(UNIT_RADIUS)),
        sprite_mesh: meshes.add(Rectangle::new(INFANTRY_SPRITE_WIDTH, INFANTRY_SPRITE_HEIGHT)),
        sprite_textures,
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
