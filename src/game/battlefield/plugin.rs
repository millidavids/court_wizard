use bevy::prelude::*;

use super::components::CastleWallAssets;

/// Plugin that handles battlefield and castle setup.
///
/// Battlefield is spawned via the loading spawn queue.
pub struct BattlefieldPlugin;

impl Plugin for BattlefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_castle_wall_assets);
    }
}

/// Pre-loads the castle wall texture at startup.
fn load_castle_wall_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("images/castle_wall.png");
    commands.insert_resource(CastleWallAssets { texture });
}
