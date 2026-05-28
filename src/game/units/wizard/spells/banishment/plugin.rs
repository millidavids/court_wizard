use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::BanishmentVfx;
use super::systems;
use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};
use crate::game::units::components::BanishedModifier;

pub struct BanishmentPlugin;

impl Plugin for BanishmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            // Local wizard casting (mouse input)
            systems::handle_banishment_casting
                .run_if(spell_is_primed(Spell::Banishment))
                .run_if(spell_input_not_blocked)
                .run_if(mouse_left_not_consumed)
                .run_if(mouse_held_or_wizard_casting)
                .run_if(is_spell_effects_active),
        );
        // Gameplay-authoritative: ticks banishment expiry and respawns units.
        app.add_systems(
            Update,
            systems::tick_banished_units
                .run_if(any_exist::<BanishedModifier>())
                .run_if(is_gameplay_running),
        );

        // Visual-only: animates the lensing-sphere collapse on each peer.
        // Runs on both MP peers so the guest sees the host's banishment VFX
        // once the BanishmentVfx component is replicated (Phase 3).
        app.add_systems(
            Update,
            systems::update_banishment_vfx
                .run_if(any_exist::<BanishmentVfx>())
                .run_if(is_spell_effects_active),
        );
    }
}
