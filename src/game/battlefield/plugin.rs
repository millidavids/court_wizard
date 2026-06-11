use bevy::prelude::*;

use super::components::{LavaPool, WaterRipple, WaterRippleAssets};
use super::ground_material::{GroundMaterial, StoneNoiseMaterial};
use super::systems;
use super::systems::load_battlefield_assets;
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
