use bevy::prelude::*;

use super::constants::*;

/// Pre-loaded sprite assets for the battlemage (swordcerer) avatar.
#[derive(Resource)]
pub struct BattlemageAssets {
    /// Rectangle mesh for sprite rendering.
    pub sprite_mesh: Handle<Mesh>,
    /// Swordcerer walking sprite sheet texture.
    pub sprite_texture: Handle<Image>,
    /// Attack animation sprite sheet.
    pub attacking_texture: Handle<Image>,
    /// Casting animation sprite sheet.
    pub casting_texture: Handle<Image>,
}

/// System to pre-load battlemage assets at startup.
pub(super) fn preload_battlemage_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let assets = BattlemageAssets {
        sprite_mesh: meshes.add(Rectangle::new(AVATAR_SPRITE_WIDTH, AVATAR_SPRITE_HEIGHT)),
        sprite_texture: asset_server.load("images/sprite_sheets/swordcerer-walking_9-frames.png"),
        attacking_texture: asset_server
            .load("images/sprite_sheets/swordcerer-attacking_6-frames.png"),
        casting_texture: asset_server.load("images/sprite_sheets/swordcerer-casting_7-frames.png"),
    };

    commands.insert_resource(assets);
}

/// Tracks the battlemage's field state.
#[derive(Resource, Debug, Clone, Default)]
pub struct BattlemageState {
    /// Animation phase for entering/exiting the field.
    pub phase: BattlemagePhase,
    /// Whether the battlemage has retreated (died on the field). Cannot re-enter.
    pub retreated: bool,
}

/// Current phase of the battlemage enter/exit sequence.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum BattlemagePhase {
    /// Idle on the castle wall, not in combat.
    #[default]
    Idle,
    /// Player clicked "Enter the Fray" and is choosing where to spawn.
    ChoosingLocation,
    /// Battlemage is actively on the field.
    OnField,
}
