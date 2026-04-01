use bevy::prelude::*;

/// A pond on the battlefield. Slows units passing through. Indestructible.
#[derive(Component)]
pub struct Pond {
    /// Center position in world space (Y = 0).
    pub center: Vec3,
    /// Radius of the pond on the XZ plane.
    pub radius: f32,
    /// Timer for ripple emission.
    pub ripple_timer: f32,
}

impl Pond {
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
}
