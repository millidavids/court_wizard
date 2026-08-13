mod casting;
mod effects;
mod helpers;
mod phantoms;

pub use casting::handle_fog_cloud_casting;
pub use effects::{
    apply_blinding_mist, apply_choking_fog_damage, apply_fog_cloud_evasion, cleanup_fog_cloud_zone,
    emit_fog_cloud_particles, move_rolling_fog, tick_blinding_mist_debuff,
};
// `spawn_fog_cloud_zone` is widened for the Arcane Crystal's Fog Cloud infusion.
pub(crate) use helpers::{is_in_fog_zone, spawn_fog_cloud_zone};
pub use phantoms::{cleanup_phantom_units, spawn_phantom_units};
