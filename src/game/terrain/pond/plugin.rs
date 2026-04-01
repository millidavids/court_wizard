use bevy::prelude::*;

use super::components::Pond;
use super::resources;
use super::systems::*;
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::wizard::archetypes::meteorologist::components::WetModifier;

pub struct PondPlugin;

impl Plugin for PondPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_pond_assets)
            .add_systems(
                Update,
                (
                    apply_pond_wet.run_if(any_with_component::<Pond>),
                    tick_wet_timer.run_if(any_with_component::<WetModifier>),
                    emit_pond_ripples.run_if(any_with_component::<Pond>),
                )
                    .run_if(is_gameplay_running),
            );
    }
}
