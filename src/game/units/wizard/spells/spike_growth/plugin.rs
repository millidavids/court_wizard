use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::super::utils::{AnimatedRingParticle, animate_ring_particles, update_circle_indicator};
use super::components::{
    SpikeGrowthIndicator, SpikeGrowthLingeringPoison, SpikeGrowthZone,
    SpikeStormProjectile,
};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

pub struct SpikeGrowthPlugin;

impl Plugin for SpikeGrowthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_spike_growth_casting
                    .run_if(spell_is_primed(Spell::SpikeGrowth))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                update_circle_indicator::<SpikeGrowthIndicator>
                    .run_if(any_exist::<SpikeGrowthIndicator>()),
                (
                    systems::apply_spike_growth_damage,
                    systems::update_death_garden,
                    systems::cleanup_spike_growth_zone,
                )
                    .chain()
                    .run_if(any_exist::<SpikeGrowthZone>()),
                // Lingering poison tick (runs independently of zones)
                systems::tick_lingering_poison
                    .run_if(any_exist::<SpikeGrowthLingeringPoison>()),
                // VFX: animated ring particles
                systems::emit_spike_growth_rings
                    .run_if(any_exist::<SpikeGrowthZone>()),
                animate_ring_particles
                    .run_if(any_exist::<AnimatedRingParticle>()),
                // Spike Storm: volley and projectile update
                systems::spike_storm_volley
                    .run_if(any_exist::<SpikeGrowthZone>()),
                systems::update_spike_storm_projectiles
                    .run_if(any_exist::<SpikeStormProjectile>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
