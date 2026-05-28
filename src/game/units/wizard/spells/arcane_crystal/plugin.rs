//! Arcane Crystal spell plugin.

use bevy::prelude::*;

use super::components::{
    ArcaneCrystal, AutoCrystalTimer, CrystalNetwork, CrystalRangeIndicator, CrystalSpawn,
    ResonanceCascade,
};
use super::systems;
use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::run_conditions::{
    mouse_held_or_wizard_casting, mouse_left_not_consumed, spell_input_not_blocked, spell_is_primed,
};

/// Plugin for the Arcane Crystal spell.
pub struct ArcaneCrystalPlugin;

impl Plugin for ArcaneCrystalPlugin {
    fn build(&self, app: &mut App) {
        // Visual / lifetime systems — safe on both MP peers. The ghost
        // crystal exists on the guest and needs its visuals updated +
        // cleanup when the host's authoritative entity expires.
        app.add_systems(
            Update,
            (
                systems::clear_absorption_flags.run_if(any_with_component::<ArcaneCrystal>),
                systems::handle_arcane_crystal_casting
                    .run_if(spell_is_primed(Spell::ArcaneCrystal))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                systems::update_crystal_visuals.run_if(any_with_component::<ArcaneCrystal>),
                systems::cleanup_expired_crystals.run_if(
                    any_with_component::<ArcaneCrystal>
                        .or(any_with_component::<CrystalRangeIndicator>),
                ),
                systems::despawn_out_of_range_crystal_spawns
                    .run_if(any_with_component::<CrystalSpawn>),
                systems::cleanup_expired_crystal_visuals.run_if(any_with_component::<CrystalSpawn>),
                systems::cleanup_expired_crystal_beams.run_if(any_with_component::<CrystalSpawn>),
            )
                .chain()
                .run_if(is_spell_effects_active),
        );

        // Gameplay / hit-detection / talent systems — HOST-ONLY in MP.
        // These require `ResMut<BattleTalentProgress>` (which only exists
        // host-side) and mutate authoritative damage / talent progress;
        // running them on the guest would crash on the missing resource
        // AND double-apply damage on the ghost crystal. Their crystal
        // queries are also gated `Without<GhostSpellEffect>` for
        // defence-in-depth.
        app.add_systems(
            Update,
            (
                systems::crystal_black_hole_interaction.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_fireball_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_beam_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_meteor_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_magic_missile_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_chain_lightning_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::resonance_cascade_burst.run_if(any_with_component::<ResonanceCascade>),
                systems::crystal_network_chain.run_if(any_with_component::<CrystalNetwork>),
                systems::auto_cast_remembered_spell.run_if(any_with_component::<ArcaneCrystal>),
                systems::auto_crystal_fire.run_if(any_with_component::<AutoCrystalTimer>),
            )
                .run_if(is_gameplay_running),
        );
    }
}
