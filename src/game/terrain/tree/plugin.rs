use bevy::prelude::*;

use super::components::Tree;
use super::resources;
use super::systems::*;
use crate::game::run_conditions::is_gameplay_running;

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_tree_assets)
            .add_systems(
                Update,
                (
                    apply_spell_damage_to_trees.run_if(any_with_component::<Tree>),
                    destroy_dead_trees.run_if(any_with_component::<Tree>),
                )
                    .run_if(is_gameplay_running),
            );
    }
}
