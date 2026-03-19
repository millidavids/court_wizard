use bevy::prelude::*;

use super::components::Shielder;
use super::resources;
use super::systems::*;
use crate::game::run_conditions::any_exist;
use crate::game::run_conditions::is_gameplay_running;

pub struct ShielderPlugin;

impl Plugin for ShielderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_shielder_assets)
            .add_systems(
                Update,
                (
                    update_shielder_targeting.in_set(crate::game::plugin::VelocitySystemSet),
                    shielder_movement.in_set(crate::game::units::MovementCalculationSet),
                    shielder_apply_shield,
                )
                    .run_if(any_exist::<Shielder>())
                    .run_if(is_gameplay_running),
            );
    }
}
