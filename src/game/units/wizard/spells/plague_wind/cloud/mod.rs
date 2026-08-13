//! Plague wind cloud: spawn, movement, damage application, cleanup.

mod damage;
mod drift;
mod particles;
mod spawn;

pub use damage::{
    apply_plague_carrier_dot, apply_plague_wind_damage, cleanup_toxic_weakness,
    track_plague_carrier,
};
pub use drift::{cleanup_plague_wind_cloud, move_plague_wind_cloud};
pub use particles::emit_plague_cloud_particles;
pub use spawn::spawn_pandemic_clouds;
// Widened from pub(super) for the Arcane Crystal's Plague Wind infusion, which
// emits the same drifting clouds radially from the crystal.
pub(crate) use spawn::spawn_plague_cloud;
