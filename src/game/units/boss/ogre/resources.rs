use bevy::prelude::*;

use super::constants::*;

/// Pre-loaded meshes, textures, and materials for the ogre boss.
#[derive(Resource)]
pub struct OgreAssets {
    /// Rectangle mesh for sprite rendering.
    pub sprite_mesh: Handle<Mesh>,
    /// Walking sprite sheet texture.
    pub walking_texture: Handle<Image>,
    /// Attacking sprite sheet texture (used for charge frames).
    pub attacking_texture: Handle<Image>,
    /// Throwing sprite sheet texture (used for rock throw).
    pub throwing_texture: Handle<Image>,
    /// Ogre melee swing sound effect.
    pub swing_sfx: Handle<AudioSource>,
    /// Ogre grunt sound effect (boulder throw).
    pub grunt_sfx: Handle<AudioSource>,
    /// Ogre charge sound effect.
    pub charge_sfx: Handle<AudioSource>,
    pub charge_rect_mesh: Handle<Mesh>,
    pub charge_line_material: Handle<StandardMaterial>,
}

/// System to pre-load ogre assets at startup.
pub(super) fn preload_ogre_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let assets = OgreAssets {
        sprite_mesh: meshes.add(Rectangle::new(OGRE_SPRITE_WIDTH, OGRE_SPRITE_HEIGHT)),
        walking_texture: asset_server
            .load("images/sprite_sheets/ogre-walking_4-frames.png"),
        attacking_texture: asset_server
            .load("images/sprite_sheets/ogre-attacking_4-frames.png"),
        throwing_texture: asset_server
            .load("images/sprite_sheets/ogre-throwing_4-frames.png"),
        swing_sfx: asset_server.load("audio/sound_effects/ogre_swing.ogg"),
        grunt_sfx: asset_server.load("audio/sound_effects/ogre_grunt.ogg"),
        charge_sfx: asset_server.load("audio/sound_effects/ogre_charge.ogg"),
        charge_rect_mesh: meshes.add(Rectangle::new(1.0, 1.0)),
        charge_line_material: materials.add(StandardMaterial {
            base_color: OGRE_CHARGE_LINE_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
