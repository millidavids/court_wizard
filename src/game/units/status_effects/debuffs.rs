//! Negative debuff components: mark of death, fog evasion, banished, sickened, poisoned, smelly.

use bevy::prelude::*;

use super::super::components::impl_timed_modifier;

/// Damage amplification modifier from Mark of Death.
///
/// Marked units take increased damage from ALL sources.
/// Combat system applies: damage * (1.0 + damage_amplification).
#[derive(Component)]
pub struct MarkedForDeathModifier {
    /// Damage amplification (e.g., 0.5 = +50% damage taken).
    pub damage_amplification: f32,
    /// Time remaining before the mark expires (in seconds).
    pub time_remaining: f32,
}

impl MarkedForDeathModifier {
    pub const fn new(amplification: f32, duration: f32) -> Self {
        Self {
            damage_amplification: amplification,
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Fog evasion effect from Fog Cloud.
///
/// Units inside the fog have a chance to evade incoming attacks.
#[derive(Component)]
pub struct FogEvasionModifier {
    /// Evasion chance (0.0–1.0, e.g., 0.4 = 40% dodge chance).
    pub evasion_chance: f32,
    /// Time remaining before the evasion expires (in seconds).
    pub time_remaining: f32,
}

impl FogEvasionModifier {
    pub const fn new(chance: f32, duration: f32) -> Self {
        Self {
            evasion_chance: chance,
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

/// Banishment effect that removes a unit from the battlefield temporarily.
///
/// Banished units are hidden, untargetable, and cannot act.
/// When the effect expires, the unit reappears.
#[derive(Component)]
pub struct BanishedModifier {
    /// Time remaining before the unit returns (in seconds).
    pub time_remaining: f32,
}

impl BanishedModifier {
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

/// Permanent marker preventing a unit from being banished again.
#[derive(Component)]
pub struct WasBanished;

/// Sickened effect that prevents a unit from moving or acting.
///
/// Applied by certain spells. The unit is incapacitated for the duration.
/// When the effect expires, the component is removed.
#[derive(Component)]
pub struct SickenedModifier {
    /// Time remaining before the effect expires (in seconds).
    pub time_remaining: f32,
}

impl SickenedModifier {
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

/// Poison debuff that reduces unit effectiveness over time.
///
/// Stacks up to a cap. If total accumulated poison reaches the sickened threshold,
/// the unit becomes sickened (stops moving) and then smelly (allies flee).
#[derive(Component)]
pub struct PoisonedModifier {
    /// Accumulated effectiveness penalty (negative, grows with stacking).
    pub effectiveness_penalty: f32,
    /// Time remaining before poison expires (resets on each stack).
    pub time_remaining: f32,
    /// Timer for periodic effectiveness penalty ticks.
    pub tick_timer: f32,
    /// Total accumulated penalty for sickened threshold check.
    pub total_accumulated: f32,
    /// Total penalty applied to spell_bonus (for accurate cleanup).
    pub applied_to_spell_bonus: f32,
}

impl PoisonedModifier {
    pub fn new(penalty_per_stack: f32, duration: f32) -> Self {
        Self {
            effectiveness_penalty: penalty_per_stack,
            time_remaining: duration,
            tick_timer: 0.0,
            total_accumulated: penalty_per_stack.abs(),
            applied_to_spell_bonus: 0.0,
        }
    }

    pub fn stack(&mut self, penalty_per_stack: f32, duration: f32, cap: f32) {
        self.effectiveness_penalty = (self.effectiveness_penalty + penalty_per_stack).max(cap);
        self.time_remaining = duration;
        self.total_accumulated += penalty_per_stack.abs();
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }

    pub fn is_sickened(&self, threshold: f32) -> bool {
        self.total_accumulated >= threshold
    }
}

/// Smelly debuff that causes allied units to flee.
///
/// Applied after sickened expires or directly by Poop damage.
/// Other units on the same team avoid the smelly unit.
#[derive(Component)]
pub struct SmellyModifier {
    /// Time remaining before the smell fades.
    pub time_remaining: f32,
}

impl SmellyModifier {
    pub fn new(duration: f32) -> Self {
        Self {
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

impl_timed_modifier!(SmellyModifier);
