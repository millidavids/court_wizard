use bevy::prelude::*;

use super::components::*;
use super::resources;
use super::systems::*;
use crate::game::plugin::{PostCombatSet, VelocitySystemSet};
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::MovementCalculationSet;

pub struct HagsPlugin;

impl Plugin for HagsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_hag_assets)
            // Core movement and combat
            .add_systems(
                Update,
                (
                    update_hag_targeting.in_set(VelocitySystemSet),
                    hag_movement.in_set(MovementCalculationSet),
                    hag_separation.after(MovementCalculationSet),
                    hag_combat.in_set(MovementCalculationSet),
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Hag>),
            )
            // Eye system
            .add_systems(
                Update,
                (tick_eye_transfer, update_eye_flight)
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Hag>),
            )
            // Death handling (blind hags become permanent corpses)
            .add_systems(
                Update,
                (
                    intercept_blind_hag_death.after(PostCombatSet),
                    apply_enrage_to_last_hag.after(PostCombatSet),
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Hag>),
            )
            // Justina abilities (chain lightning, fireball)
            .add_systems(
                Update,
                (justina_chain_lightning, justina_fireball)
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Hag>),
            )
            // Josephina abilities (leap, mauling, corpse consume)
            .add_systems(
                Update,
                (
                    josephina_leap.in_set(MovementCalculationSet),
                    josephina_leap_knockback.after(MovementCalculationSet),
                    josephina_vicious_mauling.after(MovementCalculationSet),
                    josephina_corpse_consume,
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Hag>),
            )
            // Martina abilities (teleport pull, mind control)
            .add_systems(
                Update,
                (martina_teleport_pull, martina_mind_control)
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Hag>),
            )
;
    }
}
