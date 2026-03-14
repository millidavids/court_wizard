use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::super::utils::{AnimatedRingParticle, animate_ring_particles};
use super::components::{
    EntangleGroundEffect, EntangleRooted, EntangleVine, ThornyVines,
};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

pub struct EntanglePlugin;

impl Plugin for EntanglePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_entangle_casting
                    .run_if(spell_is_primed(Spell::Entangle))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                (
                    systems::tick_entangle_ground_effect,
                    systems::cleanup_entangle_ground_effect,
                )
                    .chain()
                    .run_if(any_exist::<EntangleGroundEffect>()),
                // Vine VFX: static arches + animated ring particles
                systems::animate_entangle_vines
                    .run_if(any_exist::<EntangleVine>()),
                systems::emit_animated_vine_rings
                    .run_if(any_exist::<EntangleGroundEffect>()),
                animate_ring_particles
                    .run_if(any_exist::<AnimatedRingParticle>()),
                // Thorny Vines: DPS to rooted enemies
                systems::thorny_vines_tick
                    .run_if(any_exist::<ThornyVines>()),
                // Other talent effects — only run when EntangleRooted units exist
                (
                    systems::nourishing_roots_mana_regen,
                    systems::handle_entangle_root_expire,
                )
                    .run_if(any_exist::<EntangleRooted>()),
                // Overgrowth — only run when ground effects exist
                systems::overgrowth_root_new_units
                    .run_if(any_exist::<EntangleGroundEffect>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
