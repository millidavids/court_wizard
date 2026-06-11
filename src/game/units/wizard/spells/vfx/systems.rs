//! Re-export hub for VFX systems split into feature files (Phase 17).
//!
//! Original 1568-line file split into:
//! - `fire_effects.rs` — fire glow, smoke wisps, sparks
//! - `explosion_effects.rs` — explosion smoke, missile glow/sparkles
//! - `area_effects.rs` — heat shimmer, plague/fog smoke, fire variants, embers
//! - `cast_effects.rs` — cast flares, motes, smoke poofs, dust, aura bubbles

pub use super::area_effects::*;
pub use super::cast_effects::*;
pub use super::explosion_effects::*;
pub use super::fire_effects::*;

use bevy::prelude::*;

use super::fire_material::FireParticleMaterial;

/// Updates the global time uniform on all fire particle materials each frame.
pub(super) fn update_fire_particle_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<FireParticleMaterial>>,
) {
    let t = time.elapsed_secs();
    for (_id, material) in materials.iter_mut() {
        material.time = t;
    }
}

/// Updates the global time uniform on all aura sphere materials each frame.
pub(super) fn update_aura_sphere_time(
    time: Res<Time>,
    mut materials: ResMut<
        Assets<crate::game::units::wizard::spells::visual_assets::AuraSphereMaterial>,
    >,
) {
    let t = time.elapsed_secs();
    for (_id, material) in materials.iter_mut() {
        material.time = t;
    }
}
