use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{Fireball, FireballExplosion};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

/// Plugin that handles fireball spell casting and behavior.
pub struct FireballPlugin;

impl Plugin for FireballPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_fireball_casting
                    .run_if(spell_is_primed(Spell::Fireball))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                (
                    systems::move_fireballs,
                    systems::spawn_fireball_smoke_trail,
                    systems::check_fireball_collisions,
                    systems::despawn_distant_fireballs,
                )
                    .chain()
                    .run_if(any_exist::<Fireball>()),
                (
                    systems::update_explosions,
                    systems::apply_explosion_damage,
                    systems::cleanup_finished_explosions,
                )
                    .chain()
                    .run_if(any_exist::<FireballExplosion>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
