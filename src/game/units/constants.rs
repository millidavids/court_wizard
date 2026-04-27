//! Shared unit constants.
//!
//! Contains constants used across multiple unit types.

use bevy::prelude::*;

// ===== Fire DoT =====

/// Percentage of spell damage added as DoT DPS per fire hit.
pub const FIRE_DOT_DAMAGE_RATIO: f32 = 0.1;
/// Seconds before DoT expires (resets on each fire hit).
pub const FIRE_DOT_DURATION: f32 = 4.0;
/// Seconds between DoT damage ticks.
pub const FIRE_DOT_TICK_INTERVAL: f32 = 0.5;
/// Maximum DoT damage per tick (prevents runaway damage).
pub const FIRE_DOT_MAX_DPS: f32 = 30.0;

// ===== Frost Accumulation (general frost damage sources) =====

/// Frost accumulation per generic frost damage hit (non-squall sources).
pub const FROST_ACCUMULATION_PER_HIT: f32 = 0.1;
/// Seconds after last hit before frost decay begins (generic sources).
pub const FROST_GENERIC_DECAY_DELAY: f32 = 1.5;

// ===== Electric Arc =====

/// Arc chance added per electric damage hit.
pub const ELECTRIC_ARC_CHANCE_PER_HIT: f32 = 0.05;
/// Arc chance added per point of electric damage dealt.
pub const ELECTRIC_ARC_CHANCE_PER_DAMAGE: f32 = 0.003;
/// Maximum arc chance (60%).
pub const ELECTRIC_ARC_MAX_CHANCE: f32 = 0.6;
/// Damage dealt by each arc.
pub const ELECTRIC_ARC_DAMAGE: f32 = 5.0;
/// Range to find arc targets.
pub const ELECTRIC_ARC_RANGE: f32 = 80.0;
/// Maximum number of targets per arc event.
pub const ELECTRIC_ARC_MAX_TARGETS: usize = 2;
/// Seconds between arc events on the same unit.
pub const ELECTRIC_ARC_COOLDOWN: f32 = 1.0;
/// Seconds before electric charge expires (resets on each electric hit).
pub const ELECTRIC_ARC_DURATION: f32 = 4.0;
/// Color of electric arc visuals.
pub const ELECTRIC_ARC_COLOR: Color = Color::srgb(0.7, 0.85, 1.0);
/// Width of electric arc visuals.
pub const ELECTRIC_ARC_WIDTH: f32 = 4.0;
/// Lifetime of electric arc visuals.
pub const ELECTRIC_ARC_LIFETIME: f32 = 0.2;

// ===== Persistent Effect Visual Tinting =====

#[allow(dead_code)]
pub const FIRE_EFFECT_COLOR: Color = Color::srgb(1.0, 0.35, 0.05);
/// Color overlay for frost slow effect (icy blue).
pub const FROST_EFFECT_COLOR: Color = Color::srgb(0.3, 0.55, 1.0);
/// Color overlay for electric charge effect (yellow-white).
pub const ELECTRIC_EFFECT_COLOR: Color = Color::srgb(1.0, 0.95, 0.5);
#[allow(dead_code)]
pub const FIRE_EFFECT_PULSE_SPEED: f32 = 4.0;
/// Speed of electric effect flickering (radians per second).
pub const ELECTRIC_EFFECT_FLICKER_SPEED: f32 = 12.0;
#[allow(dead_code)]
pub const FIRE_EFFECT_MAX_INTENSITY: f32 = 0.45;
#[allow(dead_code)]
pub const FIRE_EFFECT_MIN_INTENSITY: f32 = 0.15;
/// Max frost tint intensity at full accumulation (scales with frost level).
pub const FROST_EFFECT_MAX_INTENSITY: f32 = 0.6;
/// Maximum blend strength for electric tint.
pub const ELECTRIC_EFFECT_MAX_INTENSITY: f32 = 0.5;
/// Minimum blend strength for electric tint.
pub const ELECTRIC_EFFECT_MIN_INTENSITY: f32 = 0.05;
/// Color overlay for wet status (dark blue-gray, darkens units).
pub const WET_EFFECT_COLOR: Color = Color::srgb(0.15, 0.2, 0.35);
/// Blend strength for wet tint (constant, no pulsing).
pub const WET_EFFECT_INTENSITY: f32 = 0.3;
/// Color overlay for mind control effect (saturated magenta).
pub const MIND_CONTROL_EFFECT_COLOR: Color = Color::srgb(1.0, 0.05, 0.85);
/// Blend strength for mind control tint (constant, no pulsing).
pub const MIND_CONTROL_EFFECT_INTENSITY: f32 = 0.8;

// ===== Poison =====

/// Effectiveness penalty per poison stack.
pub const POISON_EFFECTIVENESS_PER_STACK: f32 = -0.05;
/// Maximum accumulated effectiveness penalty from poison.
pub const POISON_EFFECTIVENESS_CAP: f32 = -0.5;
/// Seconds before poison expires (resets on each poison hit).
pub const POISON_DURATION: f32 = 5.0;
/// Seconds between poison effectiveness penalty applications.
pub const POISON_TICK_INTERVAL: f32 = 0.5;
/// Total accumulated poison needed to trigger sickened.
pub const SICKENED_THRESHOLD: f32 = 0.3;
/// Seconds the sickened stun lasts.
pub const SICKENED_DURATION: f32 = 2.0;
/// Seconds the smelly debuff lasts.
pub const SMELLY_DURATION: f32 = 8.0;
/// How much more allied units avoid smelly units.
pub const SMELLY_SEPARATION_MULTIPLIER: f32 = 5.0;
/// Larger avoidance radius for smelly units.
pub const SMELLY_SEPARATION_DISTANCE: f32 = 80.0;

// ===== Excremage / Poop Visual Color =====

/// Base brown color used for all Excremage visuals, poop effects, and smelly tinting.
pub const EXCREMAGE_BROWN: Color = Color::srgb(0.45, 0.3, 0.1);
/// Darker brown variant for Excremage casting/charging visuals.
pub const EXCREMAGE_BROWN_DARK: Color = Color::srgb(0.35, 0.2, 0.08);

// ===== Persistent Effect Visual Tinting (Poison/Smelly) =====

/// Color overlay for poison effect (sickly green).
pub const POISON_EFFECT_COLOR: Color = Color::srgb(0.2, 0.8, 0.1);
/// Blend strength for poison tint (constant, no pulsing).
pub const POISON_EFFECT_INTENSITY: f32 = 0.35;
/// Color overlay for smelly/poop effect (muddy brown-green).
pub const SMELLY_EFFECT_COLOR: Color = EXCREMAGE_BROWN;
/// Blend strength for smelly tint (constant, no pulsing).
pub const SMELLY_EFFECT_INTENSITY: f32 = 0.6;
/// Color overlay for sickened effect (bright green).
pub const SICKENED_EFFECT_COLOR: Color = Color::srgb(0.1, 0.9, 0.1);
/// Blend strength for sickened tint.
pub const SICKENED_EFFECT_INTENSITY: f32 = 0.5;

// ===== Berserker Rage Visual Tinting =====

/// Color overlay for berserker rage effect (deep red).
pub const BERSERKER_RAGE_EFFECT_COLOR: Color = Color::srgb(1.0, 0.15, 0.1);
/// Blend strength for berserker rage tint (constant, no pulsing).
pub const BERSERKER_RAGE_EFFECT_INTENSITY: f32 = 0.35;

// ===== Elite Visual Tinting =====

/// Color overlay for elite unit effect (deep red glow).
pub const ELITE_EFFECT_COLOR: Color = Color::srgb(1.0, 0.15, 0.1);
/// Pulse speed for elite glow (Hz-like).
pub const ELITE_EFFECT_PULSE_SPEED: f32 = 1.8;
/// Minimum blend intensity for elite glow.
pub const ELITE_EFFECT_MIN_INTENSITY: f32 = 0.2;
/// Maximum blend intensity for elite glow.
pub const ELITE_EFFECT_MAX_INTENSITY: f32 = 0.45;

// ===== Shield Buff Visual Tinting (applied to shielded units) =====

/// Color overlay for shielder shield effect (warm gold).
pub const SHIELD_EFFECT_COLOR: Color = Color::srgb(1.0, 0.85, 0.3);
/// Pulse speed for shield glow (Hz-like, slower = gentler).
pub const SHIELD_EFFECT_PULSE_SPEED: f32 = 2.5;
/// Minimum blend intensity for shield glow.
pub const SHIELD_EFFECT_MIN_INTENSITY: f32 = 0.7;
/// Maximum blend intensity for shield glow.
pub const SHIELD_EFFECT_MAX_INTENSITY: f32 = 1.0;

// ===== Unit Type Glow Colors (applied to special unit types) =====

/// Shared pulse speed for all unit type glows.
pub const UNIT_TYPE_GLOW_PULSE_SPEED: f32 = 2.5;
/// Shared min blend intensity for unit type glows.
pub const UNIT_TYPE_GLOW_MIN_INTENSITY: f32 = 0.4;
/// Shared max blend intensity for unit type glows.
pub const UNIT_TYPE_GLOW_MAX_INTENSITY: f32 = 0.7;

/// Dispeller glow color (white).
pub const DISPELLER_GLOW_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
/// Healer glow color (green).
pub const HEALER_GLOW_COLOR: Color = Color::srgb(0.2, 1.0, 0.3);
/// Shielder glow color (purple).
pub const SHIELDER_GLOW_COLOR: Color = Color::srgb(0.7, 0.2, 1.0);
/// Commander glow color (orange).
pub const COMMANDER_GLOW_COLOR: Color = Color::srgb(1.0, 0.6, 0.1);
/// Brute glow color (purple).
pub const BRUTE_GLOW_COLOR: Color = Color::srgb(0.7, 0.2, 1.0);

// ===== Default Sprite Dimensions =====

/// Default world-space sprite width for most unit types (infantry, archer, assassin, dispeller, battlemage).
pub const DEFAULT_SPRITE_WIDTH: f32 = 24.0 * crate::game::constants::UNIT_SCALE;
/// Default world-space sprite height for most unit types.
pub const DEFAULT_SPRITE_HEIGHT: f32 = 32.0 * crate::game::constants::UNIT_SCALE;
