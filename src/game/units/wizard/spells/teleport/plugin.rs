//! Plugin for the Teleport spell.

use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{
    DimensionalRift, DisorientingHaste, LingeringGateMarker, RiftCooldown,
    TeleportDestinationCircle, TeleportSourceCircle,
};
use super::systems;
use super::vfx_components::TeleportWarpEffect;
use super::vfx_systems;
use crate::game::run_conditions::{any_exist, is_gameplay_running, is_spell_effects_active};
use crate::game::units::systems::update_timed_modifier;

/// Plugin that handles the Teleport spell.
///
/// Registers systems for:
/// - Two-phase casting (destination placement, then source placement)
/// - Circle animations (pulsing effects)
/// - Unit teleportation
/// - Talent effects (stun, haste, rift, lingering gate)
pub struct TeleportPlugin;

impl Plugin for TeleportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_teleport_cancel.run_if(spell_is_primed(Spell::Teleport)),
                // Local wizard casting (mouse input)
                systems::handle_teleport_casting
                    .run_if(spell_is_primed(Spell::Teleport))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_right_not_held)
                    .run_if(mouse_held_or_wizard_casting),
                systems::update_circle_animations.run_if(
                    any_exist::<TeleportDestinationCircle>()
                        .or(any_exist::<TeleportSourceCircle>()),
                ),
                systems::cleanup_teleport_on_spell_switch.run_if(
                    any_exist::<TeleportDestinationCircle>()
                        .or(any_exist::<TeleportSourceCircle>()),
                ),
            )
                .run_if(is_spell_effects_active),
        );

        // Talent behavior systems
        // Note: Stunned timer is handled by UnitsPlugin (general-purpose component)
        app.add_systems(
            Update,
            update_timed_modifier::<DisorientingHaste>
                .run_if(is_gameplay_running)
                .run_if(any_with_component::<DisorientingHaste>),
        );

        app.add_systems(
            Update,
            update_timed_modifier::<RiftCooldown>
                .run_if(is_gameplay_running)
                .run_if(any_with_component::<RiftCooldown>),
        );

        app.add_systems(
            Update,
            systems::tick_dimensional_rift
                .run_if(is_gameplay_running)
                .run_if(any_with_component::<DimensionalRift>),
        );

        app.add_systems(
            Update,
            systems::tick_lingering_gate
                .run_if(is_gameplay_running)
                .run_if(any_with_component::<LingeringGateMarker>),
        );

        // VFX systems
        app.add_systems(
            Update,
            (
                vfx_systems::tick_warp_effects,
                vfx_systems::cleanup_rift_warp_effects,
            )
                .run_if(is_spell_effects_active)
                .run_if(any_with_component::<TeleportWarpEffect>),
        );
    }
}
