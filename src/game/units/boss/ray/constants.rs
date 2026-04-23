use bevy::prelude::*;

use crate::game::constants::{ATTACKER_BASE, TINT_GREEN, UNIT_SCALE, tint};

// ===== Visual Appearance =====

pub const RAY_COLOR: Color = tint(ATTACKER_BASE, TINT_GREEN, 0.4);

pub const RAY_BODY_RADIUS: f32 = 35.0 * UNIT_SCALE;
pub const RAY_BODY_HITBOX_HEIGHT: f32 = 50.0 * UNIT_SCALE;
pub const RAY_BODY_HOVER_HEIGHT: f32 = 10.0 * UNIT_SCALE;

pub const RAY_EYE_RADIUS: f32 = 12.0 * UNIT_SCALE;
pub const RAY_EYE_HITBOX_HEIGHT: f32 = 20.0 * UNIT_SCALE;
pub const RAY_EYE_FLOAT_HEIGHT: f32 = 80.0 * UNIT_SCALE;
pub const RAY_EYE_WANDER_RADIUS: f32 = 120.0 * UNIT_SCALE;
pub const RAY_EYE_WANDER_SPEED: f32 = 30.0 * UNIT_SCALE;
pub const RAY_EYE_TURN_RATE: f32 = 0.8;
pub const RAY_EYE_SEPARATION_RADIUS: f32 = RAY_EYE_RADIUS * 2.0;
pub const RAY_EYE_SEPARATION_FORCE: f32 = 250.0 * UNIT_SCALE;
pub const RAY_EYE_DRIFT_RATE: f32 = 3.0;
pub const RAY_STALK_PARTICLE_RADIUS: f32 = 1.0 * UNIT_SCALE;
pub const RAY_STALK_PARTICLE_SPEED: f32 = 132.0;
pub const RAY_STALK_PARTICLE_HOMING: f32 = 4.0;
pub const RAY_STALK_PARTICLE_WOBBLE_AMP: f32 = 30.0;
pub const RAY_STALK_PARTICLE_WOBBLE_FREQ: f32 = 8.0;
pub const RAY_STALK_PARTICLE_SHUDDER: f32 = 60.0;
pub const RAY_STALK_PARTICLE_SPAWN_INTERVAL: f32 = 0.12;

// ===== Health & Combat =====

pub const RAY_HEALTH: f32 = 10000.0;
pub const RAY_DAMAGE_MULTIPLIER: f32 = -0.3;
pub const RAY_MELEE_DAMAGE_REDUCTION: f32 = 0.05;
pub const RAY_EYE_HEALTH: f32 = 2000.0;
pub const RAY_EYE_IMPLODE_DURATION: f32 = 0.6;

// ===== Movement =====

pub const RAY_APPROACH_SPEED: f32 = 200.0;
pub const RAY_HOVER_SPEED: f32 = 80.0;
pub const RAY_APPROACH_TARGET_X: f32 = 0.0;

// ===== Beam Dimensions =====

pub const BEAM_LENGTH: f32 = 800.0;
pub const BEAM_WIDTH: f32 = 50.0;

// ===== Effect Parameters =====

pub const PETRIFY_DURATION: f32 = f32::MAX;
pub const PETRIFY_CHANNEL_TIME: f32 = 2.0;
pub const PETRIFY_TURN_RATE: f32 = 3.5;
pub const PETRIFY_BEAM_COOLDOWN: f32 = 4.0;
pub const PETRIFY_KING_DAMAGE_PER_SECOND: f32 = 5.0;
pub const PETRIFY_BEAM_WIDTH: f32 = 240.0;
pub const DISINTEGRATION_DAMAGE_PER_TICK: f32 = 8.0;
pub const DISINTEGRATION_DAMAGE_INTERVAL: f32 = 0.1;
pub const DISINTEGRATION_TURN_RATE: f32 = 2.0;
pub const DISINTEGRATION_RETICLE_SPEED: f32 = 133.0;
pub const DISINTEGRATION_BEAM_LIFETIME: f32 = 10.0;
pub const DISINTEGRATION_BEAM_COOLDOWN: f32 = 3.0;
pub const DISINTEGRATION_BEAM_GROWTH_TIME: f32 = 0.5;
pub const DISINTEGRATION_BEAM_PULSE_SPEED: f32 = 8.0;
pub const DISINTEGRATION_BEAM_PULSE_AMOUNT: f32 = 0.15;
pub const FEAR_DURATION: f32 = 10.0;
pub const FEAR_EYE_ORBIT_RADIUS: f32 = RAY_BODY_RADIUS * 1.5;
pub const FEAR_EYE_ORBIT_SPEED: f32 = 0.6;
pub const FEAR_BEAM_GROUND_RADIUS: f32 = 75.0;
pub const FEAR_BEAM_COOLDOWN: f32 = 0.15;
pub const TELEPORT_EYE_COOLDOWN: f32 = 15.0;
pub const TELEPORT_EYE_UNIT_COUNT: usize = 50;
pub const TELEPORT_BUBBLE_DURATION: f32 = 1.0;
pub const TELEPORT_BUBBLE_EXPAND_SPEED: f32 = 2000.0;
pub const MIND_CONTROL_DURATION: f32 = f32::MAX;
pub const MIND_CONTROL_CHANNEL_TIME: f32 = 1.5;
pub const MIND_CONTROL_BEAM_COOLDOWN: f32 = 6.0;
pub const MIND_CONTROL_BEAM_WIDTH: f32 = 180.0;
pub const MIND_CONTROL_TURN_RATE: f32 = 3.5;

// ===== Targeting =====

pub const MAX_BEAM_RANGE: f32 = 900.0;

// ===== Visible Area =====

pub const VISIBLE_MIN_X: f32 = -2200.0;
pub const VISIBLE_MAX_X: f32 = 2200.0;
pub const VISIBLE_MIN_Z: f32 = -2200.0;
pub const VISIBLE_MAX_Z: f32 = 800.0;

// ===== Beam Colors (emissive) =====

pub const PETRIFY_BEAM_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);
pub const DISINTEGRATE_BEAM_COLOR: Color = Color::srgb(1.0, 0.6, 0.1);
pub const FEAR_BEAM_COLOR: Color = Color::srgb(0.6, 0.0, 0.8);
pub const CHARM_BEAM_COLOR: Color = Color::srgb(1.0, 0.3, 0.6);
pub const TELEPORT_BEAM_COLOR: Color = Color::srgb(0.0, 1.0, 0.7);
