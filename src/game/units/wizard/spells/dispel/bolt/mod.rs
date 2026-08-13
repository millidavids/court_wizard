mod impacts;
mod multiplayer;
mod null_zone;
mod suppress;

pub(crate) use impacts::announce_dispel_wave;
pub use impacts::update_dispel_impacts;
pub use multiplayer::{forward_dispel_impacts_to_owner, tick_ghost_dispel_impacts};
pub use null_zone::update_null_zones;
pub(crate) use suppress::{is_dispellable, spell_edge_distance};
