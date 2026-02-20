use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::BattleHymnIndicator;
use super::systems;
use crate::game::run_conditions::is_gameplay_running;

pub struct BattleHymnPlugin;

impl Plugin for BattleHymnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_battle_hymn_casting
                    .run_if(spell_is_primed(Spell::BattleHymn))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                systems::update_battle_hymn_indicator.run_if(any_exist::<BattleHymnIndicator>()),
            )
                .run_if(is_gameplay_running),
        );
    }
}
