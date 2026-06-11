use bevy::prelude::*;

use super::super::components::WizardAssets;

/// Loads the wizard sprite sheet texture.
pub fn load_wizard_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sprite_texture = asset_server.load("images/wizard_idle-128px-9.png");
    let guest_sprite_texture = asset_server.load("images/guest_wizard_idle-128px-9.png");
    commands.insert_resource(WizardAssets {
        sprite_texture,
        guest_sprite_texture,
    });
}
