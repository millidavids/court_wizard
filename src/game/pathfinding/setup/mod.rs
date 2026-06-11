//! Pathfinding setup: grid initialization and static terrain registration.

mod grid_init;
mod stuck_detection;
mod wave_staging;

pub(crate) use grid_init::CELL_SIZE;
pub(crate) use grid_init::OBSTACLE_BUFFER;
pub use grid_init::{generate_initial_fields, initialize_pathfinding};
pub use stuck_detection::{detect_and_recover_stuck_units, init_stuck_detection};
pub use wave_staging::{
    WaveStagingTimers, check_wave_activation, manage_staging_speedup, suppress_staging_targeting,
    tag_new_attackers,
};
