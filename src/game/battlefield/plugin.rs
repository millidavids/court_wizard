use bevy::prelude::*;

use super::components::{BattlefieldAssets, LavaPool, WaterRipple, WaterRippleAssets};
use super::ground_material::{GroundMaterial, StoneNoiseMaterial};
use super::systems;
use super::trampling::TramplingPlugin;
use crate::game::pathfinding::resources::PathfindingGrid;
use crate::game::run_conditions::{any_exist, is_gameplay_running};

/// Plugin that handles battlefield and castle setup.
///
/// Battlefield is spawned via the loading spawn queue.
pub struct BattlefieldPlugin;

impl Plugin for BattlefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
                TramplingPlugin,
                MaterialPlugin::<GroundMaterial>::default(),
                MaterialPlugin::<StoneNoiseMaterial>::default(),
            ))
            .add_systems(Startup, load_battlefield_assets)
            .add_systems(
                Update,
                (
                    (systems::emit_lava_fire_smoke, systems::emit_lava_sparks)
                        .run_if(any_exist::<LavaPool>()),
                    systems::emit_water_ripples.run_if(resource_exists::<WaterRippleAssets>),
                    systems::update_water_ripples.run_if(any_exist::<WaterRipple>()),
                    systems::emit_ambient_motes,
                )
                    .run_if(is_gameplay_running),
            )
            // Terrain hazard systems (lava damage, water slow)
            .add_systems(
                Update,
                (systems::apply_lava_damage, systems::apply_water_slow)
                    .run_if(is_gameplay_running)
                    .run_if(resource_exists::<PathfindingGrid>),
            );
    }
}

/// Pre-loads battlefield textures at startup.
pub(crate) fn load_battlefield_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BattlefieldAssets {
        castle_wall: asset_server.load("images/castle_wall.png"),
        right_wall: asset_server.load("images/static_sprites/right_wall.png"),
        left_wall: asset_server.load("images/static_sprites/left_wall.png"),
        wall_floor: asset_server.load("images/static_sprites/wall_floor.png"),
        battlefield_tiles: asset_server.load("images/sprite_sheets/battlefield_tiles.png"),
    });
}
