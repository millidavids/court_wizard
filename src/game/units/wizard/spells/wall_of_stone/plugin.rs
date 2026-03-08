use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{WallHealth, WallOfStone};
use super::systems;
use crate::game::plugin::{PostCombatSet, VelocitySystemSet};
use crate::game::units::MovementCalculationSet;
use crate::game::run_conditions::is_spell_effects_active;
use crate::state::AppState;

/// Plugin that handles the Wall of Stone spell.
pub struct WallOfStonePlugin;

impl Plugin for WallOfStonePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            systems::register_permanent_wall_obstacles,
        )
        .add_systems(
            Update,
            (
                systems::handle_wall_of_stone_cancel.run_if(spell_is_primed(Spell::WallOfStone)),
                // Local wizard casting (mouse input)
                systems::handle_wall_of_stone_casting
                    .run_if(spell_is_primed(Spell::WallOfStone))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                (
                    systems::tick_wall_lifetime,
                    systems::animate_sinking_walls,
                    systems::cleanup_expired_walls,
                )
                    .chain()
                    .run_if(any_exist::<WallOfStone>()),
                // Blocked units attack walls — runs after all targeting systems
                // but before movement calculation, so it can override targeting
                // velocity for units with no valid path.
                systems::units_attack_blocking_walls
                    .after(VelocitySystemSet)
                    .before(MovementCalculationSet)
                    .run_if(any_exist::<WallOfStone>()),
                // Destroy walls at 0 HP
                systems::destroy_dead_walls
                    .in_set(PostCombatSet)
                    .run_if(any_with_component::<WallHealth>),
                // Visual damage tint
                systems::update_wall_damage_tint
                    .run_if(any_with_component::<WallHealth>),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
