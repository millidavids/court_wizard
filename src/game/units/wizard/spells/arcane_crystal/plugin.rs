//! Arcane Crystal spell plugin.

use bevy::prelude::*;

use super::components::{ArcaneCrystal, ArcaneCrystalCircleIndicator, CrystalSpawn};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::run_conditions::{
    any_exist, mouse_held_or_wizard_casting, mouse_left_not_consumed, spell_input_not_blocked,
    spell_is_primed,
};

/// Plugin for the Arcane Crystal spell.
pub struct ArcaneCrystalPlugin;

impl Plugin for ArcaneCrystalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_arcane_crystal_casting
                    .run_if(spell_is_primed(Spell::ArcaneCrystal))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                systems::update_circle_indicator
                    .run_if(any_exist::<ArcaneCrystalCircleIndicator>()),
                // Crystal lifetime & visuals
                systems::update_crystal_visuals.run_if(any_with_component::<ArcaneCrystal>),
                systems::cleanup_expired_crystals.run_if(any_with_component::<ArcaneCrystal>),
                // Black hole interaction
                systems::crystal_black_hole_interaction.run_if(any_with_component::<ArcaneCrystal>),
                // Spell absorption
                systems::detect_fireball_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_beam_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_meteor_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_magic_missile_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_chain_lightning_hits.run_if(any_with_component::<ArcaneCrystal>),
                // Auto-casting
                systems::auto_cast_remembered_spell.run_if(any_with_component::<ArcaneCrystal>),
                // Range-limiting
                systems::despawn_out_of_range_crystal_spawns
                    .run_if(any_with_component::<CrystalSpawn>),
            )
                .chain()
                .run_if(is_spell_effects_active),
        );
    }
}
