use bevy::prelude::*;

/// Plugin that handles battlefield and castle setup.
///
/// Battlefield is spawned via the loading spawn queue.
pub struct BattlefieldPlugin;

impl Plugin for BattlefieldPlugin {
    fn build(&self, _app: &mut App) {
        // setup_battlefield is called via the loading spawn queue
    }
}
