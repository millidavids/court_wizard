use bevy::prelude::*;

use super::components::OgreThrowWindup;
use super::resources;
use super::systems::*;
use crate::game::plugin::{PostCombatSet, VelocitySystemSet};
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::MovementCalculationSet;
use crate::game::units::boss::components::Boss;

pub struct OgrePlugin;

impl Plugin for OgrePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_ogre_assets)
            .add_systems(
                Update,
                (
                    update_ogre_targeting.in_set(VelocitySystemSet),
                    ogre_charge_system.before(ogre_movement),
                    ogre_movement.in_set(MovementCalculationSet),
                    ogre_combat.in_set(MovementCalculationSet),
                    ogre_rock_throw,
                    ogre_throw_release
                        .after(crate::game::units::systems::update_combat_animation)
                        .run_if(any_with_component::<OgreThrowWindup>),
                    update_ogre_charge_visuals
                        .after(ogre_charge_system)
                        .after(crate::game::units::systems::update_walking_animation)
                        // This restores the ogre's transform from a position
                        // cached at telegraph start, so it must not run after the
                        // playable-area clamp — it would put an out-of-bounds
                        // ogre back outside on every telegraph frame.
                        .before(crate::game::movement_systems::enforce_playable_area),
                    update_ogre_facing.after(crate::game::units::systems::update_facing_direction),
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
