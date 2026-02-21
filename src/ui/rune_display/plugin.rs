use bevy::prelude::*;

use crate::game::run_conditions;
use crate::game::run_conditions::is_local_wizard_active;
use crate::state::{InGameState, MultiplayerGameState};
use crate::ui::plugin::ButtonActionSet;

use super::components::SpellNameFadeTimer;
use super::systems;

/// Plugin managing the rune display UI. Only active for the RuneCaster archetype.
pub struct RuneDisplayPlugin;

impl Plugin for RuneDisplayPlugin {
    fn build(&self, app: &mut App) {
        // SP spawn
        app.add_systems(
            OnEnter(InGameState::Running),
            systems::spawn_rune_display.run_if(run_conditions::is_rune_caster),
        )
        // MP spawn
        .add_systems(
            OnEnter(MultiplayerGameState::Running),
            systems::spawn_rune_display.run_if(run_conditions::is_rune_caster),
        )
        .add_systems(
            Update,
            (
                systems::handle_rune_button_click.in_set(ButtonActionSet),
                systems::show_spell_name_on_activation,
                systems::update_rune_display,
                systems::update_spell_name_fade.run_if(any_with_component::<SpellNameFadeTimer>),
            )
                .run_if(is_local_wizard_active)
                .run_if(run_conditions::is_rune_caster),
        );
    }
}
