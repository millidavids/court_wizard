use bevy::prelude::*;

use super::components::*;
use super::resources;
use super::systems::*;
use crate::game::plugin::{PostCombatSet, VelocitySystemSet};
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::{ApplyTransformsSet, MovementCalculationSet};

pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_boss_assets)
            .add_systems(
                Update,
                (
                    update_boss_targeting.in_set(VelocitySystemSet),
                    boss_movement.in_set(MovementCalculationSet),
                    boss_combat.in_set(MovementCalculationSet),
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
            )
            .add_systems(
                Update,
                apply_knockback_effects
                    .after(ApplyTransformsSet)
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<KnockbackEffect>),
            );
    }
}
