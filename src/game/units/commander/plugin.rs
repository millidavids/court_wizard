use bevy::prelude::*;

use super::components::Commander;
use super::systems::*;
use crate::game::plugin::{MovementSystemSet, VelocitySystemSet};
use crate::game::run_conditions::is_gameplay_active;

pub struct CommanderPlugin;

impl Plugin for CommanderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            apply_commander_auras
                .run_if(is_gameplay_active)
                .run_if(any_with_component::<Commander>)
                .after(VelocitySystemSet)
                .before(MovementSystemSet),
        );
    }
}
