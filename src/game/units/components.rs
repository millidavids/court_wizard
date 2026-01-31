use bevy::prelude::*;

/// Team component for all units.
///
/// Determines which side a unit is on. Units attack members of opposing teams.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Team {
    Defenders,
    Attackers,
    Undead,
}

/// Health component for all units.
///
/// Tracks the current and maximum health of a unit.
#[derive(Component)]
#[allow(dead_code)]
pub struct Health {
    pub current: f32,
    pub max: f32,
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

/// Movement speed modifier from King's aura as a percentage.
///
/// Applied to defenders within the King's aura range.
/// Examples: 0.25 = +25% speed from King's aura.
/// Movement systems apply this as: speed * (1.0 + sum_of_all_modifiers).
#[derive(Component)]
pub struct KingAuraSpeedModifier(pub f32);

/// Movement speed modifier from rough terrain as a percentage.
///
/// Applied to units walking over corpses.
/// Examples: -0.6 = -60% speed (0.4x multiplier).
/// Movement systems apply this as: speed * (1.0 + sum_of_all_modifiers).
#[derive(Component)]
pub struct RoughTerrainModifier(pub f32);

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
    pub fn can_attack(&self, current_time: f32, last_time: f32) -> bool {
        match self.last_attack_time {
            None => true, // Never attacked, can attack immediately
            Some(attack_time) => {
                // Check if we've cycled past the attack time
                // Handle wrap-around: if current < last, we wrapped around
                if current_time < last_time {
                    // We wrapped, check if attack_time is in the wrapped portion
                    attack_time >= last_time || attack_time <= current_time
                } else {
                    // Normal case: check if we're in the window since last update
                    attack_time >= last_time && attack_time <= current_time
                }
            }
        }
    }

    /// Records that the unit attacked at this time offset in the cycle.
    pub fn record_attack(&mut self, current_time: f32) {
        self.last_attack_time = Some(current_time);
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

#[allow(dead_code)]
impl Hitbox {
    /// Creates a new Hitbox with the given radius and height.
    pub const fn new(radius: f32, height: f32) -> Self {
        Self { radius, height }
    }

    /// Returns the sprite width (diameter of the cylinder).
    pub fn sprite_width(&self) -> f32 {
        self.radius * 2.0
    }

    /// Returns the sprite height.
    pub fn sprite_height(&self) -> f32 {
        self.height
    }

    /// Checks if this hitbox overlaps with another hitbox in 3D space.
    ///
    /// Uses cylinder-cylinder collision detection:
    /// - Checks distance in XZ plane (horizontal) against combined radii
    /// - Checks overlap in Y axis (vertical) based on heights and positions
    pub fn overlaps(&self, self_pos: Vec3, other: &Hitbox, other_pos: Vec3) -> bool {
        // Check horizontal distance (XZ plane)
        let horizontal_dist_sq =
            (self_pos.x - other_pos.x).powi(2) + (self_pos.z - other_pos.z).powi(2);
        let combined_radius = self.radius + other.radius;

        if horizontal_dist_sq > combined_radius.powi(2) {
            return false; // Too far apart horizontally
        }

        // Check vertical overlap (Y axis)
        // Assuming positions are at the base of the cylinders
        let self_top = self_pos.y + self.height;
        let self_bottom = self_pos.y;
        let other_top = other_pos.y + other.height;
        let other_bottom = other_pos.y;

        // Cylinders overlap vertically if one's bottom is below the other's top
        self_bottom < other_top && other_bottom < self_top
    }
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

#[allow(dead_code)]
impl Health {
    /// Creates a new Health component with the given maximum health.
    ///
    /// Current health starts at the maximum value.
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
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
        self.current = (self.current + amount).min(self.max);
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

/// Marker component for dead units (corpses).
///
/// Dead units remain on the battlefield as corpses that affect living units.
/// Corpses don't move, attack, or collide, but they slow down units walking over them.
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
}
