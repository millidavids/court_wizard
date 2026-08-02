use bevy::prelude::*;

use super::components::{Lich, LichSpawnPending};
use super::resources;
use super::systems::*;
use crate::game::plugin::VelocitySystemSet;
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::{ApplyTransformsSet, MovementCalculationSet};
use crate::state::AppState;

pub struct LichPlugin;

impl Plugin for LichPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_lich_assets)
            // A leaked pending flag would summon a Lich on an unrelated level and
            // block that level's victory, so clear it whenever a match ends.
            .add_systems(OnExit(AppState::InGame), clear_lich_spawn_pending)
            .add_systems(OnExit(AppState::MultiplayerGame), clear_lich_spawn_pending)
            // Mid-game spawn check runs even before the Lich exists
            .add_systems(
                Update,
                check_lich_spawn
                    .run_if(is_gameplay_running)
                    .run_if(resource_exists::<LichSpawnPending>),
            )
            .add_systems(
                Update,
                (
                    lich_approach_system,
                    lich_summoning_system,
                    track_soul_power,
                    lich_phase_transition,
                    lich_combat_targeting,
                    lich_fire_beam,
                    tick_lich_casting,
                )
                    .chain()
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Lich>),
            )
            .add_systems(
                Update,
                (
                    update_lich_targeting.in_set(VelocitySystemSet),
                    lich_movement.in_set(MovementCalculationSet),
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Lich>),
            )
            .add_systems(
                Update,
                (
                    on_lich_cast_started,
                    on_lich_cast_ended,
                    update_lich_facing.after(crate::game::units::systems::update_facing_direction),
                    update_lich_float.after(ApplyTransformsSet),
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Lich>),
            );
    }
}
