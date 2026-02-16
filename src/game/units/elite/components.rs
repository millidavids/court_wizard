use bevy::prelude::*;

/// Elite health bonus component.
///
/// Applied once when unit becomes elite, increases max HP and current HP.
/// The value is a raw HP amount, not a percentage.
///
/// # Example
/// ```
/// EliteHealthBonus(25.0) // +25 HP
/// ```
#[derive(Component, Clone, Copy)]
pub struct EliteHealthBonus(pub f32);

/// Elite damage bonus as a percentage.
///
/// Combat system applies this as: `damage * (1.0 + percentage)`.
///
/// # Example
/// ```
/// EliteDamageBonus(0.3) // +30% damage
/// ```
#[derive(Component, Clone, Copy)]
pub struct EliteDamageBonus(pub f32);

/// Elite movement speed modifier as a percentage.
///
/// Movement systems apply this as: `speed * (1.0 + sum_of_all_modifiers)`.
///
/// # Example
/// ```
/// EliteSpeedBonus(0.2) // +20% movement speed
/// ```
#[derive(Component, Clone, Copy)]
pub struct EliteSpeedBonus(pub f32);
