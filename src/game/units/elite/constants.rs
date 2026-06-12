/// Elite health bonus (raw HP value, not percentage).
///
/// Base unit HP is typically 100, so this gives +50% health (150 total).
pub const ELITE_HEALTH_BONUS: f32 = 50.0;

/// Elite damage bonus as percentage.
///
/// Example: 0.3 = +30% damage increase.
pub const ELITE_DAMAGE_BONUS: f32 = 0.3;

/// Elite speed bonus as percentage.
///
/// Example: 0.2 = +20% movement speed increase.
pub const ELITE_SPEED_BONUS: f32 = 0.2;

/// Elite attack speed bonus as percentage.
///
/// Example: 0.25 = +25% attack speed (0.8x cycle duration).
/// Applied via upgrade_systems and queried during combat resolution.
pub const ELITE_ATTACK_SPEED_BONUS: f32 = 0.25;
