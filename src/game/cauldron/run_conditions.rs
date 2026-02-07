use bevy::prelude::*;

use super::components::{Cauldron, CauldronState};
use super::resources::CauldronBuffs;

/// Returns true if the cauldron is currently brewing.
pub fn cauldron_is_brewing(query: Query<&CauldronState, With<Cauldron>>) -> bool {
    query.iter().any(|state| state.is_brewing())
}

/// Returns true if any cauldron buffs are currently active.
pub fn has_active_buffs(cauldron_buffs: Res<CauldronBuffs>) -> bool {
    cauldron_buffs.has_active_buffs()
}
