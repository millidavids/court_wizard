use bevy::prelude::*;

use super::components::{Dispeller, DispellerBolt};
use super::resources;
use super::systems::*;
use crate::game::run_conditions::any_exist;
use crate::game::run_conditions::is_gameplay_running;

pub struct DispellerPlugin;

impl Plugin for DispellerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_dispeller_assets)
            .add_systems(
                Update,
                (
                    update_dispeller_targeting.in_set(crate::game::plugin::VelocitySystemSet),
                    dispeller_movement.in_set(crate::game::units::MovementCalculationSet),
                    (update_dispel_channeling, dispeller_ranged_combat).chain(),
                )
                    .run_if(any_exist::<Dispeller>())
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                (move_dispeller_bolts, check_bolt_collisions)
                    .chain()
                    .run_if(any_exist::<DispellerBolt>())
                    .run_if(is_gameplay_running),
            );
    }
}
