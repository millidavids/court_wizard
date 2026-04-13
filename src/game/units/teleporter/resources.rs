use bevy::prelude::*;

use crate::game::constants::ATTACKER_CORPSE_COLOR;
use crate::game::units::components::CORPSE_MATERIAL_VARIANTS;
use crate::game::units::constants::{DEFAULT_SPRITE_HEIGHT, DEFAULT_SPRITE_WIDTH};
use crate::game::units::systems::create_corpse_sprite_materials;

#[derive(Resource)]
#[allow(dead_code)]
pub(in crate::game) struct TeleporterAssets {
    pub sprite_mesh: Handle<Mesh>,
    pub sprite_texture: Handle<Image>,
    pub casting_texture: Handle<Image>,
    pub attacker_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
}

pub(super) fn preload_teleporter_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let sprite_texture = asset_server.load("images/sprite_sheets/teleporter-walking_9-frames.png");
    let casting_texture = asset_server.load("images/sprite_sheets/teleporter-casting_7-frames.png");

    let attacker_corpse_materials = create_corpse_sprite_materials(
        &mut materials,
        sprite_texture.clone(),
        ATTACKER_CORPSE_COLOR,
    );

    commands.insert_resource(TeleporterAssets {
        sprite_mesh: meshes.add(Rectangle::new(DEFAULT_SPRITE_WIDTH, DEFAULT_SPRITE_HEIGHT)),
        sprite_texture,
        casting_texture,
        attacker_corpse_materials,
    });
}
