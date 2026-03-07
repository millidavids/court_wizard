//! Wizard archetype systems (RuneCaster, Randomancer, Arcanorouter, Gunslinger, and Battlemage).

pub mod arcanorouter;
pub mod battlemage;
pub mod gunslinger;
pub(crate) mod roulette;
pub(crate) mod runes;

use bevy::prelude::*;

use arcanorouter::ArcanoRouterPlugin;
use battlemage::BattlemagePlugin;
use gunslinger::GunslingerPlugin;
use roulette::RoulettePlugin;
use runes::RunePlugin;

/// Plugin that manages all wizard archetypes.
pub(crate) struct ArchetypesPlugin;

impl Plugin for ArchetypesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RunePlugin,
            RoulettePlugin,
            ArcanoRouterPlugin,
            GunslingerPlugin,
            BattlemagePlugin,
        ));
    }
}
