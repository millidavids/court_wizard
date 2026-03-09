use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{GuardianCircleIndicator, GuardianCircleShielded};
use super::systems;
use crate::game::plugin::PostCombatSet;
use crate::game::run_conditions::{any_exist, is_gameplay_running, is_spell_effects_active};
use crate::game::units::wizard::spells::utils;

/// Plugin that handles Guardian Circle spell casting and behavior.
///
/// Registers systems for:
/// - Casting Guardian Circle with mouse button and cast time
/// - Visual circle indicator during cast
/// - Applying temporary HP buff to units in area
/// - Circle animation and updates
/// - Talent effects: Retaliating Wards, Martyrdom, Chain Ward, cleanup
pub struct GuardianCirclePlugin;

impl Plugin for GuardianCirclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            // Local wizard casting (mouse input)
            systems::handle_guardian_circle_casting
                .run_if(spell_is_primed(Spell::GuardianCircle))
                .run_if(spell_input_not_blocked)
                .run_if(mouse_left_not_consumed)
                .run_if(mouse_held_or_wizard_casting)
                .run_if(is_spell_effects_active),
        );
        app.add_systems(
            Update,
            utils::update_circle_indicator::<GuardianCircleIndicator>
                .run_if(any_exist::<GuardianCircleIndicator>())
                .run_if(is_spell_effects_active),
        );
        // Talent reaction systems — run after combat resolves
        app.add_systems(
            Update,
            (
                systems::retaliating_wards_check,
                systems::martyrdom_on_death,
                systems::chain_ward_on_death,
                systems::cleanup_guardian_circle_shielded,
            )
                .after(PostCombatSet)
                .run_if(is_gameplay_running)
                .run_if(any_exist::<GuardianCircleShielded>()),
        );
    }
}
