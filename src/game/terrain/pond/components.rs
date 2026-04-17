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

/// Accumulated water-evaporation state from fire damage hitting a pond.
/// Feeds a growing fog cloud above the pond.
#[derive(Component, Default)]
pub struct PondEvaporation {
    /// Unitless fog intensity. Grows linearly with incoming fire damage.
    pub fog_intensity: f32,
    /// Seconds since the last fire contribution; fog despawns when this crosses the linger duration.
    pub time_since_contribution: f32,
}

/// Frost-accumulation state on a pond. At `freeze_level >= 0.5` the pond is considered frozen
/// and applies stronger slow + higher pathfinding cost.
#[derive(Component, Default)]
pub struct PondFrozen {
    /// 0.0 = liquid, 1.0 = fully frozen.
    pub freeze_level: f32,
    /// Countdown before thawing begins; resets on each frost contribution.
    pub decay_delay: f32,
    /// Tracks whether the pathfinding cost has been updated to the frozen value,
    /// so we only send `ObstacleChanged` on transitions.
    pub pathfinding_frozen: bool,
}

/// Electric-charge state on a pond. Periodically arcs lightning to nearby units at a larger
/// radius than the unit-level `ElectricCharge` effect.
#[derive(Component)]
pub struct PondShocked {
    /// Seconds before the shocked state expires (reset by any new electric hit).
    pub time_remaining: f32,
    /// Cooldown before the next arc pulse can fire.
    pub arc_cooldown: f32,
}

/// Marker: a pond's `StandardMaterial` has been cloned so its color can be tinted per-instance
/// (for the frozen overlay). Mirrors the `ClonedMaterial` marker used on boulders.
#[derive(Component)]
pub struct ClonedPondMaterial;

/// Visual child entity that owns the fog-puff emission timer for an evaporating pond.
/// The parent pond is found via `ChildOf`.
#[derive(Component, Default)]
pub struct PondFogCloud {
    /// Seconds since the last fog-puff spawn.
    pub smoke_timer: f32,
}
