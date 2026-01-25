use bevy::prelude::*;

/// Team component for all units.
///
/// Determines which side a unit is on. Units attack members of opposing teams.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Team {
    Defenders,
    Attackers,
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
pub struct MovementSpeed {
    pub speed: f32,
}

impl MovementSpeed {
    /// Creates a new MovementSpeed component.
    pub const fn new(speed: f32) -> Self {
        Self { speed }
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
