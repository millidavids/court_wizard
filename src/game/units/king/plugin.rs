use bevy::prelude::*;

use crate::game::plugin::{MovementSystemSet, VelocitySystemSet};
use crate::game::run_conditions::any_exist;
use crate::game::run_conditions::is_gameplay_running;

use super::components::{King, KingSpawned, SpellShield};
use super::resources;
use super::systems;

pub struct KingPlugin;

impl Plugin for KingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KingSpawned>()
            .add_systems(Startup, resources::preload_king_assets)
            .add_systems(
                Update,
                (
                    systems::update_king_targeting.in_set(VelocitySystemSet),
                    systems::king_movement.in_set(crate::game::units::MovementCalculationSet),
                    systems::king_cohesion_force
                        .after(VelocitySystemSet)
                        .before(MovementSystemSet),
                    systems::snap_kings_guard_to_king
                        .in_set(MovementSystemSet),
                )
                    .run_if(any_exist::<King>())
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                (
                    systems::attach_king_spell_shield,
                    systems::update_king_spell_shield
                        .run_if(any_with_component::<SpellShield>)
                        .run_if(is_gameplay_running),
                ),
            );
    }
}
