use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;
use crate::game::units::components::{BattleHymnModifier, RemoteBattleHymnEffect};

pub struct BattleHymnPlugin;

impl Plugin for BattleHymnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_battle_hymn_casting
                    .run_if(spell_is_primed(Spell::BattleHymn))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                // Custom tick: handles EchoingSong re-apply on expiry
                systems::update_battle_hymn_modifier
                    .run_if(any_with_component::<BattleHymnModifier>),
                // Per-unit song-mote visual — real buff on this peer's units,
                // snapshot-mirrored marker on guest ghosts.
                systems::emit_battle_hymn_song_motes.run_if(
                    any_with_component::<BattleHymnModifier>
                        .or_else(any_with_component::<RemoteBattleHymnEffect>),
                ),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
