use bevy::prelude::*;

use super::components::Teleporter;
use super::{resources, systems};
use crate::game::plugin::VelocitySystemSet;
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::MovementCalculationSet;

pub struct TeleporterPlugin;

impl Plugin for TeleporterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_teleporter_assets)
            .add_systems(
                Update,
                (
                    systems::update_teleporter_targeting.in_set(VelocitySystemSet),
                    systems::teleporter_movement.in_set(MovementCalculationSet),
                    systems::update_channel_state,
                    systems::refresh_teleporter_casting_animation,
                    systems::spawn_channel_particles,
                    systems::teleporter_ranged_combat,
                    systems::cleanup_dead_teleporter_channels,
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Teleporter>),
            );
    }
}
