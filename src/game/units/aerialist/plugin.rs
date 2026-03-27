use bevy::prelude::*;

use super::components::Aerialist;
use super::{resources, systems};
use crate::game::run_conditions::{any_exist, is_gameplay_running};
use crate::game::units::ApplyTransformsSet;

pub struct AerialistPlugin;

impl Plugin for AerialistPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_aerialist_assets)
            .add_systems(
                Update,
                (
                    systems::update_aerialist_targeting
                        .in_set(crate::game::plugin::VelocitySystemSet),
                    systems::aerialist_movement
                        .in_set(crate::game::units::MovementCalculationSet),
                    systems::aerialist_combat,
                    systems::clamp_aerialist_height.in_set(ApplyTransformsSet),
                )
                    .run_if(any_exist::<Aerialist>())
                    .run_if(is_gameplay_running),
            );
    }
}
