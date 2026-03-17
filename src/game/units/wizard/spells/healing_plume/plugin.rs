use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{
    CleansingPlumeZone, FieldMedicConverted, FontOfLifePending, FontOfLifeZone,
    HealingPlumeZone, HealingRainZone,
};
use super::systems;
use crate::game::run_conditions::{any_exist, is_spell_effects_active};

pub struct HealingPlumePlugin;

impl Plugin for HealingPlumePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_healing_plume_casting
                    .run_if(spell_is_primed(Spell::HealingPlume))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                // Zone effects (only when zones exist)
                (
                    systems::apply_healing_plume_heal,
                    systems::fade_healing_plume_zone,
                    // Tier 2: Cleansing Plume
                    systems::apply_cleansing_plume
                        .run_if(any_exist::<CleansingPlumeZone>()),
                    // Tier 3: Font of Life — detect deaths in zone
                    systems::font_of_life_detect_deaths
                        .run_if(any_exist::<FontOfLifeZone>()),
                    // Tier 3: Healing Rain — move zone toward cursor
                    systems::move_healing_rain_zones
                        .run_if(any_exist::<HealingRainZone>()),
                    // Cleanup expired zones
                    systems::cleanup_healing_plume_zone,
                )
                    .chain()
                    .run_if(any_exist::<HealingPlumeZone>()),
                // Systems that must run even after zones are gone
                // Tier 3: Font of Life — process pending resurrections
                systems::font_of_life_resurrect
                    .run_if(any_exist::<FontOfLifePending>()),
                // Tier 3: Field Medic — revert when zone expires
                systems::field_medic_cleanup
                    .run_if(any_exist::<FieldMedicConverted>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
