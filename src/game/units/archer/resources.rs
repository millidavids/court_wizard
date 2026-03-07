use bevy::prelude::*;

use crate::game::constants::{ATTACKER_CORPSE_COLOR, DEFENDER_CORPSE_COLOR, UNDEAD_CORPSE_COLOR};
use crate::game::units::components::CORPSE_MATERIAL_VARIANTS;
use crate::game::units::systems::create_corpse_sprite_materials;

use super::constants::{ARROW_LENGTH, ARROW_WIDTH};
use super::styles::*;

/// Pre-loaded meshes and materials for archer units.
#[derive(Resource)]
pub struct ArcherAssets {
    /// Rectangle mesh for sprite rendering.
    pub sprite_mesh: Handle<Mesh>,
    /// Combined sprite sheet texture (all directions in one image).
    pub sprite_texture: Handle<Image>,
    pub arrow_mesh: Handle<Mesh>,
    pub arrow_material: Handle<StandardMaterial>,
    pub defender_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    pub attacker_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    pub undead_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
}

/// System to pre-load archer assets at startup.
pub(super) fn preload_archer_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let sprite_texture = asset_server.load("images/sprite_sheets/archer-walking_8-frames.png");

    let defender_corpse_materials = create_corpse_sprite_materials(
        &mut materials,
        sprite_texture.clone(),
        DEFENDER_CORPSE_COLOR,
    );
    let attacker_corpse_materials = create_corpse_sprite_materials(
        &mut materials,
        sprite_texture.clone(),
        ATTACKER_CORPSE_COLOR,
    );
    let undead_corpse_materials =
        create_corpse_sprite_materials(&mut materials, sprite_texture.clone(), UNDEAD_CORPSE_COLOR);

    let assets = ArcherAssets {
        sprite_mesh: meshes.add(Rectangle::new(ARCHER_SPRITE_WIDTH, ARCHER_SPRITE_HEIGHT)),
        sprite_texture,
        arrow_mesh: meshes.add(Rectangle::new(ARROW_WIDTH, ARROW_LENGTH)),
        arrow_material: materials.add(StandardMaterial {
            base_color: ARROW_COLOR,
            unlit: true,
            ..default()
        }),
        defender_corpse_materials,
        attacker_corpse_materials,
        undead_corpse_materials,
    };

    commands.insert_resource(assets);
}
