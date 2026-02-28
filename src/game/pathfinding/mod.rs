//! Flow field pathfinding module.
//!
//! Provides grid-based pathfinding using flow fields to guide units toward goals
//! while avoiding obstacles and respecting terrain costs.

pub mod components;
pub mod debug;
pub mod flow_field;
pub mod messages;
pub mod plugin;
pub mod resources;
pub mod systems;

pub use components::{FlowFieldInfluence, FlowFieldVelocity};
pub use messages::{ObstacleChanged, ObstacleShape, ObstacleType};
pub use plugin::PathfindingPlugin;
pub(crate) use systems::OBSTACLE_BUFFER;
