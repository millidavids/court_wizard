//! Multiplayer loading systems.
//!
//! Builds and processes a multiplayer-specific spawn queue. Both peers spawn
//! the full single-player battlefield and the same deterministic terrain
//! (driven by the host-shared seed), so each client sees the same world. The
//! host additionally enqueues all gameplay entities (kings, guards, infantry,
//! archers); the guest receives those via state snapshots.

mod cleanup;
mod init;
mod process;
mod queue;
mod resources;

pub use cleanup::{cleanup_mp_loading, restore_camera, setup_mp_camera};
pub use init::init_mp_loading;
pub use process::process_mp_spawn_queue;
