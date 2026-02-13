use bevy::prelude::*;

use super::components::{BrewBubble, Cauldron, CauldronBrewingEffects, CauldronState};
use super::resources::CauldronBuffs;

/// Returns true if the cauldron is currently brewing.
pub fn cauldron_is_brewing(query: Query<&CauldronState, With<Cauldron>>) -> bool {
    query.iter().any(|state| state.is_brewing())
}

/// Returns true if any cauldron buffs are currently active.
pub fn has_active_buffs(cauldron_buffs: Res<CauldronBuffs>) -> bool {
    cauldron_buffs.has_active_buffs()
}

/// Returns true if any brew bubbles exist in the world.
pub fn has_brew_bubbles(query: Query<(), With<BrewBubble>>) -> bool {
    !query.is_empty()
}

/// Returns true if the cauldron has brewing effects active.
pub fn has_brewing_effects(
    query: Query<(), (With<Cauldron>, With<CauldronBrewingEffects>)>,
) -> bool {
    !query.is_empty()
}
