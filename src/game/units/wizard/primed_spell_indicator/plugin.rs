use bevy::prelude::*;

use crate::game::run_conditions::is_spell_effects_active;
use crate::game::units::wizard::components::LocalWizard;

use super::icon;
use super::sync;

/// Plugin that floats the primed spell's icon above the wizard's head.
///
/// Gives the player a persistent read on what is loaded — the cast bar only
/// shows progress, and the rune/roulette spell names fade out.
pub struct PrimedSpellIndicatorPlugin;

impl Plugin for PrimedSpellIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            // Deliberately unchained: the three touch disjoint data (spawn
            // defers through `Commands`, sync writes the material + Visibility,
            // bob writes Transform), and chaining would insert an
            // `ApplyDeferred` barrier into `Update` every frame.
            (
                icon::spawn_primed_spell_icon,
                sync::sync_primed_spell_icon,
                icon::bob_primed_spell_icon,
            )
                // `is_spell_effects_active` is the visual/lifecycle condition —
                // unlike `is_gameplay_active` it also runs on the multiplayer
                // guest. The icon holds its last pose while paused.
                .run_if(is_spell_effects_active)
                .run_if(any_with_component::<LocalWizard>),
        );
    }
}
