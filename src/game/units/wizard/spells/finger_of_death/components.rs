use bevy::prelude::*;

/// Marker component indicating wizard is waiting for mouse release before allowing another Finger of Death cast.
///
/// This prevents the spell from immediately recasting after completion if the mouse is still held.
#[derive(Component)]
pub struct AwaitingFingerOfDeathRelease;

/// Finger of Death beam component tracking the devastating instant-cast beam.
#[derive(Component)]
pub struct FingerOfDeathBeam {
    /// Beam starting position (origin point).
    pub origin: Vec3,
    /// Normalized direction vector.
    pub direction: Vec3,
    /// Full beam length.
    pub length: f32,
    /// Time since spawn (for growth animation and despawn timing).
    pub time_alive: f32,
    /// Cast progress (0.0 to 1.0).
    pub cast_progress: f32,
    /// Whether damage has been applied (fires once).
    pub has_fired: bool,
    /// Time since the beam fired (for fade out animation).
    pub time_since_fired: f32,
}

impl FingerOfDeathBeam {
    /// Creates a new Finger of Death beam.
    pub fn new(origin: Vec3, direction: Vec3, length: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            length,
            time_alive: 0.0,
            cast_progress: 0.0,
            has_fired: false,
            time_since_fired: 0.0,
        }
    }

    /// Checks if a point is within the beam.
    pub fn contains_point(&self, point: Vec3, beam_width: f32) -> bool {
        let to_point = point - self.origin;
        let projection_length = to_point.dot(self.direction);

        // Check if within beam length
        if projection_length < 0.0 || projection_length > self.length {
            return false;
        }

        // Check distance from centerline
        let projected_point = self.origin + self.direction * projection_length;
        let distance_from_centerline = point.distance(projected_point);

        distance_from_centerline <= beam_width
    }
}
