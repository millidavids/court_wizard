use std::collections::VecDeque;

use bevy::prelude::*;

/// The three spell types the Dark Mage can cast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DarkMageSpellType {
    DarkMeteor,
    ShadowLightning,
    PlagueCloud,
}

/// Individual cooldown tracker for each spell.
#[derive(Component)]
pub struct DarkMageSpellCooldowns {
    pub meteor: f32,
    pub lightning: f32,
    pub plague: f32,
}

impl DarkMageSpellCooldowns {
    pub fn new(meteor_cd: f32, lightning_cd: f32, plague_cd: f32) -> Self {
        Self {
            meteor: meteor_cd,
            lightning: lightning_cd,
            plague: plague_cd,
        }
    }

    /// Tick all cooldowns by delta. Returns nothing; callers check readiness.
    pub fn tick(&mut self, delta: f32) {
        self.meteor = (self.meteor - delta).max(0.0);
        self.lightning = (self.lightning - delta).max(0.0);
        self.plague = (self.plague - delta).max(0.0);
    }

    /// Returns true if the given spell is off cooldown.
    pub fn is_ready(&self, spell: DarkMageSpellType) -> bool {
        match spell {
            DarkMageSpellType::DarkMeteor => self.meteor <= 0.0,
            DarkMageSpellType::ShadowLightning => self.lightning <= 0.0,
            DarkMageSpellType::PlagueCloud => self.plague <= 0.0,
        }
    }

    /// Reset the cooldown for the given spell.
    pub fn reset(&mut self, spell: DarkMageSpellType, cooldown: f32) {
        match spell {
            DarkMageSpellType::DarkMeteor => self.meteor = cooldown,
            DarkMageSpellType::ShadowLightning => self.lightning = cooldown,
            DarkMageSpellType::PlagueCloud => self.plague = cooldown,
        }
    }
}

/// Queue of spells ready to cast.
#[derive(Component)]
pub struct DarkMageSpellQueue {
    pub queue: VecDeque<DarkMageSpellType>,
}

impl DarkMageSpellQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

/// Main state machine for the Dark Mage.
#[derive(Component)]
pub enum DarkMageState {
    /// Walking from tunnel spawn to the battlefield before engaging.
    Approaching,
    /// Waiting to pick the next spell from the queue.
    Idle,
    /// Telegraph phase: showing the indicator before casting.
    Telegraphing {
        spell_type: DarkMageSpellType,
        elapsed: f32,
        duration: f32,
        target_pos: Vec3,
        /// Direction for lightning corridor (unused for circle spells).
        direction: Option<Vec3>,
        indicators: DarkMageIndicators,
    },
    /// Brief cast moment: spell fires, indicator despawns.
    Casting {
        spell_type: DarkMageSpellType,
        target_pos: Vec3,
        direction: Option<Vec3>,
    },
}

/// Handles for telegraph indicator entities.
pub struct DarkMageIndicators {
    pub entities: Vec<Entity>,
    pub fill_material: Handle<StandardMaterial>,
}

impl DarkMageIndicators {
    pub fn all(&self) -> &[Entity] {
        &self.entities
    }
}

/// Teleport cooldown timer.
#[derive(Component)]
pub struct DarkMageTeleportTimer {
    pub cooldown: f32,
}

impl DarkMageTeleportTimer {
    pub fn new(cooldown: f32) -> Self {
        Self { cooldown }
    }

    pub fn tick(&mut self, delta: f32) {
        self.cooldown = (self.cooldown - delta).max(0.0);
    }

    pub fn is_ready(&self) -> bool {
        self.cooldown <= 0.0
    }

    pub fn reset(&mut self, cooldown: f32) {
        self.cooldown = cooldown;
    }
}

/// Tracks enrage state based on HP thresholds.
#[derive(Component)]
pub struct DarkMageEnrage {
    pub phase: u8,
    /// Multiplier applied to spell cooldowns (< 1.0 = faster).
    pub cooldown_mult: f32,
}

impl DarkMageEnrage {
    pub fn new() -> Self {
        Self {
            phase: 0,
            cooldown_mult: 1.0,
        }
    }
}

/// Marker component for telegraph indicator entities.
#[derive(Component)]
pub struct DarkMageIndicator;

/// Marker component for the Dark Mage entity itself.
#[derive(Component)]
pub struct DarkMage;

/// Persistent plague cloud entity that deals periodic damage.
#[derive(Component)]
pub struct DarkMagePlagueCloud {
    pub lifetime: f32,
    pub tick_timer: f32,
    pub particle_timer: f32,
    pub radius: f32,
    pub damage: f32,
}

/// Temporary explosion visual from Dark Meteor.
#[derive(Component)]
pub struct DarkMageMeteorExplosion {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub radius: f32,
    pub damage_applied: bool,
    pub damage: f32,
}

/// Temporary lightning strike visual.
#[derive(Component)]
pub struct DarkMageLightningStrike {
    pub lifetime: f32,
    pub half_width: f32,
    pub half_length: f32,
    pub direction: Vec3,
    pub damage_applied: bool,
    pub damage: f32,
}

/// Falling meteor projectile visual (falls from sky to target).
#[derive(Component)]
pub struct DarkMageMeteorProjectile {
    pub target_pos: Vec3,
    pub velocity: f32,
}

/// Vertical lightning bolt visual (strikes down from sky).
#[derive(Component)]
pub struct DarkMageLightningBolt {
    pub lifetime: f32,
}

/// Generic temporary visual effect with a lifetime (used for puffs, particles, etc.).
#[derive(Component)]
pub struct DarkMageVisualEffect {
    pub lifetime: f32,
}
