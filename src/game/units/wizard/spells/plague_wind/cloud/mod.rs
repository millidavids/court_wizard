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
// spawn_plague_cloud was pub(super) in the original (visible only to plague_wind).
// It lives in spawn.rs as pub(super) there (visible to cloud/mod.rs), and we
// re-export it here with pub(super) so plague_wind can still call it.
pub(super) use spawn::spawn_plague_cloud;
