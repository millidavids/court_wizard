use bevy::prelude::*;

use crate::game::constants::UNDEAD_CORPSE_COLOR;
use crate::game::units::components::CORPSE_MATERIAL_VARIANTS;
use crate::game::units::constants::{DEFAULT_SPRITE_HEIGHT, DEFAULT_SPRITE_WIDTH};
use crate::game::units::systems::create_corpse_sprite_materials;

/// Pre-loaded meshes and materials for undead units.
#[derive(Resource)]
pub struct UndeadAssets {
    /// Rectangle mesh for sprite rendering (same size as infantry).
    pub sprite_mesh: Handle<Mesh>,
    /// Undead walking sprite sheet texture.
    pub sprite_texture: Handle<Image>,
    /// Melee attack animation sprite sheet.
    pub attacking_texture: Handle<Image>,
    /// Death animation sprite sheet.
    pub death_texture: Handle<Image>,
    /// Undead corpse materials.
    #[allow(dead_code)]
    pub corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
}

/// System to pre-load undead assets at startup.
pub(in crate::game) fn preload_undead_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let sprite_texture = asset_server.load("images/sprite_sheets/undead-walking_9-frames.png");
    let attacking_texture = asset_server.load("images/sprite_sheets/undead-attacking_6-frames.png");
    let death_texture = asset_server.load("images/sprite_sheets/undead-death_6-frames.png");

    let corpse_materials =
        create_corpse_sprite_materials(&mut materials, sprite_texture.clone(), UNDEAD_CORPSE_COLOR);

    let assets = UndeadAssets {
        sprite_mesh: meshes.add(Rectangle::new(DEFAULT_SPRITE_WIDTH, DEFAULT_SPRITE_HEIGHT)),
        sprite_texture,
        attacking_texture,
        death_texture,
        corpse_materials,
    };

    commands.insert_resource(assets);
}
