use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{FogCloudIndicator, FogCloudZone};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

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
                systems::update_fog_cloud_indicator.run_if(any_exist::<FogCloudIndicator>()),
                (
                    systems::apply_fog_cloud_evasion,
                    systems::fade_fog_cloud_zone,
                    systems::cleanup_fog_cloud_zone,
                )
                    .chain()
                    .run_if(any_exist::<FogCloudZone>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
