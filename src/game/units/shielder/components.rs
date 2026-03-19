use bevy::prelude::*;

/// Marker component for shielder units.
#[derive(Component)]
pub struct Shielder;

/// Cooldown timer for the shielder's shield application ability.
#[derive(Component)]
pub struct ShielderShieldCooldown {
    /// Time remaining before next shield can be applied (seconds).
    pub remaining: f32,
}

/// Marker component for the spell shield damage reduction effect.
/// When present on a unit alongside SpellShield, reduces all non-spell
/// damage by 20% (melee attacks, ground effect AOEs).
#[derive(Component)]
pub struct ShielderDamageReduction;
