use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::*;

use super::boulder::BoulderPlugin;
use super::bush::BushPlugin;
use super::flora::FloraPlugin;
use super::messages::TerrainDamageMessage;
use super::pond::PondPlugin;
use super::systems;
use super::tree::TreePlugin;
use super::wind_sway::WindSwayPlugin;
use crate::game::run_conditions::is_gameplay_running;

/// Top-level terrain plugin. Composes child terrain plugins, registers
/// `TerrainDamageMessage`, and runs cross-cutting fan-out systems that apply
/// reactive effects to ponds and boulders.
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TerrainDamageMessage>()
            .add_plugins((
                BoulderPlugin,
                BushPlugin,
                FloraPlugin,
                PondPlugin,
                TreePlugin,
                WindSwayPlugin,
            ))
            .add_systems(
                Update,
                (
                    systems::apply_terrain_damage_to_ponds,
                    systems::apply_terrain_damage_to_boulders,
                )
                    .run_if(on_message::<TerrainDamageMessage>)
                    .run_if(is_gameplay_running),
            );
    }
}
