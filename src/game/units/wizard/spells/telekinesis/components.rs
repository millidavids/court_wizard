use bevy::prelude::*;

/// Tracks transmutation stacks from the T3 Transmutation talent.
/// Each ingredient collected adds a stack. Stacks boost brew potency and reset on brew.
#[derive(Resource, Default)]
pub(crate) struct TransmutationStacks {
    pub count: u32,
}

/// Brief light blue flash entity spawned at an enemy's position when damaged by Harvest (T2).
/// Fades out and despawns automatically.
#[derive(Component)]
pub(super) struct HarvestFlash {
    pub time_remaining: f32,
    /// Whether the material has been cloned for per-entity alpha fade.
    pub material_cloned: bool,
}

/// Expanding torus ring for Psychic Shockwave (T3 talent).
/// Spawned at the ingredient pickup position and expands outward,
/// knocking back enemies as it passes.
#[derive(Component)]
pub(super) struct PsychicShockwave {
    pub time_alive: f32,
    /// Previous frame's ring radius, used for ring-collision detection.
    pub prev_radius: f32,
    /// XZ origin of the shockwave (ingredient pickup position).
    pub origin: Vec3,
    /// Whether the material has been cloned for per-entity alpha fade.
    pub material_cloned: bool,
}

/// Visual indicator ring around a targeted ingredient drop during Telekinesis casting.
#[derive(Component)]
pub(super) struct TelekinesisIndicator {
    /// Entity of the drop being targeted.
    pub target_drop: Entity,
    /// Time this indicator has been alive (for pulse animation).
    pub time_alive: f32,
}

impl TelekinesisIndicator {
    pub const fn new(target_drop: Entity) -> Self {
        Self {
            target_drop,
            time_alive: 0.0,
        }
    }

    /// Returns the current scale factor for pulse animation.
    pub fn pulse_scale(&self) -> f32 {
        let pulse_freq = 3.0;
        let pulse_amplitude = 0.1;
        1.0 + (self.time_alive * pulse_freq * std::f32::consts::TAU).sin() * pulse_amplitude
    }
}
