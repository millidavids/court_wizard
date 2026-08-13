//! Arcane Crystal spell constants.

use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// PrimedSpell constant for Arcane Crystal.
pub const PRIMED_ARCANE_CRYSTAL: PrimedSpell = PrimedSpell {
    spell: Spell::ArcaneCrystal,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

// ===== Casting & Mana =====

/// Cast time before crystal spawns (in seconds).
pub const CAST_TIME: f32 = 2.0;

/// Mana cost to place the crystal.
pub const MANA_COST: f32 = 35.0;

// ===== Crystal Properties =====

/// Total lifetime of the crystal (in seconds).
pub const CRYSTAL_DURATION: f32 = 25.0;

/// Range within which the crystal targets enemies and limits projectiles.
pub const CRYSTAL_RANGE: f32 = 500.0;

/// Collision radius for detecting incoming spell hits.
pub const CRYSTAL_COLLISION_RADIUS: f32 = 20.0;

/// Visual height of the crystal mesh.
pub const CRYSTAL_HEIGHT: f32 = 35.0;

// ===== Scaling =====

/// Global damage scale for all crystal emissions (fraction of original spell damage).
pub const DAMAGE_SCALE: f32 = 0.5;

/// Global size scale for crystal emissions (fraction of original spell size).
pub const SIZE_SCALE: f32 = 0.5;

/// Speed scale for crystal projectiles (fraction of original speed).
pub const SPEED_SCALE: f32 = 0.7;

/// Number of mini fireballs emitted per absorbed fireball.
pub const MINI_FB_COUNT: usize = 5;

/// Number of mini meteors emitted per absorbed meteor.
pub const MINI_METEOR_COUNT: usize = 2;

/// Duration of crystal-emitted beams (seconds).
pub const BEAM_DURATION: f32 = 0.5;

/// Number of lesser beams emitted by the crystal for disintegrate and finger of death.
pub const BEAM_COUNT: usize = 5;

/// Damage scale for crystal beams (50% of original).
pub const BEAM_DAMAGE_SCALE: f32 = 0.5;

/// Flat reference damage for crystal-echoed Finger of Death beams. The real
/// spell now deals a fraction of each target's max health, but crystal echoes
/// are disintegrate-style tick beams, so they keep the spell's old flat value.
pub const FOD_ECHO_BASE_DAMAGE: f32 = 500.0;

/// Half-angle of the forked fan spread for crystal beams (radians, ~7.5 degrees).
pub const FORKED_FAN_HALF_ANGLE: f32 = 0.13;

/// Number of targets for crystal lightning arcs.
pub const LIGHTNING_ARC_COUNT: usize = 3;

/// Height above crystal from which mini meteors are launched.
pub const MINI_METEOR_SPAWN_HEIGHT: f32 = 200.0;

// ===== Visual =====

/// Rotation speed of the crystal (radians/sec).
pub const ROTATION_SPEED: f32 = 0.5;

/// Scale factor for absorption pulse animation.
pub const PULSE_SCALE: f32 = 1.3;

/// Duration of absorption pulse animation.
pub const PULSE_DURATION: f32 = 0.15;

// ===== Mini Missiles =====

/// Number of mini magic missiles emitted per absorbed missile.
pub const MINI_MISSILE_COUNT: usize = 5;

/// Pre-advance time_alive on crystal-spawned missiles so homing is nearly perfect immediately.
/// Magic missile PERFECT_TRACKING_TIME is 5.0s; this skips most of the ramp.
pub const MINI_MISSILE_HOMING_ADVANCE: f32 = 4.0;

// ===== Auto-Cast =====

/// Base reference damage for auto-cast interval scaling.
/// Spells with this damage get an interval of AUTO_CAST_BASE_INTERVAL.
pub const AUTO_CAST_REFERENCE_DAMAGE: f32 = 20.0;

/// Base auto-cast interval (seconds) for a spell dealing REFERENCE_DAMAGE.
pub const AUTO_CAST_BASE_INTERVAL: f32 = 3.0;

/// Minimum auto-cast interval (floor, seconds).
pub const AUTO_CAST_MIN_INTERVAL: f32 = 1.5;

/// Maximum auto-cast interval (ceiling, seconds).
pub const AUTO_CAST_MAX_INTERVAL: f32 = 20.0;

// ===== Infusion Tuning =====

/// Default number of sub-effects a non-emitter infusion's burst produces.
pub const INFUSION_BURST_COUNT: usize = 4;

/// Number of targets a non-emitter infusion affects on an ongoing tick.
pub const INFUSION_ONGOING_COUNT: usize = 2;

/// Fraction of an absorbed spell's duration that a crystal burst reproduces.
/// Matches `DAMAGE_SCALE`: the crystal echoes a spell at half strength.
pub const INFUSION_DURATION_SCALE: f32 = 0.5;

/// Fraction of the crystal's range used when an infusion spawns a scaled-down
/// copy of a zone spell, so patches read as smaller than the real cast.
pub const INFUSION_ZONE_RADIUS_SCALE: f32 = 0.35;

/// Damage dealt when Dispel shatters a fully-charged crystal. Scaled down by how
/// much of the crystal's lifetime has already elapsed.
pub const SHATTER_BASE_DAMAGE: f32 = 200.0;

/// Radius of the Dispel shatter burst.
pub const SHATTER_RADIUS: f32 = 320.0;

// ===== Infusion Cadence =====
//
// Seconds between activations for each `InfusionFamily::Ticked` infusion.
// Continuous infusions have no interval; emitters derive theirs from damage.

/// Grease: interval between new slick patches.
pub const GREASE_INFUSION_INTERVAL: f32 = 5.0;
/// Teleport infusion: radius of the ring pulled units land on, as a fraction of
/// the crystal's range.
pub const TELEPORT_DROP_RING_SCALE: f32 = 0.15;

/// Black Hole infusion: cap on the crystal's inward pull, well below a real
/// black hole's so it gathers units instead of crushing them into the centre.
pub const CRYSTAL_GRAVITY_MAX_PULL: f32 = 1200.0;

/// Squall: radius of the storm the crystal sustains, as a fraction of its range.
pub const SQUALL_INFUSION_RADIUS_SCALE: f32 = 0.8;
/// Spike Growth: interval between new spike zones.
pub const SPIKE_GROWTH_INFUSION_INTERVAL: f32 = 4.0;
/// Plague Wind: interval between radial clouds.
pub const PLAGUE_WIND_INFUSION_INTERVAL: f32 = 6.0;
/// Raise The Dead: interval between corpse raisings.
pub const RAISE_THE_DEAD_INFUSION_INTERVAL: f32 = 3.0;
/// Guardian Circle: interval between temporary-HP refreshes.
pub const GUARDIAN_CIRCLE_INFUSION_INTERVAL: f32 = 6.0;
/// Entangle: interval between root snares.
pub const ENTANGLE_INFUSION_INTERVAL: f32 = 4.0;
/// Sleep: interval between sleep pulses.
pub const SLEEP_INFUSION_INTERVAL: f32 = 5.0;
/// Fog Cloud: interval between fog refreshes.
pub const FOG_CLOUD_INFUSION_INTERVAL: f32 = 8.0;
/// Banishment: interval between banishments.
pub const BANISHMENT_INFUSION_INTERVAL: f32 = 8.0;
/// How far outside a black hole's horizon a warded crystal is thrown clear when
/// its ward absorbs a consumption. Enough that gravity does not immediately drag
/// it back in on the following frame.
pub const WARD_ESCAPE_MARGIN: f32 = 120.0;

/// How often a buff aura re-applies. Must stay comfortably inside the shortest
/// buff duration it refreshes (Berserker Rage, 8s base) so the aura feels
/// unbroken while costing a fraction of the per-frame work.
pub const AURA_REFRESH_INTERVAL: f32 = 2.0;

/// Telekinesis: interval between drop collections. Deliberately unhurried —
/// the crystal is a background collector, and leaving drops on the ground is
/// what keeps Telekinesis itself castable.
pub const TELEKINESIS_INFUSION_INTERVAL: f32 = 2.0;
/// Teleport: interval between yanking an enemy to the crystal.
pub const TELEPORT_INFUSION_INTERVAL: f32 = 5.0;

// ===== Modifier Constants =====

/// Hastened: divisor applied to the crystal's auto-cast interval.
pub const HASTENED_INTERVAL_DIVISOR: f32 = 2.0;
/// Enraged: emission damage multiplier.
pub const ENRAGED_DAMAGE_MULT: f32 = 1.5;
/// Enraged: how much faster the crystal burns its lifetime.
pub const ENRAGED_LIFETIME_SCALE: f32 = 2.0;
/// Warded: destruction attempts absorbed before the ward breaks.
pub const WARDED_CHARGES: u32 = 1;

/// Default emissive tint of an uninfused crystal (matches the `arcane_crystal`
/// material in `visual_assets.rs`).
pub const CRYSTAL_DEFAULT_EMISSIVE: bevy::color::LinearRgba =
    bevy::color::LinearRgba::new(0.4, 0.05, 0.6, 1.0);

// ===== Talent Constants =====

// --- Tier 1 ---

/// Refined Facets: multiplier for sub-projectile damage.
pub const REFINED_FACETS_DAMAGE_MULT: f32 = 1.25;

/// Wider Prism: multiplier for the crystal's targeting range (not its hit window).
pub const WIDER_PRISM_RANGE_MULT: f32 = 1.4;

/// Enduring Crystal: multiplier for crystal duration.
pub const ENDURING_CRYSTAL_DURATION_MULT: f32 = 1.3;

// --- Tier 2 ---

/// Overcharged Matrix: multiplier for sub-projectile count (rounded up).
pub const OVERCHARGED_MATRIX_COUNT_MULT: f32 = 1.5;

/// Resonance Cascade: number of absorptions needed before burst.
pub const RESONANCE_CASCADE_THRESHOLD: u32 = 3;

/// Resonance Cascade: damage dealt by the burst to each enemy in range.
pub const RESONANCE_CASCADE_DAMAGE: f32 = 80.0;

/// Resonance Cascade: burst radius (centered on crystal).
pub const RESONANCE_CASCADE_RADIUS: f32 = 350.0;

/// Spell Echo: chance to duplicate an absorbed spell (0.0 - 1.0).
pub const SPELL_ECHO_CHANCE: f32 = 0.3;

// --- Tier 3 ---

/// Crystal Network: maximum number of crystals that can exist simultaneously.
pub const CRYSTAL_NETWORK_MAX_CRYSTALS: usize = 3;

/// Crystal Network: range for chaining spell absorptions between crystals.
pub const CRYSTAL_NETWORK_CHAIN_RANGE: f32 = 400.0;

/// Prismatic Explosion: damage dealt by expiry detonation.
pub const PRISMATIC_EXPLOSION_DAMAGE: f32 = 150.0;

/// Prismatic Explosion: radius of the detonation.
pub const PRISMATIC_EXPLOSION_RADIUS: f32 = 300.0;

/// Auto-Crystal: interval between magic missile shots (seconds).
pub const AUTO_CRYSTAL_INTERVAL: f32 = 0.2;
