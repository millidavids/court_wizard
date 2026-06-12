//! Sleep effect and all sleep-talent components.

use bevy::prelude::*;

/// Sleep effect from Sleep spell.
///
/// Sleeping units cannot move or attack. First damage hit deals bonus damage
/// and wakes them (removes this effect).
///
/// Talent-specific behaviors are separate components: [`NightTerrors`],
/// [`Comatose`], [`NarcolepticWave`], [`Sleepwalking`].
#[derive(Component)]
pub struct SleepModifier {
    /// Time remaining before the effect expires (in seconds).
    pub time_remaining: f32,
    /// Bonus damage multiplier on first hit (e.g., 2.0 = double damage).
    pub bonus_damage_multiplier: f32,
    /// Full duration this modifier was created with (for narcoleptic wave inheritance).
    pub full_duration: f32,
}

impl SleepModifier {
    pub fn new(duration: f32, bonus_multiplier: f32) -> Self {
        Self {
            time_remaining: duration,
            bonus_damage_multiplier: bonus_multiplier,
            full_duration: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Night Terrors talent: sleeping units take minor DPS.
#[derive(Component)]
pub struct NightTerrors {
    pub dps: f32,
    pub tick_accumulator: f32,
}

impl NightTerrors {
    pub fn new(dps: f32) -> Self {
        Self {
            dps,
            tick_accumulator: 0.0,
        }
    }
}

/// Comatose talent: sleeping units only wake if a single hit exceeds a fraction of max HP.
#[derive(Component)]
pub struct Comatose {
    /// Fraction of max HP that a single hit must exceed to wake (e.g., 0.3 = 30%).
    pub wake_threshold: f32,
}

impl Comatose {
    pub fn new(threshold: f32) -> Self {
        Self {
            wake_threshold: threshold,
        }
    }
}

/// Narcoleptic Wave talent: after a delay, sleep spreads to nearby awake enemies.
/// Removed from the entity once it has spread.
#[derive(Component)]
pub struct NarcolepticWave {
    /// Timer counting down before sleep spreads.
    pub timer: f32,
    /// Radius for spreading sleep.
    pub radius: f32,
}

impl NarcolepticWave {
    pub fn new(delay: f32, radius: f32) -> Self {
        Self {
            timer: delay,
            radius,
        }
    }
}

/// Dreamwalker talent: sleeping units sleepwalk back toward spawn instead of being immobilized.
#[derive(Component)]
pub struct Sleepwalking {
    pub speed_mult: f32,
}

impl Sleepwalking {
    pub fn new(speed_mult: f32) -> Self {
        Self { speed_mult }
    }
}
