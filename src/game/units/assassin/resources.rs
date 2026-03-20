use bevy::prelude::*;

use crate::game::constants::{ATTACKER_CORPSE_COLOR, UNDEAD_CORPSE_COLOR};
use crate::game::units::components::CORPSE_MATERIAL_VARIANTS;
use crate::game::units::systems::create_corpse_sprite_materials;


/// Pre-loaded meshes and materials for assassin units.
/// Reuses the archer sprite sheet but with a translucent material.
#[derive(Resource)]
#[allow(dead_code)]
pub struct AssassinAssets {
    /// Rectangle mesh for sprite rendering (same size as archer).
    pub sprite_mesh: Handle<Mesh>,
    /// Archer sprite sheet texture (shared).
    pub sprite_texture: Handle<Image>,
    /// Attacker corpse materials.
    pub attacker_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    /// Undead corpse materials.
    pub undead_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
}

/// System to pre-load assassin assets at startup.
pub(super) fn preload_assassin_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Reuse the archer sprite sheet
    let sprite_texture = asset_server.load("images/sprite_sheets/archer-walking_8-frames.png");

    let attacker_corpse_materials = create_corpse_sprite_materials(
        &mut materials,
        sprite_texture.clone(),
        ATTACKER_CORPSE_COLOR,
    );
    let undead_corpse_materials =
        create_corpse_sprite_materials(&mut materials, sprite_texture.clone(), UNDEAD_CORPSE_COLOR);

    // Use archer sprite dimensions
    let sprite_width = crate::game::units::archer::styles::ARCHER_SPRITE_WIDTH;
    let sprite_height = crate::game::units::archer::styles::ARCHER_SPRITE_HEIGHT;

    let assets = AssassinAssets {
        sprite_mesh: meshes.add(Rectangle::new(sprite_width, sprite_height)),
        sprite_texture,
        attacker_corpse_materials,
        undead_corpse_materials,
    };

    commands.insert_resource(assets);
}
