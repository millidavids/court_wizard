//! Squall spell plugin.

use bevy::prelude::*;

use super::components::{
    AbsoluteZeroSlow, FrozenGround, IceExplosion, IceProjectile, SnowParticle,
    SquallStorm, SquallStormRing,
};
use super::systems::*;
use crate::game::run_conditions::{any_exist, is_spell_effects_active};
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::run_conditions::*;

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
                // Storm systems (spawn projectiles, update physics, check collisions)
                spawn_ice_projectiles.run_if(any_exist::<SquallStorm>()),
                update_ice_projectiles.run_if(any_exist::<IceProjectile>()),
                check_ice_projectile_collisions.run_if(any_exist::<IceProjectile>()),
                // Explosion updates
                update_ice_explosions.run_if(any_exist::<IceExplosion>()),
                // Storm ring reticle
                update_storm_ring.run_if(any_exist::<SquallStormRing>()),
                // Talent systems
                apply_sleet_storm_evasion.run_if(any_exist::<SquallStorm>()),
                update_absolute_zero.run_if(any_exist::<SquallStorm>()),
                end_absolute_zero_on_release.run_if(any_exist::<SquallStorm>()),
                decay_absolute_zero_slow.run_if(any_exist::<AbsoluteZeroSlow>()),
                update_blizzard_position.run_if(any_exist::<SquallStorm>()),
                update_frozen_ground.run_if(any_exist::<FrozenGround>()),
                // Snow VFX
                spawn_snow_particles.run_if(any_exist::<SquallStorm>()),
                update_snow_particles.run_if(any_exist::<SnowParticle>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
