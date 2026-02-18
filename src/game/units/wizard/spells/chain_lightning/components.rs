use bevy::prelude::*;

use crate::game::units::DamageType;

/// Shared hit tracking for all bolts from a single chain lightning cast.
/// Prevents two sibling bolts from targeting the same unit.
#[derive(Component)]
pub struct ChainLightningGroup {
    /// All entities hit by any bolt in this group.
    pub hit_entities: Vec<Entity>,
}

/// Tracks a single chain lightning bolt through its lifecycle.
/// Each bolt splits into multiple child bolts when it bounces.
#[derive(Component)]
pub struct ChainLightningBolt {
    /// Reference to the shared group entity for hit tracking.
    pub group_entity: Entity,
    /// Current damage for next hit (decreases with each split level).
    pub current_damage: f32,
    /// Type of damage dealt.
    #[allow(dead_code)]
    pub damage_type: DamageType,
    /// Number of split levels remaining.
    pub bounces_remaining: u32,
    /// Position of last hit (origin for next arc).
    pub last_hit_position: Vec3,
    /// Time remaining before next bounce triggers.
    pub bounce_delay_timer: f32,
    /// Whether this chain lightning is empowered.
    pub empowerment: f32,
    /// Current depth in the splitting tree (0 = first bounce after initial hit).
    pub split_depth: u32,
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
    /// Depth in the splitting tree (for visual scaling).
    pub depth: u32,
}
