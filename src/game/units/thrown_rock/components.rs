use bevy::prelude::*;

/// A landed rock obstacle on the battlefield. Blocks movement and pathfinding.
#[derive(Component)]
pub struct ThrownRock {
    /// Center position in world space.
    pub center: Vec3,
    /// Collision radius on the XZ plane.
    pub radius: f32,
    /// Vertical extent for projectile collision checks.
    pub height: f32,
    /// Whether the rock is currently sinking into the ground.
    pub sinking: bool,
    /// Time this rock has been alive (for sink timing).
    pub time_alive: f32,
    /// When sinking starts, the time at which the rock should be despawned.
    pub sink_deadline: f32,
}

impl ThrownRock {
    /// Checks if a point on the XZ plane is inside this rock's footprint.
    pub fn contains_point_xz(&self, point: Vec3) -> bool {
        let dx = point.x - self.center.x;
        let dz = point.z - self.center.z;
        (dx * dx + dz * dz) <= self.radius * self.radius
    }

    /// Checks if a line segment (on XZ plane) intersects this rock.
    /// Returns the parametric t value (0..1) of the first intersection, if any.
    pub fn line_segment_intersects(&self, start: Vec3, end: Vec3) -> Option<f32> {
        let dx = end.x - start.x;
        let dz = end.z - start.z;
        let fx = start.x - self.center.x;
        let fz = start.z - self.center.z;

        let a = dx * dx + dz * dz;
        let b = 2.0 * (fx * dx + fz * dz);
        let c = fx * fx + fz * fz - self.radius * self.radius;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrt_disc = discriminant.sqrt();
        let t1 = (-b - sqrt_disc) / (2.0 * a);
        let t2 = (-b + sqrt_disc) / (2.0 * a);

        if t1 >= 0.0 && t1 <= 1.0 {
            Some(t1)
        } else if t2 >= 0.0 && t2 <= 1.0 {
            Some(t2)
        } else if t1 < 0.0 && t2 > 1.0 {
            // Segment entirely inside circle
            Some(0.0)
        } else {
            None
        }
    }

    /// Pushes a point outside the rock if it overlaps.
    /// Returns the corrected position if the point was inside.
    pub fn push_out(&self, point: Vec3, unit_radius: f32) -> Option<Vec3> {
        let dx = point.x - self.center.x;
        let dz = point.z - self.center.z;
        let dist = (dx * dx + dz * dz).sqrt();
        let required = self.radius + unit_radius;

        if dist >= required {
            return None;
        }

        // If exactly at center, push in arbitrary direction
        if dist < 0.001 {
            return Some(Vec3::new(
                self.center.x + required,
                point.y,
                self.center.z,
            ));
        }

        let scale = required / dist;
        Some(Vec3::new(
            self.center.x + dx * scale,
            point.y,
            self.center.z + dz * scale,
        ))
    }

    /// Returns the XZ-plane distance from a point to the rock's surface.
    pub fn distance_to_surface(&self, point: Vec3) -> f32 {
        let dx = point.x - self.center.x;
        let dz = point.z - self.center.z;
        let dist_to_center = (dx * dx + dz * dz).sqrt();
        (dist_to_center - self.radius).max(0.0)
    }

    /// Returns obstacle bounds as `[min_x, min_z, max_x, max_z]` for pathfinding.
    pub fn obstacle_bounds(&self) -> [f32; 4] {
        [
            self.center.x - self.radius,
            self.center.z - self.radius,
            self.center.x + self.radius,
            self.center.z + self.radius,
        ]
    }

    /// Returns `true` if this rock would block a projectile at the given position.
    pub fn blocks_projectile(&self, pos: Vec3) -> bool {
        !self.sinking && self.contains_point_xz(pos) && pos.y <= self.height
    }

    /// Returns `true` if any rock in the slice blocks line-of-sight between two points.
    pub fn any_blocks_los(rocks: &[&Self], from: Vec3, to: Vec3) -> bool {
        rocks
            .iter()
            .any(|rock| rock.line_segment_intersects(from, to).is_some())
    }
}

/// Shadow entity that follows a rock. Despawned when the rock is destroyed.
#[derive(Component)]
pub struct RockShadow {
    pub owner: Entity,
}

/// A rock projectile in flight (parabolic arc from thrower to target).
#[derive(Component)]
pub struct RockProjectile {
    /// Starting position (thrower's position).
    pub start: Vec3,
    /// Target landing position.
    pub target: Vec3,
    /// Total flight duration.
    pub duration: f32,
    /// Time elapsed since launch.
    pub elapsed: f32,
    /// Peak height of the arc above ground.
    pub arc_height: f32,
}

impl RockProjectile {
    /// Returns the current interpolated position along the parabolic arc.
    pub fn current_position(&self) -> Vec3 {
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        let xz = self.start.lerp(self.target, t);
        // Parabolic arc: y = 4 * h * t * (1 - t)
        let y = 4.0 * self.arc_height * t * (1.0 - t);
        Vec3::new(xz.x, y, xz.z)
    }

    pub fn is_landed(&self) -> bool {
        self.elapsed >= self.duration
    }
}
