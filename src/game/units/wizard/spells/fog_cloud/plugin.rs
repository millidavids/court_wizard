use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{BlindingMistDebuff, BlindingMistZone, ChokingFogZone, FogCloudZone, PhantomFogZone, PhantomUnit, RollingFogZone};
use super::systems;
use crate::game::run_conditions::{any_exist, is_spell_effects_active};

pub struct FogCloudPlugin;

impl Plugin for FogCloudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_fog_cloud_casting
                    .run_if(spell_is_primed(Spell::FogCloud))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                (
                    systems::apply_fog_cloud_evasion,
                    systems::emit_fog_cloud_particles,
                    systems::apply_blinding_mist
                        .run_if(any_exist::<BlindingMistZone>()),
                    systems::tick_blinding_mist_debuff
                        .run_if(any_exist::<BlindingMistDebuff>()),
                    systems::apply_choking_fog_damage
                        .run_if(any_exist::<ChokingFogZone>()),
                    systems::move_rolling_fog
                        .run_if(any_exist::<RollingFogZone>()),
                    systems::spawn_phantom_units
                        .run_if(any_exist::<PhantomFogZone>()),
                    systems::cleanup_fog_cloud_zone,
                )
                    .chain()
                    .run_if(any_exist::<FogCloudZone>()),
                systems::cleanup_phantom_units
                    .run_if(any_exist::<PhantomUnit>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
