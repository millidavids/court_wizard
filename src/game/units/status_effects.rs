//! Status effect components (CC, buffs, debuffs).

use bevy::prelude::*;

use super::components::{Team, impl_timed_modifier};

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

/// Polymorph effect that transforms a unit into a sheep.
///
/// Stores the original unit state for restoration when the effect expires.
#[derive(Component)]
pub struct PolymorphedModifier {
    /// Time remaining before the unit reverts (in seconds).
    pub time_remaining: f32,
    /// Original current health to restore on revert.
    pub original_health_current: f32,
    /// Original max health to restore on revert.
    pub original_health_max: f32,
    /// Original material handle to restore on revert.
    pub original_material: Handle<StandardMaterial>,
    /// Original mesh handle to restore on revert.
    pub original_mesh: Handle<Mesh>,
    /// Original team to restore on revert.
    pub original_team: Team,
}

impl PolymorphedModifier {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        duration: f32,
        health_current: f32,
        health_max: f32,
        material: Handle<StandardMaterial>,
        mesh: Handle<Mesh>,
        team: Team,
    ) -> Self {
        Self {
            time_remaining: duration,
            original_health_current: health_current,
            original_health_max: health_max,
            original_material: material,
            original_mesh: mesh,
            original_team: team,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}
