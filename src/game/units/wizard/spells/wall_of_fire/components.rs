use bevy::prelude::*;

use crate::game::units::DamageType;

/// Persistent line-shaped fire effect on the ground.
///
/// Deals periodic damage to all units within its rectangular area.
/// Uses point-to-line-segment distance for damage checks.
#[derive(Component)]
pub struct WallOfFireEffect {
    /// Line segment start (XZ plane).
    pub start: Vec3,
    /// Line segment end (XZ plane).
    pub end: Vec3,
    /// Half-width of the wall (distance from center line to edge).
    pub half_width: f32,
    /// Damage dealt each tick.
    pub damage_per_tick: f32,
    /// Type of damage dealt.
    #[allow(dead_code)]
    pub damage_type: DamageType,
    /// Time between damage ticks (seconds).
    pub tick_interval: f32,
    /// Total lifetime (seconds).
    pub duration: f32,
    /// Elapsed time (seconds).
    pub time_alive: f32,
    /// Accumulator for tick timing.
    pub time_since_last_tick: f32,
}

impl WallOfFireEffect {
    pub fn new(
        start: Vec3,
        end: Vec3,
        half_width: f32,
        damage_per_tick: f32,
        damage_type: DamageType,
        tick_interval: f32,
        duration: f32,
    ) -> Self {
        Self {
            start,
            end,
            half_width,
            damage_per_tick,
            damage_type,
            tick_interval,
            duration,
            time_alive: 0.0,
            time_since_last_tick: 0.0,
        }
    }

    /// Returns the shortest XZ-plane distance from a point to the wall's center line segment.
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        let p = Vec2::new(point.x, point.z);
        let a = Vec2::new(self.start.x, self.start.z);
        let b = Vec2::new(self.end.x, self.end.z);
        let ab = b - a;
        let ap = p - a;
        let ab_len_sq = ab.length_squared();
        if ab_len_sq < 0.0001 {
            return ap.length();
        }
        let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);
        let closest = a + ab * t;
        (p - closest).length()
    }
}

/// Component on the wizard tracking wall of fire placement state.
#[derive(Component)]
pub struct WallOfFireCaster {
    /// World position where the player first clicked.
    pub anchor: Option<Vec3>,
    /// Entity ID of the preview mesh.
    pub preview_entity: Option<Entity>,
}

impl WallOfFireCaster {
    pub const fn new() -> Self {
        Self {
            anchor: None,
            preview_entity: None,
        }
    }
}

/// Marker component for the wall of fire preview mesh shown during drag.
#[derive(Component)]
pub struct WallOfFirePreview;
