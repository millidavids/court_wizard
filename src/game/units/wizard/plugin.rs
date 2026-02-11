use bevy::prelude::*;

use crate::game::run_conditions;
use crate::state::InGameState;

use super::archetypes::ArchetypesPlugin;
use super::messages::PrimeSpellMessage;
use super::spell_range_indicator::SpellRangeIndicatorPlugin;
use super::spells::SpellsPlugin;
use super::systems;

/// Plugin that handles wizard entity setup, spells, and archetypes.
///
/// Registers systems for:
/// - Wizard entity setup on entering InGame state
/// - Re-setup when entering Running state from GameOver (for replay)
/// - Mana regeneration during gameplay
/// - Spell priming via messages
/// - Spell casting and projectile management (via SpellsPlugin)
/// - Spell range visualization (via SpellRangeIndicatorPlugin)
/// - Archetype systems (runes and roulette via ArchetypesPlugin)
pub struct WizardPlugin;

impl Plugin for WizardPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PrimeSpellMessage>()
            .add_plugins((SpellsPlugin, SpellRangeIndicatorPlugin, ArchetypesPlugin))
            // Note: setup_wizard is now called via the loading spawn queue
            // Only re-setup when coming from GameOver for replay
            .add_systems(
                OnEnter(InGameState::Running),
                systems::setup_wizard.run_if(run_conditions::coming_from_game_over),
            )
            .add_systems(
                Update,
                (
                    systems::regenerate_mana,
                    systems::handle_prime_spell_messages,
                    systems::reset_empowerment_after_cast,
                )
                    .run_if(in_state(InGameState::Running)),
            )
            .add_systems(OnExit(InGameState::Running), systems::cancel_active_casts);
    }
}
