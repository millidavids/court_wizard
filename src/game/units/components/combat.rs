use bevy::prelude::*;

use super::team::Team;

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

/// Damage bonus as a percentage.
///
/// Used by special units and buffs to modify damage output.
/// Examples: 0.5 = +50% damage, 1.0 = +100% damage (double), -0.4 = -40% damage.
/// Combat system applies this as: damage * (1.0 + percentage).
#[derive(Component)]
pub struct DamageMultiplier(pub f32);

/// Component indicating a unit is currently engaged in melee combat with a specific team.
///
/// A unit is considered in melee when there is an enemy within melee range.
/// This is used by archers to avoid friendly fire - they won't target units in melee
/// with someone on their own team.
#[derive(Component)]
pub struct InMelee(pub Team);
