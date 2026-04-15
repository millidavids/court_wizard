//! Black Hole spell plugin.

use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{BlackHole, BlackHoleSfx};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;
use crate::game::units::MovementCalculationSet;
use crate::game::units::wizard::spells::utils::{PendingDefenderHeal, apply_pending_defender_heal};

/// Plugin that manages the Black Hole spell.
#[derive(Default)]
pub struct BlackHolePlugin;

impl Plugin for BlackHolePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_black_hole_casting
                    .run_if(spell_is_primed(Spell::BlackHole))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                (
                    systems::apply_gravitational_forces.in_set(MovementCalculationSet),
                    systems::apply_corpse_gravity_and_despawn.in_set(MovementCalculationSet),
                    systems::apply_black_hole_damage,
                    systems::apply_crushing_pressure,
                    systems::apply_dimensional_rift,
                    systems::remove_units_from_black_hole,
                    systems::update_black_hole_visuals,
                    systems::despawn_expired_black_holes,
                )
                    .chain()
                    .run_if(any_exist::<BlackHole>()),
                systems::cleanup_black_hole_sfx.run_if(any_exist::<BlackHoleSfx>()),
                apply_pending_defender_heal.run_if(resource_exists::<PendingDefenderHeal>),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
