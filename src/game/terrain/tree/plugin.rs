use bevy::prelude::*;

use super::components::{BurningTree, Tree};
use super::resources;
use super::systems::*;
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::meteor_fall::components::MeteorExplosion;

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_tree_assets)
            .add_systems(
                Update,
                (
                    ignite_trees_from_fire.run_if(
                        any_with_component::<Tree>.and(
                            any_with_component::<FireballExplosion>
                                .or(any_with_component::<MeteorExplosion>)
                                .or(any_with_component::<DisintegrateBeam>),
                        ),
                    ),
                    apply_burning_tree_damage.run_if(any_with_component::<BurningTree>),
                    emit_burning_tree_vfx.run_if(any_with_component::<BurningTree>),
                )
                    .run_if(is_gameplay_running),
            );
    }
}
