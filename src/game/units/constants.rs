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

// ===== Frost Slow =====

/// Additional slow percentage per frost damage hit.
pub const FROST_SLOW_PER_STACK: f32 = -0.05;
/// Seconds before frost slow expires (resets on each frost hit).
pub const FROST_SLOW_DURATION: f32 = 3.0;

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

/// Color overlay for fire DoT effect (orange-red).
pub const FIRE_EFFECT_COLOR: Color = Color::srgb(1.0, 0.35, 0.05);
/// Color overlay for frost slow effect (icy blue).
pub const FROST_EFFECT_COLOR: Color = Color::srgb(0.3, 0.55, 1.0);
/// Color overlay for electric charge effect (yellow-white).
pub const ELECTRIC_EFFECT_COLOR: Color = Color::srgb(1.0, 0.95, 0.5);
/// Speed of fire effect pulsing (radians per second).
pub const FIRE_EFFECT_PULSE_SPEED: f32 = 4.0;
/// Speed of electric effect flickering (radians per second).
pub const ELECTRIC_EFFECT_FLICKER_SPEED: f32 = 12.0;
/// Maximum blend strength for fire tint.
pub const FIRE_EFFECT_MAX_INTENSITY: f32 = 0.45;
/// Minimum blend strength for fire tint.
pub const FIRE_EFFECT_MIN_INTENSITY: f32 = 0.15;
/// Blend strength for frost tint (constant, no pulsing).
pub const FROST_EFFECT_INTENSITY: f32 = 0.35;
/// Maximum blend strength for electric tint.
pub const ELECTRIC_EFFECT_MAX_INTENSITY: f32 = 0.5;
/// Minimum blend strength for electric tint.
pub const ELECTRIC_EFFECT_MIN_INTENSITY: f32 = 0.05;
/// Color overlay for mind control effect (bright pink).
pub const MIND_CONTROL_EFFECT_COLOR: Color = Color::srgb(1.0, 0.2, 0.7);
/// Blend strength for mind control tint (constant, no pulsing).
pub const MIND_CONTROL_EFFECT_INTENSITY: f32 = 0.45;
