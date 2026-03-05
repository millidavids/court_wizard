use bevy::prelude::*;

use super::components::{CastleWallAssets, RightWallAssets};

/// Plugin that handles battlefield and castle setup.
///
/// Battlefield is spawned via the loading spawn queue.
pub struct BattlefieldPlugin;

impl Plugin for BattlefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_battlefield_assets);
    }
}

/// Pre-loads battlefield textures at startup.
fn load_battlefield_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let castle_texture = asset_server.load("images/castle_wall.png");
    commands.insert_resource(CastleWallAssets {
        texture: castle_texture,
    });

    let right_wall_texture = asset_server.load("images/static_sprites/right_wall.png");
    commands.insert_resource(RightWallAssets {
        texture: right_wall_texture,
    });
}
