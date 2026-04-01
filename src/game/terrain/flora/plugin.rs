use bevy::prelude::*;

use super::components::Flora;
use super::resources::preload_flora_assets;
use super::systems;
use crate::game::run_conditions::{any_exist, is_gameplay_running};

pub struct FloraPlugin;

impl Plugin for FloraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, preload_flora_assets).add_systems(
            Update,
            systems::trample_flora
                .run_if(is_gameplay_running)
                .run_if(any_exist::<Flora>()),
        );
    }
}
