//! Wizard archetype systems (RuneCaster, Randomancer, Arcanorouter, Gunslinger, Battlemage, Meteorologist, and Shepherd).

pub mod arcanorouter;
pub mod battlemage;
pub mod gunslinger;
pub mod meteorologist;
pub(crate) mod roulette;
pub(crate) mod runes;
pub mod shepherd;

use bevy::prelude::*;

use arcanorouter::ArcanoRouterPlugin;
use battlemage::BattlemagePlugin;
use gunslinger::GunslingerPlugin;
use meteorologist::MeteorologistPlugin;
use roulette::RoulettePlugin;
use runes::RunePlugin;
use shepherd::ShepherdPlugin;

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
            MeteorologistPlugin,
            ShepherdPlugin,
        ));
    }
}
