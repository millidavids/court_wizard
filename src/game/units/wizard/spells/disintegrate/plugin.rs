use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{
    BeamGlow, BeamOriginFlare, BeamSmoke, DisintegrateBeam, DisintegrateParticle,
};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;
use crate::game::units::wizard::spells::audio::ChannelingSfx;

/// Plugin that handles disintegrate spell casting and behavior.
pub struct DisintegratePlugin;

impl Plugin for DisintegratePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_disintegrate_casting
                    .run_if(spell_is_primed(Spell::Disintegrate))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting)
                    .run_if(is_spell_effects_active),
                (
                    systems::update_beam_visuals,
                    systems::update_beam_glow,
                    systems::update_beam_origin_flare,
                    systems::spawn_impact_particles,
                    systems::spawn_beam_smoke,
                    systems::apply_disintegrate_damage,
                    systems::cleanup_beams_on_cancel,
                )
                    .chain()
                    .run_if(
                        any_exist::<DisintegrateBeam>()
                            .or(any_exist::<BeamGlow>())
                            .or(any_exist::<BeamOriginFlare>())
                            .or(any_exist::<ChannelingSfx>()),
                    )
                    .run_if(is_spell_effects_active),
                systems::update_impact_particles
                    .run_if(any_exist::<DisintegrateParticle>())
                    .run_if(is_spell_effects_active),
                systems::update_beam_smoke
                    .run_if(any_exist::<BeamSmoke>())
                    .run_if(is_spell_effects_active),
            ),
        );
    }
}
