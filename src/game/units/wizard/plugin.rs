use bevy::prelude::*;

use crate::state::{AppState, InGameState};

use super::components::PrimeSpellMessage;
use super::spell_range_indicator::SpellRangeIndicatorPlugin;
use super::spells::SpellsPlugin;
use super::systems;

/// Plugin that handles wizard entity setup and spells.
///
/// Registers systems for:
/// - Wizard entity setup on entering InGame state
/// - Mana regeneration during gameplay
/// - Spell priming via messages
/// - Spell casting and projectile management (via SpellsPlugin)
/// - Spell range visualization (via SpellRangeIndicatorPlugin)
pub struct WizardPlugin;

impl Plugin for WizardPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PrimeSpellMessage>()
            .add_plugins((SpellsPlugin, SpellRangeIndicatorPlugin))
            .add_systems(OnEnter(AppState::InGame), systems::setup_wizard)
            .add_systems(
                Update,
                (
                    systems::regenerate_mana,
                    systems::handle_prime_spell_messages,
                )
                    .run_if(in_state(InGameState::Running)),
            )
            .add_systems(OnExit(InGameState::Running), systems::cancel_active_casts);
    }
}
