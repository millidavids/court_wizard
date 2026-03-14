use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{
    GreaseBubble, GreaseIgnited, GreaseRegenerating, GreaseSplatter,
    GreaseZone, GreaseZonePresenceTracker,
};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

pub struct GreasePlugin;

impl Plugin for GreasePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_grease_casting
                    .run_if(spell_is_primed(Spell::Grease))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                (
                    systems::apply_grease_slow,
                    systems::check_grease_ignition,
                    systems::fade_grease_zone,
                    systems::cleanup_grease_zone,
                    systems::cleanup_grease_debuffs
                        .run_if(any_with_component::<GreaseZonePresenceTracker>),
                )
                    .chain()
                    .run_if(any_exist::<GreaseZone>()),
                // Fire-specific systems — only run when ignited zones exist
                (
                    systems::update_grease_fire_spread,
                    systems::spawn_grease_fire_smoke,
                    systems::apply_grease_burn,
                )
                    .chain()
                    .run_if(any_exist::<GreaseIgnited>()),
                // Zone VFX — fumes, bubbles, splatters for non-ignited zones
                systems::spawn_grease_zone_vfx
                    .run_if(any_exist::<GreaseZone>()),
                // Bubble and splatter update systems
                systems::update_grease_bubbles
                    .run_if(any_exist::<GreaseBubble>()),
                systems::update_grease_splatters
                    .run_if(any_exist::<GreaseSplatter>()),
                // Endless Oil regeneration — only run when regenerating zones exist
                systems::update_grease_regeneration
                    .run_if(any_with_component::<GreaseRegenerating>),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
