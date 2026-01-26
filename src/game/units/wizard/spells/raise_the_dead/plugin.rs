use bevy::prelude::*;

use crate::state::InGameState;

use super::systems::*;

/// Plugin for the Raise The Dead spell.
///
/// Manages the necromancy spell that resurrects corpses as hostile undead units.
pub struct RaiseTheDeadPlugin;

impl Plugin for RaiseTheDeadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_raise_the_dead_casting.run_if(in_state(InGameState::Running)),
        );
    }
}
