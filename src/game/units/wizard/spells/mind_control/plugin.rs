use bevy::prelude::*;

use super::components::{
    AmnesiaEffect, Demoralized, MassHysteriaTarget, MindControlCooldown, SleeperAgentActive,
    TraitorsMarkAura,
};
use super::systems;
use crate::game::plugin::{PostCombatSet, VelocitySystemSet};
use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};
use crate::game::units::MovementCalculationSet;
use crate::game::units::components::{MindControlled, RetaliationTarget};
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::run_conditions::*;
use crate::game::units::wizard::spells::utils::tick_spell_cooldown;

pub struct MindControlPlugin;

impl Plugin for MindControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tick_spell_cooldown::<MindControlCooldown>
                    .run_if(any_exist::<MindControlCooldown>())
                    .run_if(is_spell_effects_active),
                systems::handle_mind_control_casting
                    .run_if(spell_is_primed(Spell::MindControl))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting)
                    .run_if(is_spell_effects_active),
            ),
        )
        // Shared mind control behavior systems (used by both wizard spell and hag boss)
        .add_systems(
            Update,
            (
                crate::game::units::boss::hags::systems::update_mind_controlled_targeting
                    .in_set(VelocitySystemSet),
                crate::game::units::boss::hags::systems::update_mind_control_wear_off,
                crate::game::units::boss::hags::systems::mind_controlled_combat
                    .after(PostCombatSet),
                // Override the blended steering so MC'd units charge their former
                // allies instead of marching with the herd toward the enemy base.
                crate::game::units::boss::hags::systems::mind_controlled_pursue_allies
                    .after(MovementCalculationSet),
            )
                .run_if(is_gameplay_running)
                .run_if(any_with_component::<MindControlled>),
        )
        // Clean up RetaliationTarget when the MC unit is no longer mind-controlled
        .add_systems(
            Update,
            crate::game::units::boss::hags::systems::cleanup_retaliation_targets
                .run_if(is_gameplay_running)
                .run_if(any_with_component::<RetaliationTarget>),
        )
        // Talent behavior systems
        .add_systems(
            Update,
            systems::update_traitors_mark_aura
                .run_if(is_gameplay_running)
                .run_if(
                    any_with_component::<TraitorsMarkAura>
                        .or_else(any_with_component::<Demoralized>),
                ),
        )
        .add_systems(
            Update,
            systems::update_mass_hysteria_targeting
                .after(VelocitySystemSet)
                .run_if(is_gameplay_running)
                .run_if(any_with_component::<MassHysteriaTarget>),
        )
        .add_systems(
            Update,
            systems::tick_mass_hysteria
                .run_if(is_gameplay_running)
                .run_if(any_with_component::<MassHysteriaTarget>),
        )
        .add_systems(
            Update,
            systems::tick_amnesia_effect
                .run_if(is_gameplay_running)
                .run_if(any_with_component::<AmnesiaEffect>),
        )
        .add_systems(
            Update,
            systems::tick_sleeper_agent
                .run_if(is_gameplay_running)
                .run_if(any_with_component::<SleeperAgentActive>),
        );
    }
}
