//! Pathfinding messages for obstacle changes.

use bevy::prelude::*;

/// Message sent when an obstacle appears, changes, or is removed.
#[derive(Message)]
pub struct ObstacleChanged {
    /// World-space AABB bounds (broadphase). All affected cells lie within this rect.
    pub bounds: Rect,
    /// Type of obstacle change.
    pub obstacle_type: ObstacleType,
    /// Actual shape for per-cell narrowphase testing.
    /// When `None`, every cell in `bounds` is affected (legacy AABB behavior).
    pub shape: Option<ObstacleShape>,
    /// When true, triggers a full async flow field rebuild even for non-Blocked obstacles.
    /// Use for hazards that should cause units to reroute (e.g. Wall of Fire).
    pub rebuild: bool,
}

/// Types of obstacle changes.
#[derive(Clone, Copy, Debug)]
pub enum ObstacleType {
    /// Cell is completely blocked (walls, boulders).
    Blocked,
    /// Cell has increased movement cost (mud, water).
    /// The f32 value is the cost multiplier (e.g., 3.0 = 3x slower).
    #[allow(dead_code)]
    SlowTerrain(f32),
    /// Cell is a hazard (fire, poison) with increased movement cost.
    /// The f32 value is the cost multiplier — higher values make units avoid more strongly.
    /// Units will still path through if the detour is too long.
    Hazard(f32),
    /// Obstacle was removed, reset to normal terrain.
    Removed,
}

/// Actual obstacle shape for precise per-cell intersection testing.
#[derive(Clone, Copy, Debug)]
pub enum ObstacleShape {
    /// Circle defined by center (XZ as Vec2) and radius.
    Circle { center: Vec2, radius: f32 },
    /// Oriented bounding box defined by center, half-extents, and local axes.
    Obb {
        center: Vec2,
        half_length: f32,
        half_width: f32,
        /// Normalized forward direction (along length axis) in XZ plane.
        forward: Vec2,
    },
}

impl ObstacleShape {
    /// Creates a circle shape from a world-space XZ center and radius.
    pub fn circle(center_xz: Vec2, radius: f32) -> Self {
        Self::Circle {
            center: center_xz,
            radius,
        }
    }

    /// Creates an OBB shape from a wall defined by start/end points and half-width.
    pub fn obb_from_wall(start: Vec3, end: Vec3, half_width: f32) -> Self {
        let a = Vec2::new(start.x, start.z);
        let b = Vec2::new(end.x, end.z);
        let dir = b - a;
        let half_length = dir.length() * 0.5;
        let forward = dir.normalize_or_zero();
        let center = (a + b) * 0.5;
        Self::Obb {
            center,
            half_length,
            half_width,
            forward,
        }
    }

    /// Creates an OBB shape from a `WallOfStone`-style component (center, forward, half-extents).
    pub fn obb_from_center(center: Vec3, forward: Vec3, half_length: f32, half_width: f32) -> Self {
        Self::Obb {
            center: Vec2::new(center.x, center.z),
            half_length,
            half_width,
            forward: Vec2::new(forward.x, forward.z).normalize_or_zero(),
        }
    }

    /// Tests whether a grid cell (given its center in world XZ) intersects this shape.
    /// Uses cell half-size for box-vs-shape overlap testing.
    pub fn intersects_cell(&self, cell_center: Vec2, cell_half_size: f32) -> bool {
        match *self {
            ObstacleShape::Circle { center, radius } => {
                // Circle vs AABB: find closest point on cell to circle center
                let closest = Vec2::new(
                    center.x.clamp(
                        cell_center.x - cell_half_size,
                        cell_center.x + cell_half_size,
                    ),
                    center.y.clamp(
                        cell_center.y - cell_half_size,
                        cell_center.y + cell_half_size,
                    ),
                );
                closest.distance_squared(center) <= radius * radius
            }
            ObstacleShape::Obb {
                center,
                half_length,
                half_width,
                forward,
            } => {
                // Separating axis theorem: OBB vs axis-aligned cell
                let right = Vec2::new(-forward.y, forward.x);
                let diff = cell_center - center;

                // Test 4 axes: cell's X, cell's Y, OBB forward, OBB right
                let axes = [Vec2::X, Vec2::Y, forward, right];

                for axis in axes {
                    let cell_proj = cell_half_size * axis.x.abs() + cell_half_size * axis.y.abs();
                    let obb_proj =
                        half_length * forward.dot(axis).abs() + half_width * right.dot(axis).abs();
                    let dist = diff.dot(axis).abs();

                    if dist > cell_proj + obb_proj {
                        return false; // Separating axis found
                    }
                }
                true // No separating axis — overlapping
            }
        }
    }
}
