use bevy::prelude::*;

use super::constants::{
    ELECTRIC_ARC_CHANCE_PER_DAMAGE, ELECTRIC_ARC_CHANCE_PER_HIT, ELECTRIC_ARC_COOLDOWN,
    ELECTRIC_ARC_DURATION, ELECTRIC_ARC_MAX_CHANCE, FIRE_DOT_DAMAGE_RATIO, FIRE_DOT_DURATION,
    FIRE_DOT_MAX_DPS, FIRE_DOT_TICK_INTERVAL,
};
use super::damage::DamageType;

// Sub-module re-exports (Phase 9 split)
pub use super::animation::*;
pub use super::status_effects::*;
pub use super::unit_type::*;

/// Trait for modifier components with a timed duration that can expire.
///
/// Implementing this trait allows use of the generic `update_timed_modifier::<T>` system
/// which automatically ticks and removes expired modifiers.
pub trait TimedModifier {
    /// Tick the modifier's timer by `delta` seconds. Returns `true` if expired.
    fn tick(&mut self, delta: f32) -> bool;
}

#[macro_export]
macro_rules! impl_timed_modifier {
    ($($ty:ty),* $(,)?) => {
        $(impl $crate::game::units::components::TimedModifier for $ty {
            fn tick(&mut self, delta: f32) -> bool {
                self.update(delta)
            }
        })*
    };
}
pub(crate) use impl_timed_modifier;

impl_timed_modifier!(
    SlowMovementModifier,
    TemporaryHitPoints,
    RootedModifier,
    HasteModifier,
    MarkedForDeathModifier,
    SleepModifier,
    // BattleHymnModifier has a custom tick system (handles EchoingSong)
    BerserkerRageModifier,
    FogEvasionModifier,
    FrozenSolidModifier,
    Stunned,
    Petrified,
    FearModifier,
);

/// Team component for all units.
///
/// Determines which side a unit is on. Units attack members of opposing teams.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Team {
    Defenders,
    Attackers,
    Undead,
}

impl Team {
    /// Returns true if units on these two teams are hostile to each other.
    /// Undead are hostile to everyone (including other Undead is false).
    pub fn is_enemy(&self, other: &Team) -> bool {
        match (self, other) {
            (Team::Undead, Team::Undead) => false,
            (Team::Undead, _) | (_, Team::Undead) => true,
            _ => self != other,
        }
    }
}

/// Health component for all units.
///
/// Tracks the current and maximum health of a unit.
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    /// Extra vulnerability to spell damage (0.0 = normal, 0.3 = +30% spell damage taken).
    /// Used by the Psychopath archetype to amplify spell damage against defenders.
    pub spell_vulnerability: f32,
    /// Healing reduction (0.0 = normal, 0.5 = 50% less healing received).
    /// Applied inside heal() so all heal sources are affected automatically.
    pub healing_reduction: f32,
}

/// Movement speed component for all units.
///
/// Determines how fast a unit moves in units per second.
#[derive(Component, Clone, Copy)]
pub struct MovementSpeed(pub f32);

/// Damage bonus as a percentage.
///
/// Used by special units and buffs to modify damage output.
/// Examples: 0.5 = +50% damage, 1.0 = +100% damage (double), -0.4 = -40% damage.
/// Combat system applies this as: damage * (1.0 + percentage).
#[derive(Component)]
pub struct DamageMultiplier(pub f32);

/// Movement speed modifier from Commander aura as a percentage.
///
/// Applied to units within a Commander's aura range.
/// Examples: 0.25 = +25% speed from commander aura.
/// Movement systems apply this as: speed * (1.0 + sum_of_all_modifiers).
#[derive(Component)]
pub struct CommanderAuraSpeedModifier(pub f32);

/// Movement speed modifier from rough terrain as a percentage.
///
/// Applied to units walking over corpses.
/// Examples: -0.6 = -60% speed (0.4x multiplier).
/// Movement systems apply this as: speed * (1.0 + sum_of_all_modifiers).
#[derive(Component)]
pub struct RoughTerrainModifier(pub f32);

/// Unified movement speed slow modifier.
///
/// Replaces the separate FrostSlowModifier, SpikeGrowthSlowModifier, and
/// GreaseSlipModifier. Uses strongest-wins semantics: when a new slow is
/// applied, the stronger modifier and longer duration are kept.
#[derive(Component)]
pub struct SlowMovementModifier {
    /// Speed reduction as a percentage (negative value, e.g., -0.4 = 40% slower).
    pub modifier: f32,
    /// Time remaining before the slow effect expires (in seconds).
    pub time_remaining: f32,
}

impl SlowMovementModifier {
    /// Creates a new slow modifier with the given strength and duration.
    pub const fn new(modifier: f32, duration: f32) -> Self {
        Self {
            modifier,
            time_remaining: duration,
        }
    }

    /// Updates the timer, returning true if expired.
    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }

    /// Apply a new slow. Keeps the stronger modifier and longer duration.
    pub fn apply(&mut self, modifier: f32, duration: f32) {
        if modifier < self.modifier {
            self.modifier = modifier;
        }
        if duration > self.time_remaining {
            self.time_remaining = duration;
        }
    }
}

/// Progressive frost accumulation that drives blue tint, movement slow, and eventual freeze.
///
/// Each frost hit increases `level`. As level rises, the unit turns bluer and moves
/// slower. At level 1.0 the unit freezes solid (can't move or attack).
/// Frost decays after a delay when no new hits land.
#[derive(Component)]
pub struct FrostAccumulation {
    /// Current frost level (0.0 = none, 1.0 = frozen).
    pub level: f32,
    /// Seconds remaining before frost starts decaying (resets on each hit).
    pub decay_delay: f32,
}

impl FrostAccumulation {
    pub fn new(initial_level: f32, decay_delay: f32) -> Self {
        Self {
            level: initial_level.min(1.0),
            decay_delay,
        }
    }

    /// Add frost from a hit. Resets the decay delay.
    pub fn add_frost(&mut self, amount: f32, decay_delay: f32) {
        self.level = (self.level + amount).min(1.0);
        self.decay_delay = decay_delay;
    }
}

/// Attack timing component for all units.
///
/// Tracks when in the global attack cycle a unit can attack.
/// Units attack at a specific time offset (0.0 to cycle_duration) and can only
/// attack again when the global timer cycles back to that offset. This naturally
/// staggers attacks across all units.
#[derive(Component)]
pub struct AttackTiming {
    /// The time offset in the cycle when this unit last attacked, or None if never attacked
    pub last_attack_time: Option<f32>,
}

impl AttackTiming {
    /// Creates a new AttackTiming with no attack scheduled.
    pub const fn new() -> Self {
        Self {
            last_attack_time: None,
        }
    }

    /// Returns true if the unit can attack at the current cycle time.
    /// Units can attack if they haven't attacked yet, or if the cycle has come back
    /// around to their attack time.
    ///
    /// **Window is `(last_time, current_time]`** — strictly greater than
    /// `last_time`. If we used `>=`, then immediately after `record_attack`
    /// (which sets `attack_time = current_time`) the next frame's window
    /// `[last=current, new_current]` would still contain `attack_time`,
    /// causing the unit to fire every frame until the cycle wraps — a
    /// 60-hits-per-second instakill in tight melee. The strict `>` makes
    /// the unit's slot fire exactly once per cycle.
    pub fn can_attack(&self, current_time: f32, last_time: f32) -> bool {
        match self.last_attack_time {
            None => true, // Never attacked, can attack immediately
            Some(attack_time) => {
                // Check if we've cycled past the attack time
                // Handle wrap-around: if current < last, we wrapped around
                if current_time < last_time {
                    // We wrapped, check if attack_time is in the wrapped portion
                    attack_time > last_time || attack_time <= current_time
                } else {
                    // Normal case: check if we're in the window since last update
                    attack_time > last_time && attack_time <= current_time
                }
            }
        }
    }

    /// Records that the unit attacked at this time offset in the cycle.
    pub fn record_attack(&mut self, current_time: f32) {
        self.last_attack_time = Some(current_time);
    }

    /// Like [`can_attack`](Self::can_attack), but for a unit with an attack-speed
    /// buff. Instead of firing once when the cycle sweeps its slot, the unit may
    /// fire every `cycle_duration / (1 + attack_speed_bonus)` of cycle time since
    /// its last swing (e.g. +30% → every 2.0/1.3 ≈ 1.54s instead of 2.0s).
    ///
    /// A `bonus <= 0.0` falls back to the slot-based `can_attack` (and avoids any
    /// divide-by-zero: the `1.0 + bonus` divisor is only used when `bonus > 0`).
    ///
    /// Attack speed must NOT be implemented by widening `can_attack`'s lookback
    /// window: that pushes the window's lower bound behind the attack time
    /// recorded last frame, so the slot stays inside the window every frame and
    /// the unit re-fires ~60×/sec — the instakill `can_attack` warns about. That
    /// is what made attack-speed buffs (Battle Hymn, Haste, Frenzy, …) one-shot.
    pub fn can_attack_with_speed_bonus(
        &self,
        current_time: f32,
        last_time: f32,
        cycle_duration: f32,
        attack_speed_bonus: f32,
    ) -> bool {
        if attack_speed_bonus <= 0.0 {
            return self.can_attack(current_time, last_time);
        }
        match self.last_attack_time {
            None => true,
            Some(last) => {
                let elapsed = if current_time >= last {
                    current_time - last
                } else {
                    cycle_duration - last + current_time
                };
                elapsed >= cycle_duration / (1.0 + attack_speed_bonus)
            }
        }
    }
}

/// Hitbox component for all units.
///
/// Represents a cylindrical collision volume for the unit.
/// The cylinder's radius determines the width of the billboard sprite,
/// and the height determines the sprite's height. The cylinder provides
/// depth for 3D collision detection while the billboard renders at the center.
#[derive(Component, Clone, Copy)]
pub struct Hitbox {
    /// Radius of the cylinder (determines sprite width).
    pub radius: f32,
    /// Height of the cylinder (determines sprite height).
    pub height: f32,
}

impl Hitbox {
    /// Creates a new Hitbox with the given radius and height.
    pub const fn new(radius: f32, height: f32) -> Self {
        Self { radius, height }
    }
}

/// Flat bonus added to a unit's melee attack range (world units).
#[derive(Component)]
pub struct MeleeRangeBonus(pub f32);

/// Reduces incoming melee (non-spell) damage by a multiplier. Read by
/// `combat_systems::melee::combat`. Used by the Ogre boss and the
/// Swordcerer's field avatar.
#[derive(Component)]
pub struct MeleeDamageReduction {
    /// Fraction of damage taken (0.3 = takes 30%, blocks 70%).
    pub multiplier: f32,
}

/// Temporary hit points that absorb damage before real health.
///
/// Expires after a duration and is consumed before health when taking damage.
/// Multiple applications of temporary HP do not stack - only the maximum is kept.
#[derive(Component)]
pub struct TemporaryHitPoints {
    /// Current amount of temporary HP.
    pub amount: f32,
    /// Time remaining before temp HP expires (in seconds).
    pub time_remaining: f32,
}

impl TemporaryHitPoints {
    /// Creates new temporary hit points with a duration.
    pub const fn new(amount: f32, duration: f32) -> Self {
        Self {
            amount,
            time_remaining: duration,
        }
    }

    /// Absorbs damage from temporary HP, returning overflow damage.
    ///
    /// Returns the amount of damage that wasn't absorbed (overflow to real HP).
    pub fn absorb_damage(&mut self, damage: f32) -> f32 {
        if self.amount >= damage {
            self.amount -= damage;
            0.0 // All damage absorbed
        } else {
            let overflow = damage - self.amount;
            self.amount = 0.0;
            overflow // This much damage overflows to real HP
        }
    }

    /// Updates the timer, returning true if expired.
    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0 || self.amount <= 0.0
    }
}

impl Health {
    /// Creates a new Health component with the given maximum health.
    ///
    /// Current health starts at the maximum value.
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
            spell_vulnerability: 0.0,
            healing_reduction: 0.0,
        }
    }

    /// Returns true if the unit is dead (current health <= 0).
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    /// Takes damage, reducing current health.
    ///
    /// Current health is clamped to not go below 0.
    pub fn take_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    /// Heals the unit, increasing current health.
    ///
    /// Current health is clamped to not exceed max health.
    pub fn heal(&mut self, amount: f32) {
        let effective = amount * (1.0 - self.healing_reduction).max(0.0);
        self.current = (self.current + effective).min(self.max);
    }
}

/// Applies damage to a unit, absorbing with temporary HP first.
///
/// This function should be used instead of directly calling `health.take_damage()`
/// when temporary hit points should be respected. Damage is first absorbed by
/// temporary HP (if present), and any overflow damage is applied to real health.
///
/// # Arguments
///
/// * `health` - The unit's Health component
/// * `temp_hp` - Optional TemporaryHitPoints component
/// * `damage` - Amount of damage to apply
pub fn apply_damage_to_unit(
    health: &mut Health,
    temp_hp: Option<&mut TemporaryHitPoints>,
    damage: f32,
) {
    let overflow = if let Some(temp) = temp_hp {
        temp.absorb_damage(damage)
    } else {
        damage
    };

    health.take_damage(overflow);
}

/// Invulnerability status effect — prevents all damage while attached.
/// Health is snapshotted each frame; any damage taken is undone.
#[derive(Component)]
pub struct Invulnerable {
    /// Health value to restore to each frame (damage negation).
    pub health_snapshot: f32,
}

/// Marker component for units that have been damaged by a spell.
/// Used to detect friendly fire for the achievement system.
#[derive(Component)]
pub struct SpellDamaged;

/// Marker component for units that have been damaged by residual fire (ground fire).
/// Used to detect deaths from fireball ground fire for the Scorched Earth achievement.
#[derive(Component)]
pub struct ResidualFireDamaged;

/// Stores the original shared material handle before persistent effect tinting.
///
/// Inserted when a persistent damage effect (FireDoT, FrostEffectMarker, Shocked)
/// is first applied to a unit. The unit's MeshMaterial3d is replaced with a cloned
/// per-entity material that can be safely tinted without affecting other units.
/// When all effects expire, the original material is restored and this component is removed.
#[derive(Component)]
pub struct OriginalMaterial(pub Handle<StandardMaterial>);

/// Visual-only markers for status effects active on the remote peer.
///
/// These are inserted/removed based on network snapshot flags so that
/// `update_persistent_effect_visuals` can tint units without creating
/// damage-ticking DoT components (which would double-count damage via CRDT).
#[derive(Component)]
pub struct RemoteFireEffect;

#[derive(Component)]
pub struct RemoteFrostEffect;

#[derive(Component)]
pub struct RemoteElectricEffect;

/// Visual-only poison marker, mirrored from the host's `PoisonedModifier` via
/// `UnitFlags::POISON_EFFECT`. Drives the green tint in
/// `update_persistent_effect_visuals` without inserting a damage-ticking
/// `PoisonedModifier` (which would double-count poison DoT on the guest).
#[derive(Component)]
pub struct RemotePoisonEffect;

/// Visual-only polymorph marker, mirrored from the host's `PolymorphedModifier`
/// via `UnitFlags::POLYMORPH`. Tracks that a guest ghost is currently rendered as
/// a sheep so the snapshot loop swaps the mesh/material to the sheep sprite on the
/// off→on edge and restores the original sprite on the on→off edge. No gameplay
/// component is created on the guest — the sheep state is host-authoritative.
#[derive(Component)]
pub struct RemotePolymorphEffect;

/// Marker component inserted by `apply_spell_damage` to defer persistent effect stacking.
///
/// A central system (`process_pending_damage_effects`) reads these each frame and
/// creates/stacks the real `FireDoT`, `SlowMovementModifier`, or `Shocked` components.
#[derive(Component)]
pub struct PendingDamageEffect {
    pub damage_type: DamageType,
    pub damage: f32,
}

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

/// Applies spell damage to a unit and inserts a `PendingDamageEffect` marker
/// so the persistent effect system can stack the appropriate effect.
///
/// This replaces the pattern of calling `apply_damage_to_unit()` + inserting
/// `SpellDamaged` manually at each spell call site.
pub fn apply_spell_damage(
    commands: &mut Commands,
    entity: Entity,
    health: &mut Health,
    temp_hp: Option<&mut TemporaryHitPoints>,
    damage: f32,
    damage_type: DamageType,
    has_spell_shield: bool,
) {
    if has_spell_shield {
        return;
    }
    let modified_damage = damage * (1.0 + health.spell_vulnerability);
    apply_damage_to_unit(health, temp_hp, modified_damage);
    commands.entity(entity).insert(SpellDamaged);
    commands.entity(entity).insert(PendingDamageEffect {
        damage_type,
        damage: modified_damage,
    });
}

// Animation components moved to animation.rs (Phase 9)

#[derive(Component)]
pub struct Corpse;

/// Marker component for permanent corpses that cannot be resurrected.
///
/// Applied to undead corpses to prevent them from being raised again.
#[derive(Component)]
pub struct PermanentCorpse;

/// Marker component for units that can be teleported.
///
/// Applied to all combat units (defenders, attackers, undead) but not the wizard.
#[derive(Component)]
pub struct Teleportable;

/// Marker component for flying units that render above the battlefield.
///
/// Flying units ignore wall obstacles (no wall avoidance, collision, or LOS suppression)
/// and cannot be targeted by melee ground units (infantry, king, kingsguard).
/// Archers can still target and hit flying units.
#[derive(Component)]
pub struct Flying;

/// Shadow entity that tracks a unit's XZ position at ground level.
#[derive(Component)]
pub struct UnitShadow {
    pub owner: Entity,
}

/// Marker on a unit that already has a shadow spawned for it.
#[derive(Component)]
pub struct HasShadow;

/// Component that slows units walking over rough terrain (corpses).
///
/// Applied to corpses to create a movement penalty for living units that walk over them.
#[derive(Component)]
pub struct RoughTerrain {
    /// Movement speed multiplier (0.0 = no movement, 1.0 = full speed).
    /// For example, 0.6 means units move at 60% of their normal speed.
    pub slowdown_factor: f32,
}

/// Effectiveness coefficient applied to movement speed and attack damage.
///
/// Dynamically calculated based on:
/// - Number of allies in melee range (positive effect)
/// - Number of enemies in melee range (negative effect)
/// - Spell-based modifiers (future feature)
///
/// Units with allies nearby become more effective; units surrounded by enemies
/// become less effective. This creates strategic depth and rewards good positioning.
#[derive(Component, Clone, Copy)]
pub struct Effectiveness {
    /// Current effectiveness multiplier (applied to speed and damage).
    pub current: f32,
    /// Base effectiveness before any modifiers (always 1.0).
    pub base: f32,
    /// Bonus from spell effects (future feature).
    pub spell_bonus: f32,
}

impl Effectiveness {
    /// Creates a new Effectiveness component with default values.
    pub const fn new() -> Self {
        Self {
            current: 1.0,
            base: 1.0,
            spell_bonus: 0.0,
        }
    }

    /// Recalculates effectiveness based on proximity modifiers and spell bonuses.
    ///
    /// Formula: current = clamp(base + proximity_modifier + spell_bonus, MIN, MAX)
    ///
    /// # Arguments
    /// * `ally_count` - Number of allies in melee range
    /// * `enemy_count` - Number of enemies in melee range
    pub fn recalculate(&mut self, ally_count: i32, enemy_count: i32) {
        use crate::game::constants::{
            EFFECTIVENESS_ALLY_BONUS_PER_UNIT, EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT,
            EFFECTIVENESS_MAX, EFFECTIVENESS_MIN,
        };

        let proximity_modifier = (ally_count as f32 * EFFECTIVENESS_ALLY_BONUS_PER_UNIT)
            + (enemy_count as f32 * EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT);

        self.current = (self.base + proximity_modifier + self.spell_bonus)
            .clamp(EFFECTIVENESS_MIN, EFFECTIVENESS_MAX);
    }

    /// Returns the current effectiveness multiplier.
    pub fn multiplier(&self) -> f32 {
        self.current
    }
}

impl Default for Effectiveness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::constants::{
        EFFECTIVENESS_ALLY_BONUS_PER_UNIT, EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT, EFFECTIVENESS_MAX,
        EFFECTIVENESS_MIN,
    };

    #[test]
    fn test_effectiveness_base_values() {
        let eff = Effectiveness::new();
        assert_eq!(eff.current, 1.0);
        assert_eq!(eff.base, 1.0);
        assert_eq!(eff.spell_bonus, 0.0);
    }

    #[test]
    fn test_effectiveness_default() {
        let eff = Effectiveness::default();
        assert_eq!(eff.current, 1.0);
        assert_eq!(eff.base, 1.0);
        assert_eq!(eff.spell_bonus, 0.0);
    }

    #[test]
    fn test_effectiveness_ally_bonus() {
        let mut eff = Effectiveness::new();
        eff.recalculate(3, 0); // 3 allies, 0 enemies
        assert_eq!(eff.current, 1.0 + 3.0 * EFFECTIVENESS_ALLY_BONUS_PER_UNIT);
    }

    #[test]
    fn test_effectiveness_enemy_penalty() {
        let mut eff = Effectiveness::new();
        eff.recalculate(0, 2); // 0 allies, 2 enemies
        assert_eq!(
            eff.current,
            1.0 + 2.0 * EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT
        );
    }

    #[test]
    fn test_effectiveness_mixed() {
        let mut eff = Effectiveness::new();
        eff.recalculate(2, 1); // 2 allies, 1 enemy
        let expected = 1.0
            + 2.0 * EFFECTIVENESS_ALLY_BONUS_PER_UNIT
            + 1.0 * EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT;
        assert_eq!(eff.current, expected);
    }

    #[test]
    fn test_effectiveness_clamping_min() {
        let mut eff = Effectiveness::new();
        eff.recalculate(0, 10); // Many enemies
        assert_eq!(eff.current, EFFECTIVENESS_MIN);
    }

    #[test]
    fn test_effectiveness_clamping_max() {
        let mut eff = Effectiveness::new();
        eff.recalculate(20, 0); // Many allies
        assert_eq!(eff.current, EFFECTIVENESS_MAX);
    }

    #[test]
    fn test_effectiveness_with_spell_bonus() {
        let mut eff = Effectiveness::new();
        eff.spell_bonus = 0.5;
        eff.recalculate(0, 0); // No proximity modifiers
        assert_eq!(eff.current, 1.0 + 0.5);
    }

    #[test]
    fn test_effectiveness_multiplier() {
        let mut eff = Effectiveness::new();
        eff.recalculate(2, 1);
        assert_eq!(eff.multiplier(), eff.current);
    }
}

// Status effect components moved to status_effects.rs (Phase 9)

/// Component indicating a unit is currently engaged in melee combat with a specific team.
///
/// A unit is considered in melee when there is an enemy within melee range.
/// This is used by archers to avoid friendly fire - they won't target units in melee
/// with someone on their own team.
#[derive(Component)]
pub struct InMelee(pub Team);

/// Targeting velocity toward target, set by the targeting system.
///
/// The targeting system calculates this based on the nearest enemy.
/// This is a normalized direction vector with distance information for weighting.
#[derive(Component, Default)]
pub struct TargetingVelocity {
    pub velocity: Vec3,
    pub distance_to_target: f32,
}

/// Per-unit multipliers for flocking forces.
///
/// Units without this component default to 1.0 for all forces.
/// Set individual fields to 0.0 to disable that force for a unit.
#[derive(Component)]
pub struct FlockingModifier {
    pub separation: f32,
    pub alignment: f32,
    pub cohesion: f32,
}

impl FlockingModifier {
    pub const fn new(separation: f32, alignment: f32, cohesion: f32) -> Self {
        Self {
            separation,
            alignment,
            cohesion,
        }
    }
}

/// King's Guard unit. Stores the slot index for positioning around the King.
#[derive(Component)]
pub struct KingsGuard(pub u32);

/// Flocking velocity from separation, alignment, and cohesion forces.
///
/// The flocking system calculates this based on nearby allies.
/// This is a normalized direction vector.
#[derive(Component, Default)]
pub struct FlockingVelocity {
    pub velocity: Vec3,
    /// Direct repulsion force from smelly units, applied as raw acceleration
    /// (bypasses the weighted flocking normalization).
    pub smelly_repulsion: Vec3,
}

/// Mind control effect — unit targets allies instead of enemies.
/// Used by both the Hag boss (Martina) and the player's Mind Control spell.
#[derive(Component)]
pub struct MindControlled {
    /// Time elapsed since mind control was applied.
    pub time_elapsed: f32,
    /// Duration before mind control wears off.
    pub wear_off_duration: f32,
    /// Original defender spawn position for restoring flow field on wear-off.
    pub original_spawn_pos: Option<Vec2>,
    /// Damage multiplier for controlled unit's attacks (Deep Domination talent).
    pub damage_multiplier: f32,
}

/// Marks a unit as wanting to retaliate against a specific entity.
/// Inserted when a mind-controlled unit attacks a same-team ally, causing
/// the victim to consider the attacker a valid target despite being on the same team.
#[derive(Component)]
pub struct RetaliationTarget(pub Entity);

/// Airborne state for units launched into the air (geyser, explosions, etc.).
/// Applies gravity, offsets Y visually, and deals velocity-based fall damage on landing.
/// The unit is rooted separately via `RootedModifier` during flight.
#[derive(Component)]
pub struct Airborne {
    /// Current vertical velocity (positive = upward).
    pub vertical_velocity: f32,
    /// Current vertical offset from ground.
    pub height: f32,
    /// The unit's original Y position before launch (restored on landing).
    pub base_y: f32,
    /// Gravity acceleration applied per second (units/s²).
    pub gravity: f32,
    /// Damage type to apply on landing.
    pub damage_type: DamageType,
}

impl Airborne {
    /// Creates a new airborne state with the given launch velocity, gravity, and base Y.
    pub fn new(launch_velocity: f32, gravity: f32, base_y: f32, damage_type: DamageType) -> Self {
        Self {
            vertical_velocity: launch_velocity,
            height: 0.0,
            base_y,
            gravity,
            damage_type,
        }
    }
}

/// Damage scale factor for fall damage: `damage = abs(impact_velocity) * FALL_DAMAGE_SCALE`.
/// Calibrated so a geyser launch (200 velocity, 120 gravity → ~200 impact velocity) deals 15 damage.
pub const FALL_DAMAGE_SCALE: f32 = 0.075;

/// Knockback effect that moves a unit outward over time with decay.
/// Applied by ogre melee attacks, hag leaps, and meteor aftershock.
/// Decays linearly for a "tumbling through dirt" feel.
#[derive(Component)]
pub struct Knockback {
    /// Direction of knockback (normalized XZ).
    pub direction_x: f32,
    pub direction_z: f32,
    /// Initial knockback speed (units/s at full strength).
    pub speed: f32,
    /// Total duration of the effect.
    pub duration: f32,
    /// Time remaining before the effect expires.
    pub remaining: f32,
}

impl Knockback {
    pub fn new(direction: Vec3, speed: f32, duration: f32) -> Self {
        let normalized = direction.normalize_or_zero();
        Self {
            direction_x: normalized.x,
            direction_z: normalized.z,
            speed,
            duration,
            remaining: duration,
        }
    }
}

pub use super::elite::{EliteAttackSpeedBonus, EliteDamageBonus, EliteSpeedBonus};

/// Persistent color glow for special unit types (dispeller, healer, shielder, commander, brute).
///
/// The visual system uses this to apply a pulsing color tint to the unit's material.
/// This is separate from elite glow and shield buff glow, allowing them to stack.
#[derive(Component, Clone)]
pub struct UnitTypeGlow {
    pub color: Color,
}
