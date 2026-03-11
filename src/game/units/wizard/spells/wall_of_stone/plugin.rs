use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{DispelledWall, LivingStoneTracker, PermafrostAuraTimer, WallHealth, WallOfStone, WallRising, WallTalents};
use super::systems;
use crate::game::plugin::{PostCombatSet, VelocitySystemSet};
use crate::game::units::MovementCalculationSet;
use crate::game::run_conditions::is_spell_effects_active;
use crate::state::AppState;

/// Plugin that handles the Wall of Stone spell.
pub struct WallOfStonePlugin;

impl Plugin for WallOfStonePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PermafrostAuraTimer>()
            .add_systems(
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
                    // Process walls marked for dispel (starts sink animation)
                    systems::handle_dispelled_walls
                        .run_if(any_with_component::<DispelledWall>),
                    // Destroy walls at 0 HP
                    systems::destroy_dead_walls
                        .in_set(PostCombatSet)
                        .run_if(any_with_component::<WallHealth>),
                    // Visual damage tint
                    systems::update_wall_damage_tint
                        .run_if(any_with_component::<WallHealth>),
                    // Wall rise animation (grows from ground)
                    systems::animate_rising_walls
                        .run_if(any_with_component::<WallRising>),
                    // Dust VFX during rise and sink
                    systems::spawn_wall_dust
                        .run_if(any_exist::<WallOfStone>()),
                    // --- Talent systems ---
                    // Permafrost Aura: slow enemies near walls
                    systems::apply_permafrost_aura
                        .run_if(any_with_component::<WallTalents>),
                    // Living Stone: regen wall HP when not being attacked
                    systems::regenerate_living_stone
                        .run_if(any_with_component::<LivingStoneTracker>),
                    // Collapsing Wall: AoE damage on wall destruction
                    systems::collapsing_wall_explosion
                        .in_set(PostCombatSet)
                        .after(systems::destroy_dead_walls)
                        .run_if(any_with_component::<WallTalents>),
                    // Maze Architect: bonus HP with 3+ walls
                    systems::maze_architect_bonus
                        .run_if(any_with_component::<WallTalents>),
                )
                    .run_if(is_spell_effects_active),
            );
    }
}
