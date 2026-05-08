//! Flow field pathfinding module.
//!
//! Provides grid-based pathfinding using flow fields to guide units toward goals
//! while avoiding obstacles and respecting terrain costs.

pub(crate) mod components;
#[cfg(debug_assertions)]
pub(crate) mod debug;
pub(crate) mod flow_field;
pub(crate) mod messages;
mod plugin;
pub(crate) mod resources;
pub(crate) mod runtime;
pub(crate) mod setup;
pub(crate) mod staging;
pub(crate) mod systems;

pub use components::{FlowFieldInfluence, FlowFieldVelocity, StagingAttacker, WaveGroup};
pub use messages::{ObstacleChanged, ObstacleShape, ObstacleType};
pub use plugin::PathfindingPlugin;
pub(crate) use systems::OBSTACLE_BUFFER;
