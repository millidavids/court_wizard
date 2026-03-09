use bevy::prelude::*;

/// Plugin for the Shepherd wizard archetype.
///
/// The Shepherd has no custom runtime systems — its 1.30x spell power bonus is
/// applied directly to the Wizard component in `setup_wizard()`, and its spell
/// restriction is enforced by filtering in the spell book and action bar UIs.
pub struct ShepherdPlugin;

impl Plugin for ShepherdPlugin {
    fn build(&self, _app: &mut App) {
        // No runtime systems needed — bonus is static and spell filtering
        // is handled by `Spell::is_shepherd_allowed()` in the UI layer.
    }
}
