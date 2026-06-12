//! Classic crowd-control components: stun, frozen, petrified, fear, rooted.

use bevy::prelude::*;

/// A dedicated system zeros velocity on stunned units each frame.
#[derive(Component)]
pub struct Stunned {
    /// Time remaining before the stun expires (in seconds).
    pub time_remaining: f32,
}

impl Stunned {
    pub const fn new(duration: f32) -> Self {
        Self {
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Frozen solid effect that prevents both movement and attacking.
///
/// Applied by Squall's Permafrost and Absolute Zero talents.
/// Unlike RootedModifier, frozen units cannot attack either.
#[derive(Component)]
pub struct FrozenSolidModifier {
    /// Time remaining before the freeze expires (in seconds).
    pub time_remaining: f32,
}

impl FrozenSolidModifier {
    pub const fn new(duration: f32) -> Self {
        Self {
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Petrification effect — unit is turned to stone, cannot move or attack.
/// Applied by Ray's petrification eye beam.
#[derive(Component)]
pub struct Petrified {
    pub time_remaining: f32,
}

impl Petrified {
    pub const fn new(duration: f32) -> Self {
        Self {
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Fear effect — unit flees away from the source position at full speed.
/// Applied by Ray's fear eye beam.
#[derive(Component)]
pub struct FearModifier {
    pub time_remaining: f32,
    pub flee_from: Vec3,
}

impl FearModifier {
    pub fn new(duration: f32, flee_from: Vec3) -> Self {
        Self {
            time_remaining: duration,
            flee_from,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Movement speed modifier from being rooted (unable to move).
///
/// Applied to units hit by the Entangle spell.
/// Rooted units cannot move but can still attack.
#[derive(Component)]
pub struct RootedModifier {
    /// Time remaining before the root effect expires (in seconds).
    pub time_remaining: f32,
}

impl RootedModifier {
    /// Creates a new rooted modifier with the given duration.
    pub const fn new(duration: f32) -> Self {
        Self {
            time_remaining: duration,
        }
    }

    /// Updates the timer, returning true if expired.
    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}
