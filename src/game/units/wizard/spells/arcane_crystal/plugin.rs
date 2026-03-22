//! Arcane Crystal spell plugin.

use bevy::prelude::*;

use super::components::{ArcaneCrystal, AutoCrystalTimer, CrystalNetwork, CrystalRangeIndicator, CrystalSpawn, ResonanceCascade};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::run_conditions::{
    mouse_held_or_wizard_casting, mouse_left_not_consumed, spell_input_not_blocked,
    spell_is_primed,
};

/// Plugin for the Arcane Crystal spell.
pub struct ArcaneCrystalPlugin;

impl Plugin for ArcaneCrystalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Clear one-shot absorption flags before detection runs
                systems::clear_absorption_flags.run_if(any_with_component::<ArcaneCrystal>),
                // Local wizard casting (mouse input)
                systems::handle_arcane_crystal_casting
                    .run_if(spell_is_primed(Spell::ArcaneCrystal))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                // Crystal lifetime & visuals
                systems::update_crystal_visuals.run_if(any_with_component::<ArcaneCrystal>),
                systems::cleanup_expired_crystals.run_if(
                    any_with_component::<ArcaneCrystal>.or(any_with_component::<CrystalRangeIndicator>)
                ),
                // Black hole interaction
                systems::crystal_black_hole_interaction.run_if(any_with_component::<ArcaneCrystal>),
                // Spell absorption
                systems::detect_fireball_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_beam_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_meteor_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_magic_missile_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_chain_lightning_hits.run_if(any_with_component::<ArcaneCrystal>),
                // Talent: Resonance Cascade burst
                systems::resonance_cascade_burst.run_if(any_with_component::<ResonanceCascade>),
                // Talent: Crystal Network chaining
                systems::crystal_network_chain.run_if(any_with_component::<CrystalNetwork>),
                // Auto-casting
                systems::auto_cast_remembered_spell.run_if(any_with_component::<ArcaneCrystal>),
                // Talent: Auto-Crystal firing
                systems::auto_crystal_fire.run_if(any_with_component::<AutoCrystalTimer>),
                // Range-limiting & lifetime cleanup
                systems::despawn_out_of_range_crystal_spawns
                    .run_if(any_with_component::<CrystalSpawn>),
                systems::cleanup_expired_crystal_visuals
                    .run_if(any_with_component::<CrystalSpawn>),
                systems::cleanup_expired_crystal_beams.run_if(any_with_component::<CrystalSpawn>),
            )
                .chain()
                .run_if(is_spell_effects_active),
        );
    }
}
