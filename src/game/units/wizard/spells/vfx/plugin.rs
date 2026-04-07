//! Shared visual effect plugin.

use bevy::prelude::*;

use super::components::{
    CastFlare, FireEmber, FireGlow, FireOrangeSmokePuff, FireSmoke, FireSpark, FloatingMote,
    HeatShimmer, MissileGlow, MissileSparkle, PlagueSmoke, SmokePoof,
};
use super::fire_material::FireParticleMaterial;
use super::systems;
use crate::game::run_conditions::{any_exist, is_gameplay_running, is_spell_effects_active};

/// Plugin that runs shared VFX update systems.
///
/// Spawning is handled by individual spell systems; this plugin only
/// updates and cleans up the visual entities.
pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MaterialPlugin::<FireParticleMaterial>::default(),
            MaterialPlugin::<super::fire_material::SmokeParticleMaterial>::default(),
        ));
        app.add_systems(
            Update,
            update_fire_particle_time.run_if(is_gameplay_running),
        );
        app.add_systems(
            Update,
            (
                (systems::update_fire_glow, systems::cleanup_orphaned_glows)
                    .chain()
                    .run_if(any_exist::<FireGlow>()),
                systems::update_fire_smoke.run_if(any_exist::<FireSmoke>()),
                systems::update_fire_sparks.run_if(any_exist::<FireSpark>()),
                systems::update_fire_embers.run_if(any_exist::<FireEmber>()),
                (
                    systems::update_missile_glow,
                    systems::cleanup_orphaned_missile_glows,
                )
                    .chain()
                    .run_if(any_exist::<MissileGlow>()),
                systems::update_missile_sparkles.run_if(any_exist::<MissileSparkle>()),
                systems::update_heat_shimmer.run_if(any_exist::<HeatShimmer>()),
                systems::update_plague_smoke.run_if(any_exist::<PlagueSmoke>()),
                systems::emit_fire_smoke_apex_puffs.run_if(any_exist::<FireOrangeSmokePuff>()),
                systems::update_cast_flares.run_if(any_exist::<CastFlare>()),
                systems::update_floating_motes.run_if(any_exist::<FloatingMote>()),
                systems::update_smoke_poofs.run_if(any_exist::<SmokePoof>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}

/// Updates the global time uniform on all fire particle materials each frame.
fn update_fire_particle_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<FireParticleMaterial>>,
) {
    let t = time.elapsed_secs();
    for (_id, material) in materials.iter_mut() {
        material.time = t;
    }
}
