use bevy::prelude::*;

use crate::game::units::constants::{
    ELECTRIC_ARC_CHANCE_PER_DAMAGE, ELECTRIC_ARC_CHANCE_PER_HIT, ELECTRIC_ARC_COOLDOWN,
    ELECTRIC_ARC_DURATION, ELECTRIC_ARC_MAX_CHANCE, FIRE_DOT_DAMAGE_RATIO, FIRE_DOT_DURATION,
    FIRE_DOT_MAX_DPS, FIRE_DOT_TICK_INTERVAL,
};

/// Fire damage-over-time effect that stacks with repeated fire hits.
///
/// Each fire hit adds a percentage of spell damage as extra DoT DPS.
/// Duration resets on each new fire hit.
#[derive(Component)]
pub struct FireDoT {
    /// Accumulated DoT DPS (grows with each fire hit).
    pub damage_per_tick: f32,
    /// Time remaining before the DoT expires (resets on each fire hit).
    pub time_remaining: f32,
    /// Accumulator for tick timing.
    pub tick_timer: f32,
}

impl FireDoT {
    /// Creates a new FireDoT from the initial fire damage.
    pub fn new(spell_damage: f32) -> Self {
        let dps = (spell_damage * FIRE_DOT_DAMAGE_RATIO).min(FIRE_DOT_MAX_DPS);
        Self {
            damage_per_tick: dps,
            time_remaining: FIRE_DOT_DURATION,
            tick_timer: 0.0,
        }
    }

    /// Stacks additional fire damage and resets the duration.
    pub fn stack(&mut self, spell_damage: f32) {
        self.damage_per_tick =
            (self.damage_per_tick + spell_damage * FIRE_DOT_DAMAGE_RATIO).min(FIRE_DOT_MAX_DPS);
        self.time_remaining = FIRE_DOT_DURATION;
    }

    /// Ticks the DoT timer, returning damage to apply this frame (if any).
    /// Returns `None` if no tick happened, `Some(damage)` if a tick occurred.
    /// Also returns `true` in the second tuple element if the DoT has expired.
    pub fn update(&mut self, delta: f32) -> (Option<f32>, bool) {
        self.time_remaining -= delta;
        if self.time_remaining <= 0.0 {
            return (None, true);
        }

        self.tick_timer += delta;
        if self.tick_timer >= FIRE_DOT_TICK_INTERVAL {
            self.tick_timer -= FIRE_DOT_TICK_INTERVAL;
            let tick_damage = self.damage_per_tick * FIRE_DOT_TICK_INTERVAL;
            (Some(tick_damage), false)
        } else {
            (None, false)
        }
    }
}

/// Electric charge effect that builds arc chance with repeated electric hits.
///
/// Each electric hit adds arc chance. When the charge arcs, it deals damage
/// to nearby enemies and builds charge on them too.
#[derive(Component)]
pub struct Shocked {
    /// Chance per tick to arc (0.0–1.0).
    pub arc_chance: f32,
    /// Time remaining before the charge expires (resets on each electric hit).
    pub time_remaining: f32,
    /// Cooldown timer preventing arcing every frame.
    pub arc_cooldown: f32,
}

impl Shocked {
    /// Creates a new Shocked from the initial electric damage.
    pub fn new(spell_damage: f32) -> Self {
        let chance = (ELECTRIC_ARC_CHANCE_PER_HIT + spell_damage * ELECTRIC_ARC_CHANCE_PER_DAMAGE)
            .min(ELECTRIC_ARC_MAX_CHANCE);
        Self {
            arc_chance: chance,
            time_remaining: ELECTRIC_ARC_DURATION,
            arc_cooldown: ELECTRIC_ARC_COOLDOWN,
        }
    }

    /// Stacks additional electric charge and resets the duration.
    pub fn stack(&mut self, spell_damage: f32) {
        self.arc_chance = (self.arc_chance
            + ELECTRIC_ARC_CHANCE_PER_HIT
            + spell_damage * ELECTRIC_ARC_CHANCE_PER_DAMAGE)
            .min(ELECTRIC_ARC_MAX_CHANCE);
        self.time_remaining = ELECTRIC_ARC_DURATION;
    }

    /// Updates timers. Returns `true` if the charge has expired.
    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        if self.time_remaining <= 0.0 {
            return true;
        }
        self.arc_cooldown = (self.arc_cooldown - delta).max(0.0);
        false
    }

    /// Returns `true` if arc is off cooldown.
    pub fn can_arc(&self) -> bool {
        self.arc_cooldown <= 0.0
    }

    /// Resets the arc cooldown after a successful arc.
    pub fn reset_arc_cooldown(&mut self) {
        self.arc_cooldown = ELECTRIC_ARC_COOLDOWN;
    }
}
