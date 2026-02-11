//! Wizard archetype systems (RuneCaster, Randomancer, and Arcanorouter).

pub mod arcanorouter;
pub(crate) mod roulette;
pub(crate) mod runes;

use bevy::prelude::*;

use arcanorouter::ArcanoRouterPlugin;
use roulette::RoulettePlugin;
use runes::RunePlugin;

/// Plugin that manages all wizard archetypes.
pub(crate) struct ArchetypesPlugin;

impl Plugin for ArchetypesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RunePlugin, RoulettePlugin, ArcanoRouterPlugin));
    }
}
