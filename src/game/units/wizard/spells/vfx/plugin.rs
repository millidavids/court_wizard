//! Shared visual effect plugin.

use bevy::prelude::*;

use super::components::{
    FireGlow, FireSmoke, FireSpark, HeatShimmer, MissileGlow, MissileSparkle, PlagueSmoke,
};
use super::systems;
use crate::game::run_conditions::{any_exist, is_spell_effects_active};

/// Plugin that runs shared VFX update systems.
///
/// Spawning is handled by individual spell systems; this plugin only
/// updates and cleans up the visual entities.
pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (systems::update_fire_glow, systems::cleanup_orphaned_glows)
                    .chain()
                    .run_if(any_exist::<FireGlow>()),
                systems::update_fire_smoke.run_if(any_exist::<FireSmoke>()),
                systems::update_fire_sparks.run_if(any_exist::<FireSpark>()),
                (
                    systems::update_missile_glow,
                    systems::cleanup_orphaned_missile_glows,
                )
                    .chain()
                    .run_if(any_exist::<MissileGlow>()),
                systems::update_missile_sparkles.run_if(any_exist::<MissileSparkle>()),
                systems::update_heat_shimmer.run_if(any_exist::<HeatShimmer>()),
                systems::update_plague_smoke.run_if(any_exist::<PlagueSmoke>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
