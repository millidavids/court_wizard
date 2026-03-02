//! Lightning Rod spell plugin.

use bevy::prelude::*;

use super::components::{
    LightningRod, LightningRodArc, LightningRodCircleIndicator, LightningStrike,
};
use super::systems::*;
use crate::game::run_conditions::is_spell_effects_active;
use crate::game::units::wizard::spells::utils;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::run_conditions::*;

/// Plugin for the Lightning Rod spell.
///
/// Handles spell casting, tower placement, lightning strikes, and arc damage.
pub struct LightningRodPlugin;

impl Plugin for LightningRodPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                handle_lightning_rod_casting
                    .run_if(spell_is_primed(Spell::LightningRod))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                // Circle indicator updates
                utils::update_circle_indicator::<LightningRodCircleIndicator>.run_if(any_exist::<LightningRodCircleIndicator>()),
                // Tower systems
                update_lightning_rod.run_if(any_exist::<LightningRod>()),
                // Strike systems
                update_lightning_strikes.run_if(any_exist::<LightningStrike>()),
                // Arc visual updates
                update_lightning_rod_arcs.run_if(any_exist::<LightningRodArc>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
