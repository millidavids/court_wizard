use bevy::prelude::*;

/// A tree obstacle on the battlefield. Impassable AND blocks all projectiles.
/// Destroyed only by fire spells. Persists between levels.
#[derive(Component)]
pub struct Tree {
    /// Center position in world space.
    pub center: Vec3,
    /// Collision radius on the XZ plane.
    pub radius: f32,
    /// Vertical extent for projectile collision checks.
    pub height: f32,
}

impl Tree {
    /// Checks if a point on the XZ plane is inside this tree's footprint.
    pub fn contains_point_xz(&self, point: Vec3) -> bool {
        let dx = point.x - self.center.x;
        let dz = point.z - self.center.z;
        (dx * dx + dz * dz) <= self.radius * self.radius
    }

    /// Returns `true` if this tree would block a projectile at the given position.
    pub fn blocks_projectile(&self, pos: Vec3) -> bool {
        self.contains_point_xz(pos) && pos.y <= self.height
    }

    /// Checks if a line segment (on XZ plane) intersects this tree.
    /// Returns the parametric t value (0..1) of the first intersection, if any.
    /// Used for line-of-sight checks (chain lightning, archer targeting).
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
            Some(0.0)
        } else {
            None
        }
    }

    /// Returns obstacle bounds as `[min_x, min_z, max_x, max_z]` for pathfinding.
    #[allow(dead_code)]
    pub fn obstacle_bounds(&self) -> [f32; 4] {
        [
            self.center.x - self.radius,
            self.center.z - self.radius,
            self.center.x + self.radius,
            self.center.z + self.radius,
        ]
    }

    /// Pushes a point outside the tree if it overlaps.
    #[allow(dead_code)]
    pub fn push_out(&self, point: Vec3, unit_radius: f32) -> Option<Vec3> {
        let dx = point.x - self.center.x;
        let dz = point.z - self.center.z;
        let dist = (dx * dx + dz * dz).sqrt();
        let required = self.radius + unit_radius;

        if dist >= required {
            return None;
        }

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

    /// Returns `true` if any tree in the slice blocks line-of-sight between two points.
    pub fn any_blocks_los(trees: &[&Self], from: Vec3, to: Vec3) -> bool {
        trees
            .iter()
            .any(|tree| tree.line_segment_intersects(from, to).is_some())
    }
}
