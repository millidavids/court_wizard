use bevy::prelude::*;

use crate::game::plugin::PostCombatSet;
use crate::game::run_conditions::any_exist;
use crate::game::run_conditions::is_gameplay_running;
use crate::state::AppState;

use super::components::{FlyingToWizard, IngredientDrop, LockedIngredients};
use super::messages::SpawnIngredientDropMessage;
use super::systems;

pub struct DropsPlugin;

impl Plugin for DropsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LockedIngredients>()
            .add_message::<SpawnIngredientDropMessage>()
            .add_systems(OnEnter(AppState::Loading), systems::init_locked_ingredients)
            .add_systems(
                Update,
                (
                    systems::spawn_ingredient_drops,
                    (systems::animate_drops, systems::tick_drop_lifetimes)
                        .run_if(any_exist::<IngredientDrop>()),
                    systems::fly_drops_to_wizard.run_if(any_exist::<FlyingToWizard>()),
                )
                    .after(PostCombatSet)
                    .run_if(is_gameplay_running),
            );
    }
}
