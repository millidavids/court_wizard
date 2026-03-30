use bevy::prelude::*;

use super::components::*;
use super::resources;
use super::systems::*;
use crate::game::plugin::VelocitySystemSet;
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::MovementCalculationSet;

pub struct BrutePlugin;

impl Plugin for BrutePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_brute_assets)
            .add_systems(
                Update,
                (
                    update_brute_targeting.in_set(VelocitySystemSet),
                    brute_movement.in_set(MovementCalculationSet),
                    brute_rock_throw,
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Brute>),
            );
    }
}
