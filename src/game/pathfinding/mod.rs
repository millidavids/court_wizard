//! Flow field pathfinding module.
//!
//! Provides grid-based pathfinding using flow fields to guide units toward goals
//! while avoiding obstacles and respecting terrain costs.

pub mod components;
pub mod events;
pub mod flow_field;
pub mod plugin;
pub mod resources;
pub mod systems;

pub use components::{FlowFieldInfluence, FlowFieldType};
pub use events::{ObstacleChanged, ObstacleType};
pub use plugin::PathfindingPlugin;
pub use resources::PathfindingGrid;
