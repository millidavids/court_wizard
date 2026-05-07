use std::collections::HashSet;

use bevy::prelude::*;

use crate::game::units::DamageType;

/// Shared hit tracking for all bolts from a single chain lightning cast.
/// Prevents two sibling bolts from targeting the same unit.
#[derive(Component)]
pub struct ChainLightningGroup {
    /// All entities hit by any bolt in this group.
    pub hit_entities: HashSet<Entity>,
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
    // --- Talent fields ---
    /// Number of targets per bounce (default 2, Forked Lightning = 3, Overcharge = 1).
    pub split_count: usize,
    /// Damage multiplier per bounce (default 0.6, High Voltage = 0.4, Overcharge = 1.0).
    pub damage_falloff: f32,
    /// Whether hit enemies are slowed (Static Charge talent).
    pub static_charge: bool,
    /// Whether hit enemies are pulled toward bolt origin (Magnetic Pull talent).
    pub magnetic_pull: bool,
    /// Whether kills trigger AoE + sub-chains (Chain Reaction talent).
    pub chain_reaction: bool,
    /// Bounce range multiplier (Conducting Bolts = 1.5).
    pub bounce_range_mult: f32,
}

/// Snapshot-only marker on a `LightningBolt` parent for multiplayer serialization.
#[derive(Component)]
pub struct ChainLightningArc {
    pub start: Vec3,
    pub end: Vec3,
}
