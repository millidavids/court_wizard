//! Constants for the Teleport spell.

use crate::game::units::wizard::components::PrimedSpell;
use crate::game::units::wizard::components::Spell;

/// Primed Teleport spell configuration.
pub const PRIMED_TELEPORT: PrimedSpell = PrimedSpell {
    spell: Spell::Teleport,
    cast_time: 0.0, // First cast is instant (places crosshair immediately)
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

/// Second cast time for source circle and teleportation.
pub const SECOND_CAST_TIME: f32 = 2.0;

/// Mana cost for teleportation (charged on second cast).
pub const MANA_COST: f32 = 20.0;

/// Radius of source circle (grows during channeling).
pub const CIRCLE_RADIUS: f32 = 150.0;

/// Radius of the destination crosshair (small marker).
pub const CROSSHAIR_RADIUS: f32 = 30.0;

/// Scale threshold at which pulse animation begins (prevents pulsing during growth).
pub const PULSE_THRESHOLD: f32 = 0.9;

// ===== Talent Constants =====

// Tier 1
/// Wide Aperture: source circle radius multiplier (+50%).
pub const WIDE_APERTURE_RADIUS_MULT: f32 = 1.5;
/// Hasty Translocation: second cast time multiplier (-40%).
pub const HASTY_TRANSLOCATION_CAST_TIME_MULT: f32 = 0.6;
/// Lingering Gate: duration the destination persists for a second teleport (seconds).
pub const LINGERING_GATE_DURATION: f32 = 5.0;

// Tier 2
/// Disorienting Arrival: stun duration for enemies on arrival (seconds).
pub const DISORIENTING_STUN_DURATION: f32 = 2.0;
/// Disorienting Arrival: attack speed bonus for allies on arrival.
pub const DISORIENTING_ATTACK_SPEED: f32 = 0.2;
/// Disorienting Arrival: duration of the attack speed buff (seconds).
pub const DISORIENTING_ATTACK_SPEED_DURATION: f32 = 3.0;

// Tier 3
/// Dimensional Rift: how long the two-way portal persists (seconds).
pub const DIMENSIONAL_RIFT_DURATION: f32 = 10.0;
/// Dimensional Rift: walk-through detection radius around each portal end.
pub const DIMENSIONAL_RIFT_WALK_RADIUS: f32 = 40.0;
/// Dimensional Rift: cooldown per unit after being teleported by rift (seconds).
pub const DIMENSIONAL_RIFT_UNIT_COOLDOWN: f32 = 2.0;
/// Up: height units are teleported to (world units above ground).
pub const TELEPORT_UP_HEIGHT: f32 = 300.0;
/// Up: gravity acceleration for falling units.
pub const TELEPORT_UP_GRAVITY: f32 = 120.0;
/// Scatterport: scatter radius (large area).
pub const SCATTERPORT_RADIUS: f32 = 400.0;
/// Scatterport: mana cost multiplier (cheaper).
pub const SCATTERPORT_MANA_MULT: f32 = 0.5;
