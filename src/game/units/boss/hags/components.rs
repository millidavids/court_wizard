use bevy::prelude::*;

/// Marker component for all hag boss units.
#[derive(Component)]
pub struct Hag;

/// Identifies which hag this entity is.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum HagIdentity {
    /// Fire-themed hag (chain lightning, fireball).
    Justina,
    /// Mind-themed hag (teleport pull, mind control).
    Martina,
    /// Physical-themed hag (leap, vicious mauling, corpse consume).
    Josephina,
}

/// Tracks which magical eyes this hag currently holds.
#[derive(Component)]
pub struct HagEyeState {
    /// Holds the invulnerability eye (damage is negated while held).
    pub has_invulnerability_eye: bool,
    /// Holds the ability eye (can use special abilities).
    pub has_ability_eye: bool,
}

impl HagEyeState {
    pub fn new() -> Self {
        Self {
            has_invulnerability_eye: false,
            has_ability_eye: false,
        }
    }

    /// Returns true if this hag has no eyes (blind).
    pub fn is_blind(&self) -> bool {
        !self.has_invulnerability_eye && !self.has_ability_eye
    }
}

/// Cooldown timer for hag melee attacks (independent of global cycle).
#[derive(Component)]
pub struct HagAttackCooldown {
    pub time_remaining: f32,
}

impl HagAttackCooldown {
    pub const fn new() -> Self {
        Self {
            time_remaining: 0.0,
        }
    }

    pub fn tick(&mut self, delta: f32) {
        self.time_remaining -= delta;
    }

    pub fn is_ready(&self) -> bool {
        self.time_remaining <= 0.0
    }

    pub fn reset(&mut self, cooldown: f32) {
        self.time_remaining = cooldown;
    }
}

/// Marker for eye visual child entities.
#[derive(Component)]
pub struct EyeVisual {
    pub eye_type: EyeType,
}

/// Type of magical eye.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EyeType {
    Invulnerability,
    Ability,
}

/// Component for eyes flying between hags during transfer.
#[derive(Component)]
pub struct EyeInFlight {
    pub eye_type: EyeType,
    pub target: Entity,
    pub start_pos: Vec3,
    pub progress: f32,
}

/// Random wandering state for blind (eyeless) hags.
#[derive(Component)]
pub struct BlindWanderState {
    /// Time until next direction change.
    pub timer: f32,
    /// Current wandering direction (normalized XZ).
    pub direction_x: f32,
    pub direction_z: f32,
}

impl BlindWanderState {
    pub fn new() -> Self {
        Self {
            timer: 0.0,
            direction_x: 0.0,
            direction_z: 1.0,
        }
    }
}

/// Marker component for hags that are permanently dead (killed while blind).
#[derive(Component)]
pub struct PermanentlyDead;

// ===== Justina Ability Components =====

/// Cooldown for Justina's chain lightning ability.
#[derive(Component)]
pub struct ChainLightningCooldown {
    pub time_remaining: f32,
}

impl ChainLightningCooldown {
    pub fn new() -> Self {
        Self {
            time_remaining: 2.0,
        }
    }
}

/// Cooldown for Justina's fireball ability.
#[derive(Component)]
pub struct FireballCooldown {
    pub time_remaining: f32,
}

impl FireballCooldown {
    pub fn new() -> Self {
        Self {
            time_remaining: 5.0,
        }
    }
}

// ===== Josephina Ability Components =====

/// Tracks Josephina's leap ability state.
#[derive(Component)]
pub enum LeapState {
    Idle {
        cooldown: f32,
    },
    InAir {
        target: Vec3,
        start_pos: Vec3,
        progress: f32,
    },
    Landing {
        timer: f32,
        knockback_applied: bool,
    },
}

impl LeapState {
    pub fn new() -> Self {
        Self::Idle { cooldown: 5.0 }
    }
}

/// Tracks Josephina's frenzy state (5x attack speed after leap landing).
#[derive(Component)]
pub struct MaulingState {
    /// Time remaining for the frenzy buff. 0.0 = inactive.
    pub frenzy_timer: f32,
}

impl MaulingState {
    pub fn new() -> Self {
        Self { frenzy_timer: 0.0 }
    }

    pub fn is_frenzied(&self) -> bool {
        self.frenzy_timer > 0.0
    }
}

/// Tracks Josephina's corpse consuming state.
#[derive(Component)]
pub struct CorpseConsumeState {
    pub timer: f32,
    pub corpse_entity: Entity,
}

// ===== Martina Ability Components =====

/// Cooldown for Martina's teleport pull ability.
#[derive(Component)]
pub struct TeleportPullCooldown {
    pub time_remaining: f32,
}

impl TeleportPullCooldown {
    pub fn new() -> Self {
        Self {
            time_remaining: 6.0,
        }
    }
}
