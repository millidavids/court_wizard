//! Spawn-queue processing (per-frame batched entity spawn).

mod cleanup;
mod processor;

pub use cleanup::cleanup_loading_progress;
pub use processor::process_spawn_queue;
