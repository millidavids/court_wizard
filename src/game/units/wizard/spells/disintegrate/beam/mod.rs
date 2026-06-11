//! Disintegrate beam spawning, visuals, and cleanup.

mod particles;
mod searing_finale;
mod spawn;
mod visuals;

// pub(crate) items — used by arcane_crystal and multiplayer modules
pub(crate) use particles::{emit_beam_smoke, emit_impact_particles};
pub(crate) use spawn::spawn_beam_with_damage;

// pub items — registered as systems in plugin.rs (via systems.rs pub use beam::*)
pub use particles::{
    spawn_beam_smoke, spawn_impact_particles, update_beam_smoke, update_impact_particles,
};
pub use searing_finale::update_searing_finale_detonations;
pub use visuals::{
    update_beam_eclipse, update_beam_glow, update_beam_origin_flare, update_beam_visuals,
    update_sweep_beams,
};

// pub(super) items — used within the disintegrate module (casting.rs)
pub(super) use spawn::spawn_searing_finale;
pub(super) use spawn::{despawn_all_beam_visuals, spawn_beam_with_talents};
