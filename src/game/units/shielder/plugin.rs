use bevy::prelude::*;

use super::components::Shielder;
use super::resources;
use super::systems::*;
use crate::game::run_conditions::any_exist;
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::wizard::spells::vfx::channel::ChannelingCast;

pub struct ShielderPlugin;

impl Plugin for ShielderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_shielder_assets)
            .add_systems(
                Update,
                (
                    update_shielder_targeting.in_set(crate::game::plugin::VelocitySystemSet),
                    shielder_movement.in_set(crate::game::units::MovementCalculationSet),
                    shielder_start_shield_channel,
                )
                    .run_if(any_exist::<Shielder>())
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                (
                    shielder_tick_shield_channel,
                    shielder_refresh_casting_animation,
                    shielder_spawn_channel_particles,
                )
                    .run_if(any_with_component::<ChannelingCast>)
                    .run_if(is_gameplay_running),
            );
    }
}
