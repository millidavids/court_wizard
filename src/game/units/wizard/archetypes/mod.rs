//! Wizard archetype systems (RuneCaster and Randomancer).

pub(crate) mod roulette;
pub(crate) mod runes;

use bevy::prelude::*;

use roulette::RoulettePlugin;
use runes::RunePlugin;

/// Plugin that manages all wizard archetypes.
pub(crate) struct ArchetypesPlugin;

impl Plugin for ArchetypesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RunePlugin, RoulettePlugin));
    }
}
