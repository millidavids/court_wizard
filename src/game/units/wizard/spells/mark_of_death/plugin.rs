use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{ActiveMarkOfDeath, DeathsLedgerBurst, MarkTalentFlags, MarkVisualIndicator};
use super::systems;
use crate::game::run_conditions::{any_exist, is_spell_effects_active};

pub struct MarkOfDeathPlugin;

impl Plugin for MarkOfDeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_mark_of_death_casting
                    .run_if(spell_is_primed(Spell::MarkOfDeath))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                // Talent systems (run when marks exist on living units)
                systems::tick_doom_marks
                    .run_if(any_exist::<ActiveMarkOfDeath>()),
                systems::executioner_brand_check
                    .run_if(any_exist::<ActiveMarkOfDeath>()),
                systems::focal_point_retarget
                    .run_if(any_exist::<ActiveMarkOfDeath>())
                    .after(crate::game::plugin::VelocitySystemSet),
                // Visual indicators
                systems::spawn_mark_indicators
                    .run_if(any_exist::<ActiveMarkOfDeath>()),
                systems::update_mark_indicators
                    .run_if(any_exist::<MarkVisualIndicator>()),
                // Death-triggered talent systems
                systems::handle_marked_corpses
                    .run_if(any_exist::<MarkTalentFlags>()),
                // Death's Ledger explosion systems
                systems::apply_deaths_ledger_damage
                    .run_if(any_exist::<DeathsLedgerBurst>()),
                systems::update_deaths_ledger_bursts
                    .run_if(any_exist::<DeathsLedgerBurst>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
