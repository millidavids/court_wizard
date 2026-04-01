use bevy::prelude::*;

use super::components::{BurningBush, Bush};
use super::resources;
use super::systems::*;
use crate::game::run_conditions::is_gameplay_running;

pub struct BushPlugin;

impl Plugin for BushPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_bush_assets)
            .add_systems(
                Update,
                (
                    apply_bush_slow.run_if(any_with_component::<Bush>),
                    ignite_bushes_from_fire.run_if(any_with_component::<Bush>),
                    apply_burning_bush_damage.run_if(any_with_component::<BurningBush>),
                    emit_burning_bush_vfx.run_if(any_with_component::<BurningBush>),
                )
                    .run_if(is_gameplay_running),
            );
    }
}
