use bevy::prelude::*;

use super::components::{
    ClonedPondMaterial, Pond, PondEvaporation, PondFogCloud, PondFrozen, PondShocked,
};
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
                    apply_frozen_pond_slow
                        .after(crate::game::movement_systems::apply_rough_terrain_slowdown)
                        .run_if(any_with_component::<PondFrozen>),
                    tick_pond_evaporation.run_if(any_with_component::<PondEvaporation>),
                    emit_pond_fog_particles.run_if(any_with_component::<PondFogCloud>),
                    tick_pond_frozen.run_if(any_with_component::<PondFrozen>),
                    update_frozen_pond_tint.run_if(any_with_component::<PondFrozen>),
                    restore_pond_material_on_thaw
                        .run_if(any_with_component::<ClonedPondMaterial>),
                    tick_pond_shocked.run_if(any_with_component::<PondShocked>),
                )
                    .run_if(is_gameplay_running),
            );
    }
}
