//! Positive and mixed buff components: haste, battle hymn, berserker rage.

use bevy::prelude::*;

/// Movement speed modifier from haste effect as a percentage.
///
/// Applied to units affected by the Haste spell.
/// Examples: 0.5 = +50% speed (1.5x multiplier).
/// Movement systems apply this as: speed * (1.0 + sum_of_all_modifiers).
#[derive(Component)]
pub struct HasteModifier {
    /// Speed increase as a percentage (positive value).
    pub modifier: f32,
    /// Time remaining before the haste effect expires (in seconds).
    pub time_remaining: f32,
    /// Attack speed bonus (e.g., 0.2 = +20% attack speed). From Adrenaline Surge / Time Warp.
    pub attack_speed: f32,
}

impl HasteModifier {
    /// Creates a new haste modifier with the given strength and duration.
    pub const fn new(modifier: f32, duration: f32) -> Self {
        Self {
            modifier,
            time_remaining: duration,
            attack_speed: 0.0,
        }
    }

    /// Creates a new haste modifier with attack speed bonus.
    pub const fn with_attack_speed(modifier: f32, duration: f32, attack_speed: f32) -> Self {
        Self {
            modifier,
            time_remaining: duration,
            attack_speed,
        }
    }

    /// Updates the timer, returning true if expired.
    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }

    /// Refreshes the duration (used when reapplying the haste).
    pub fn refresh(&mut self, duration: f32) {
        self.time_remaining = duration;
    }
}

/// Battle Hymn buff granting damage and attack speed bonuses.
///
/// Combat system adds damage_bonus to outgoing damage and scales attack timing.
/// Talent-specific behaviors are separate components: [`EchoingSong`], [`AnthemResilience`].
#[derive(Component)]
pub struct BattleHymnModifier {
    /// Damage bonus as a percentage (e.g., 0.4 = +40% damage).
    pub damage_bonus: f32,
    /// Attack speed bonus as a percentage (e.g., 0.3 = 30% faster attacks).
    pub attack_speed: f32,
    /// Time remaining before the buff expires (in seconds).
    pub time_remaining: f32,
}

impl BattleHymnModifier {
    pub const fn new(damage_bonus: f32, attack_speed: f32, duration: f32) -> Self {
        Self {
            damage_bonus,
            attack_speed,
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }

    pub fn refresh(&mut self, duration: f32) {
        self.time_remaining = duration;
    }
}

/// Echoing Song talent: when BattleHymnModifier expires, re-apply at reduced duration.
#[derive(Component)]
pub struct EchoingSong {
    /// Duration to re-apply when the buff expires.
    pub echo_duration: f32,
}

impl EchoingSong {
    pub fn new(duration: f32) -> Self {
        Self {
            echo_duration: duration,
        }
    }
}

/// Anthem of Resilience talent: damage reduction while Battle Hymn is active.
#[derive(Component)]
pub struct AnthemResilience {
    /// Damage reduction percentage (e.g., 0.3 = 30% less damage taken).
    pub damage_reduction: f32,
}

impl AnthemResilience {
    pub fn new(reduction: f32) -> Self {
        Self {
            damage_reduction: reduction,
        }
    }
}

/// Berserker Rage buff granting damage bonus but increasing damage taken.
///
/// Risk/reward buff: +damage dealt, +damage taken.
#[derive(Component)]
pub struct BerserkerRageModifier {
    /// Damage bonus as a percentage (e.g., 0.8 = +80% damage dealt).
    pub damage_bonus: f32,
    /// Damage vulnerability as a percentage (e.g., 0.5 = +50% damage taken).
    pub damage_vulnerability: f32,
    /// Time remaining before the buff expires (in seconds).
    pub time_remaining: f32,
}

impl BerserkerRageModifier {
    pub const fn new(damage_bonus: f32, vulnerability: f32, duration: f32) -> Self {
        Self {
            damage_bonus,
            damage_vulnerability: vulnerability,
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }

    pub fn refresh(&mut self, duration: f32) {
        self.time_remaining = duration;
    }
}
