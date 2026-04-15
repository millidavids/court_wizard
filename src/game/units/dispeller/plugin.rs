use bevy::prelude::*;

use super::components::Dispeller;
use super::resources;
use super::systems::*;
use crate::game::run_conditions::any_exist;
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::wizard::spells::vfx::channel::ChannelingCast;

pub struct DispellerPlugin;

impl Plugin for DispellerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_dispeller_assets)
            .add_systems(
                Update,
                (
                    update_dispeller_targeting.in_set(crate::game::plugin::VelocitySystemSet),
                    dispeller_movement.in_set(crate::game::units::MovementCalculationSet),
                    (dispeller_start_dispel_channel, dispeller_ranged_combat).chain(),
                )
                    .run_if(any_exist::<Dispeller>())
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                (
                    dispeller_tick_dispel_channel,
                    dispeller_refresh_casting_animation,
                    dispeller_spawn_channel_particles,
                )
                    .run_if(any_with_component::<ChannelingCast>)
                    .run_if(is_gameplay_running),
            );
    }
}
