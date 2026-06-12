use bevy::prelude::*;

/// Pre-loaded meshes and materials for assassin units.
#[derive(Resource)]
pub struct AssassinAssets {
    /// Rectangle mesh for sprite rendering (same size as archer).
    pub sprite_mesh: Handle<Mesh>,
    /// Assassin walking sprite sheet texture.
    pub sprite_texture: Handle<Image>,
    /// Melee attack animation sprite sheet.
    pub attacking_texture: Handle<Image>,
    /// Death animation sprite sheet.
    pub death_texture: Handle<Image>,
}

/// System to pre-load assassin assets at startup.
pub(super) fn preload_assassin_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let sprite_texture = asset_server.load("images/sprite_sheets/assassin-walking_9-frames.png");
    let attacking_texture =
        asset_server.load("images/sprite_sheets/assassin-attacking_6-frames.png");
    let death_texture = asset_server.load("images/sprite_sheets/assassin-death_6-frames.png");

    // Use archer sprite dimensions
    let sprite_width = crate::game::units::archer::constants::ARCHER_SPRITE_WIDTH;
    let sprite_height = crate::game::units::archer::constants::ARCHER_SPRITE_HEIGHT;

    let assets = AssassinAssets {
        sprite_mesh: meshes.add(Rectangle::new(sprite_width, sprite_height)),
        sprite_texture,
        attacking_texture,
        death_texture,
    };

    commands.insert_resource(assets);
}
