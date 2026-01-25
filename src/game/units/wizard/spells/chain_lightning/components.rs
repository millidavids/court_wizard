use bevy::prelude::*;

/// Tracks the chain lightning spell state through its lifecycle.
#[derive(Component)]
pub struct ChainLightningBolt {
    /// Entities already hit (to prevent re-targeting).
    pub hit_entities: Vec<Entity>,
    /// Current damage for next hit (decreases with each bounce).
    pub current_damage: f32,
    /// Number of bounces remaining.
    pub bounces_remaining: u32,
    /// Position of last hit (origin for next arc).
    pub last_hit_position: Vec3,
    /// Time remaining before next bounce triggers.
    pub bounce_delay_timer: f32,
}

/// Visual lightning arc between two points.
#[derive(Component)]
pub struct ChainLightningArc {
    /// Start position of the arc.
    #[allow(dead_code)]
    pub start: Vec3,
    /// End position of the arc.
    #[allow(dead_code)]
    pub end: Vec3,
    /// Time remaining before arc despawns.
    pub lifetime: f32,
    /// Time since arc was created (for animation).
    pub time_alive: f32,
}
