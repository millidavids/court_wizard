use bevy::prelude::*;

use super::resources;
use super::systems::*;
use crate::game::plugin::{PostCombatSet, VelocitySystemSet};
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::boss::components::Boss;
use crate::game::units::MovementCalculationSet;

pub struct OgrePlugin;

impl Plugin for OgrePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_ogre_assets)
            .add_systems(
                Update,
                (
                    update_ogre_targeting.in_set(VelocitySystemSet),
                    ogre_movement.in_set(MovementCalculationSet),
                    ogre_combat.in_set(MovementCalculationSet),
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Boss>),
            )
            .add_systems(
                Update,
                update_enrage_state
                    .after(PostCombatSet)
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Boss>),
            );
    }
}
