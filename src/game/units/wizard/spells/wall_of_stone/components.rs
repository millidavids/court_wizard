use bevy::prelude::*;

/// Active wall entity that blocks movement and projectiles.
#[derive(Component)]
pub struct WallOfStone {
    /// Center position of the wall in world space.
    pub center: Vec3,
    /// Half the wall length (along the drag direction).
    pub half_length: f32,
    /// Half the wall width (perpendicular to drag direction).
    pub half_width: f32,
    /// Normalized direction along the wall length (XZ plane).
    pub forward: Vec3,
    /// Normalized direction perpendicular to wall length (XZ plane).
    pub right: Vec3,
    /// Wall height for vertical collision checks.
    pub height: f32,
    /// Time this wall has been alive.
    pub time_alive: f32,
    /// Total lifetime before despawn.
    pub duration: f32,
    /// Whether the wall is currently sinking into the ground.
    pub sinking: bool,
}

impl WallOfStone {
    /// Checks if a point on the XZ plane is inside this wall's footprint.
    pub fn contains_point_xz(&self, point: Vec3) -> bool {
        let diff = Vec3::new(point.x - self.center.x, 0.0, point.z - self.center.z);
        let forward_proj = diff.dot(self.forward).abs();
        let right_proj = diff.dot(self.right).abs();
        forward_proj <= self.half_length && right_proj <= self.half_width
    }

    /// Checks if a line segment (on XZ plane) intersects this wall.
    /// Returns the parametric t value (0..1) of the first intersection, if any.
    pub fn line_segment_intersects(&self, start: Vec3, end: Vec3) -> Option<f32> {
        // Separating axis theorem on XZ plane for OBB vs line segment
        let dir = Vec3::new(end.x - start.x, 0.0, end.z - start.z);
        let to_start = Vec3::new(start.x - self.center.x, 0.0, start.z - self.center.z);

        // Test against forward axis
        let (t_min, t_max) = Self::slab_intersect(
            to_start.dot(self.forward),
            dir.dot(self.forward),
            self.half_length,
        )?;

        // Test against right axis
        let (t_min2, t_max2) = Self::slab_intersect(
            to_start.dot(self.right),
            dir.dot(self.right),
            self.half_width,
        )?;

        let t_enter = t_min.max(t_min2);
        let t_exit = t_max.min(t_max2);

        if t_enter <= t_exit && t_exit >= 0.0 && t_enter <= 1.0 {
            Some(t_enter.max(0.0))
        } else {
            None
        }
    }

    /// Pushes a point outside the wall along the nearest edge normal.
    /// Returns the corrected position if the point was inside.
    pub fn push_out(&self, point: Vec3, radius: f32) -> Option<Vec3> {
        let diff = Vec3::new(point.x - self.center.x, 0.0, point.z - self.center.z);
        let forward_proj = diff.dot(self.forward);
        let right_proj = diff.dot(self.right);

        let forward_pen = self.half_length + radius - forward_proj.abs();
        let right_pen = self.half_width + radius - right_proj.abs();

        if forward_pen <= 0.0 || right_pen <= 0.0 {
            return None; // Not overlapping
        }

        // Push along axis with least penetration
        if forward_pen < right_pen {
            let sign = forward_proj.signum();
            Some(Vec3::new(
                point.x + self.forward.x * forward_pen * sign,
                point.y,
                point.z + self.forward.z * forward_pen * sign,
            ))
        } else {
            let sign = right_proj.signum();
            Some(Vec3::new(
                point.x + self.right.x * right_pen * sign,
                point.y,
                point.z + self.right.z * right_pen * sign,
            ))
        }
    }

    fn slab_intersect(origin: f32, dir: f32, half_extent: f32) -> Option<(f32, f32)> {
        if dir.abs() < 1e-6 {
            // Ray parallel to slab
            if origin.abs() > half_extent {
                return None;
            }
            return Some((f32::NEG_INFINITY, f32::INFINITY));
        }
        let t1 = (-half_extent - origin) / dir;
        let t2 = (half_extent - origin) / dir;
        Some((t1.min(t2), t1.max(t2)))
    }
}

/// Component on the wizard tracking wall placement state.
#[derive(Component)]
pub struct WallOfStoneCaster {
    /// World position where the player first clicked.
    pub anchor: Option<Vec3>,
    /// Entity ID of the preview mesh.
    pub preview_entity: Option<Entity>,
}

impl WallOfStoneCaster {
    pub const fn new() -> Self {
        Self {
            anchor: None,
            preview_entity: None,
        }
    }
}

/// Marker component for the wall preview mesh shown during drag.
#[derive(Component)]
pub struct WallOfStonePreview;
