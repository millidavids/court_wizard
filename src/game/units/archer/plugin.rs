use bevy::prelude::*;

use super::components::{Archer, Arrow};
use super::resources;
use super::systems::*;
use crate::game::run_conditions::any_exist;
use crate::state::InGameState;

pub struct ArcherPlugin;

impl Plugin for ArcherPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_archer_assets)
            .add_systems(
                Update,
                (
                    update_archer_targeting.in_set(crate::game::plugin::VelocitySystemSet),
                    archer_movement.in_set(crate::game::units::MovementCalculationSet),
                    (
                        update_archer_movement_timers,
                        archer_melee_combat,
                        archer_ranged_combat,
                    )
                        .chain(),
                )
                    .run_if(any_exist::<Archer>())
                    .run_if(in_state(InGameState::Running)),
            )
            .add_systems(
                Update,
                (move_arrows, check_arrow_collisions)
                    .chain()
                    .run_if(any_exist::<Arrow>())
                    .run_if(in_state(InGameState::Running)),
            );
    }
}
