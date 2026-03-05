use bevy::prelude::*;

use super::components::BattlefieldAssets;

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
    commands.insert_resource(BattlefieldAssets {
        castle_wall: asset_server.load("images/castle_wall.png"),
        right_wall: asset_server.load("images/static_sprites/right_wall.png"),
    });
}
