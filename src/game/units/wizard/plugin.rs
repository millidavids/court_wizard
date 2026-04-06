use bevy::prelude::*;

use crate::game::run_conditions::{
    is_local_wizard_active, is_not_warglock, is_spell_effects_active,
};
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
            .add_systems(
                Update,
                (
                    systems::regenerate_mana.run_if(is_not_warglock),
                    systems::mana_on_kill.run_if(
                        |toggles: Option<Res<crate::game::game_mode::components::ActiveToggles>>| {
                            toggles.as_ref().is_some_and(|t| {
                                t.is_active(
                                    crate::game::game_mode::components::ToggleModifier::ManaDrought,
                                )
                            })
                        },
                    ),
                    systems::reset_empowerment_after_cast,
                    systems::apply_wizard_stats_to_primed_spell,
                )
                    .run_if(is_spell_effects_active),
            )
            .add_systems(
                Update,
                systems::update_wizard_animation.run_if(is_spell_effects_active),
            )
            .add_systems(
                Update,
                systems::handle_prime_spell_messages.run_if(is_local_wizard_active),
            )
            .add_systems(
                PostUpdate,
                systems::track_spell_casts_for_rotation
                    .run_if(is_spell_effects_active)
                    .run_if(resource_exists::<crate::game::game_mode::components::LastCastSpell>),
            )
            .add_systems(OnExit(InGameState::Running), systems::cancel_active_casts);
    }
}
