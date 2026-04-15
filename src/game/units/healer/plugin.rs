use bevy::prelude::*;

use super::components::{HealBolt, Healer};
use super::resources;
use super::systems::*;
use crate::game::run_conditions::any_exist;
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::wizard::spells::vfx::channel::ChannelingCast;

pub struct HealerPlugin;

impl Plugin for HealerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_healer_assets)
            .add_systems(
                Update,
                (
                    update_healer_targeting.in_set(crate::game::plugin::VelocitySystemSet),
                    healer_movement.in_set(crate::game::units::MovementCalculationSet),
                    healer_start_heal_channel,
                )
                    .run_if(any_exist::<Healer>())
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                (
                    healer_tick_heal_channel,
                    healer_refresh_casting_animation,
                    healer_spawn_channel_particles,
                )
                    .run_if(any_with_component::<ChannelingCast>)
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                (move_heal_bolts, check_heal_bolt_arrivals)
                    .chain()
                    .run_if(any_exist::<HealBolt>())
                    .run_if(is_gameplay_running),
            );
    }
}
