use bevy::prelude::*;

use super::components::Commander;
use super::systems::*;
use crate::game::plugin::MovementSystemSet;
use crate::game::shared_systems::apply_separation;
use crate::state::AppState;

pub struct CommanderPlugin;

impl Plugin for CommanderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            apply_commander_auras
                .run_if(in_state(AppState::InGame))
                .run_if(any_with_component::<Commander>)
                .after(apply_separation)
                .before(MovementSystemSet),
        );
    }
}
