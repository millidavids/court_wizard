mod bolt;
mod channel;
mod channel_vfx;
mod targeting;
mod targeting_helpers;

// Re-export pub(super) constants from healer::constants so sub-modules can reach them.
// (pub(super) in constants.rs is visible here because our super == healer.)
pub(super) use super::constants::{
    HEALER_CAST_DURATION, HEALER_CHANNEL_PARTICLE_COUNT_PER_SPAWN,
    HEALER_CHANNEL_PARTICLE_LIFETIME, HEALER_CHANNEL_PARTICLE_MAX_RADIUS,
    HEALER_CHANNEL_PARTICLE_SIZE, HEALER_CHANNEL_PARTICLE_SPAWN_INTERVAL,
    HEALER_CHANNEL_PARTICLE_START_RADIUS,
};

pub use bolt::{check_heal_bolt_arrivals, move_heal_bolts};
pub use channel::{healer_start_heal_channel, healer_tick_heal_channel};
pub use channel_vfx::{healer_refresh_casting_animation, healer_spawn_channel_particles};
pub use targeting::{healer_movement, update_healer_targeting};
