use bevy::prelude::*;

/// Marker component identifying this entity as The Lich.
#[derive(Component)]
pub struct Lich;

/// Resource that signals a Lich will spawn on this level.
/// Inserted at level start on Lich levels; removed when the Lich spawns.
/// While this exists, the win condition is blocked.
#[derive(Resource)]
pub struct LichSpawnPending;

/// State machine tracking the Lich's current phase.
#[derive(Component, Default, PartialEq, Eq)]
pub enum LichPhase {
    /// Moving from tunnel spawn to staging point.
    #[default]
    Approaching,
    /// Stationary at staging point, summoning undead waves.
    Summoning,
    /// Soul power full — wading into battle with Finger of Death beams.
    Combat,
}

/// Tracks soul power gained from undead kills.
/// Displayed as a filling bar in Phase 1. When full, triggers Phase 2.
#[derive(Component)]
pub struct SoulPower {
    pub current: f32,
    pub max: f32,
    /// Snapshot of undead_killed from KillStats to detect new kills.
    pub last_known_undead_killed: u32,
}

impl SoulPower {
    pub fn new(max: f32) -> Self {
        Self {
            current: 0.0,
            max,
            last_known_undead_killed: 0,
        }
    }

    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    pub fn percent(&self) -> f32 {
        (self.current / self.max * 100.0).clamp(0.0, 100.0)
    }
}

/// Timer for summoning undead waves during Phase 1.
#[derive(Component)]
pub struct LichSummonTimer {
    pub time_remaining: f32,
    pub waves_summoned: u32,
}

impl LichSummonTimer {
    pub fn new(interval: f32) -> Self {
        Self {
            // First summon happens after a short delay.
            time_remaining: interval * 0.5,
            waves_summoned: 0,
        }
    }

    pub fn tick(&mut self, delta: f32) {
        self.time_remaining -= delta;
    }

    pub fn is_ready(&self) -> bool {
        self.time_remaining <= 0.0
    }

    pub fn reset(&mut self, interval: f32) {
        self.time_remaining = interval;
        self.waves_summoned += 1;
    }
}

/// Finger of Death attack state for Phase 2.
#[derive(Component)]
pub struct LichFingerOfDeath {
    pub cooldown: f32,
    pub target: Option<Entity>,
}

impl LichFingerOfDeath {
    pub fn new() -> Self {
        Self {
            cooldown: 1.0, // Brief delay before first beam.
            target: None,
        }
    }

    pub fn tick(&mut self, delta: f32) {
        self.cooldown -= delta;
    }

    pub fn is_ready(&self) -> bool {
        self.cooldown <= 0.0
    }

    pub fn reset(&mut self, cooldown: f32) {
        self.cooldown = cooldown;
        self.target = None;
    }
}

