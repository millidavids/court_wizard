use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{HarvestFlash, PsychicShockwave, TelekinesisIndicator};
use super::run_conditions::has_telekinesis_talent;
use super::systems;
use crate::game::drops::components::IngredientDrop;
use crate::game::run_conditions::is_spell_effects_active;
use crate::state::AppState;

pub struct TelekinesisPlugin;

impl Plugin for TelekinesisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_telekinesis_casting
                    .run_if(spell_is_primed(Spell::Telekinesis))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                systems::update_telekinesis_indicator.run_if(any_exist::<TelekinesisIndicator>()),
                // T2: Magnetic Pull — passive drift when talent is active
                systems::magnetic_pull_ingredients
                    .run_if(has_telekinesis_talent(1, 0))
                    .run_if(any_exist::<IngredientDrop>()),
                // T3: Transmutation — track stacks on ingredient collection
                systems::track_transmutation_stacks.run_if(has_telekinesis_talent(2, 1)),
                // T2: Harvest flash visual effect
                systems::update_harvest_flash.run_if(any_exist::<HarvestFlash>()),
                // T3: Psychic Shockwave expanding ring
                systems::update_psychic_shockwave.run_if(any_exist::<PsychicShockwave>()),
            )
                .run_if(is_spell_effects_active),
        )
        .add_systems(
            OnEnter(AppState::InGame),
            systems::init_transmutation_stacks,
        )
        .add_systems(
            OnExit(AppState::InGame),
            systems::cleanup_transmutation_stacks,
        )
        .add_systems(
            OnExit(AppState::MultiplayerGame),
            systems::cleanup_transmutation_stacks,
        );
    }
}
