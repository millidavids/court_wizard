use bevy::prelude::*;

use super::components::Assassin;
use super::{resources, systems};
use crate::game::run_conditions::any_exist;
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::MovementCalculationSet;

pub struct AssassinPlugin;

impl Plugin for AssassinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_assassin_assets)
            .add_systems(
                Update,
                (
                    systems::update_assassin_targeting
                        .in_set(crate::game::plugin::VelocitySystemSet),
                    systems::assassin_movement.in_set(MovementCalculationSet),
                )
                    .run_if(any_exist::<Assassin>())
                    .run_if(is_gameplay_running),
            );
    }
}
