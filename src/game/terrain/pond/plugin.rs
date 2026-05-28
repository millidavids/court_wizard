use bevy::prelude::*;

use super::components::{
    ClonedPondMaterial, Pond, PondEvaporation, PondFogCloud, PondFrozen, PondShocked,
};
use super::resources;
use super::systems::*;
use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};
use crate::game::units::wizard::archetypes::meteorologist::components::WetModifier;

pub struct PondPlugin;

impl Plugin for PondPlugin {
    fn build(&self, app: &mut App) {
        // Gameplay-authoritative systems — host-only in MP. These insert
        // status components (WetModifier), deal damage (tick_pond_shocked),
        // emit ObstacleChanged messages (tick_pond_frozen), or modify
        // movement (apply_frozen_pond_slow). Running them on the guest would
        // double-apply effects or diverge the pathfinding grid.
        app.add_systems(Startup, resources::preload_pond_assets)
            .add_systems(
                Update,
                (
                    apply_pond_wet.run_if(any_with_component::<Pond>),
                    tick_wet_timer.run_if(any_with_component::<WetModifier>),
                    apply_frozen_pond_slow
                        .after(crate::game::movement_systems::apply_rough_terrain_slowdown)
                        .run_if(any_with_component::<PondFrozen>),
                    tick_pond_evaporation.run_if(any_with_component::<PondEvaporation>),
                    tick_pond_frozen.run_if(any_with_component::<PondFrozen>),
                    tick_pond_shocked.run_if(any_with_component::<PondShocked>),
                )
                    .run_if(is_gameplay_running),
            );

        // Visual-only systems — run on both MP peers so the guest sees pond
        // ripples, fog wisps, frost tint, and material restoration after
        // thaw. None of these mutate gameplay state.
        app.add_systems(
            Update,
            (
                emit_pond_ripples.run_if(any_with_component::<Pond>),
                emit_pond_fog_particles.run_if(any_with_component::<PondFogCloud>),
                update_frozen_pond_tint.run_if(any_with_component::<PondFrozen>),
                restore_pond_material_on_thaw.run_if(any_with_component::<ClonedPondMaterial>),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
