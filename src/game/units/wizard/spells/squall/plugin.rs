//! Squall spell plugin.

use bevy::prelude::*;

use super::components::SquallCircleIndicator;
use super::systems::*;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::run_conditions::*;
use crate::game::run_conditions::is_spell_effects_active;

/// Plugin for the Squall spell.
///
/// Handles spell casting, ice projectile spawning, physics, and explosions.
pub struct SquallPlugin;

impl Plugin for SquallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                handle_squall_casting
                    .run_if(spell_is_primed(Spell::Squall))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                // Guest wizard casting (network signals)
                handle_squall_casting_guest
                    .run_if(guest_spell_is_primed(Spell::Squall))
                    .run_if(guest_input_or_wizard_casting),
                // Circle indicator updates
                update_circle_indicator.run_if(any_exist::<SquallCircleIndicator>()),
                // Storm systems (spawn projectiles, update physics, check collisions)
                spawn_ice_projectiles,
                update_ice_projectiles,
                check_ice_projectile_collisions,
                // Explosion updates
                update_ice_explosions,
            )
                .run_if(is_spell_effects_active),
        );
    }
}
