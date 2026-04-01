//! Shepherd wizard archetype.
//!
//! The Shepherd has no custom runtime systems — its spell power bonus is
//! applied directly to the Wizard component in `setup_wizard()`, and its spell
//! restriction is enforced by filtering in the spell book and action bar UIs.

use bevy::prelude::*;

/// Spell power multiplier for the Shepherd wizard type.
/// All non-damaging spells get this bonus to heal amounts, shield values, buff durations, etc.
pub(crate) const SPELL_POWER_MULTIPLIER: f32 = 1.30;

/// Plugin for the Shepherd wizard archetype.
pub struct ShepherdPlugin;

impl Plugin for ShepherdPlugin {
    fn build(&self, _app: &mut App) {
        // No runtime systems needed — bonus is static and spell filtering
        // is handled by `Spell::is_shepherd_allowed()` in the UI layer.
    }
}
