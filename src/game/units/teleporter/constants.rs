use bevy::prelude::*;

use crate::game::constants::UNIT_SCALE;

pub(super) const TELEPORTER_HEALTH: f32 = 60.0;
pub(super) const TELEPORTER_MOVEMENT_SPEED: f32 =
    crate::game::constants::UNIT_MOVEMENT_SPEED * 0.98;

pub(super) const TELEPORTER_RADIUS: f32 = 8.0 * UNIT_SCALE;
pub(super) const TELEPORTER_HITBOX_HEIGHT: f32 = 20.0 * UNIT_SCALE;
pub(super) const TELEPORTER_SCALE: f32 = 1.1;

pub(super) const TELEPORTER_SPRITE_TINT: Color = Color::srgb(0.4, 0.8, 1.4);
pub(super) const TELEPORTER_GLOW_COLOR: Color = Color::srgb(0.35, 0.55, 1.0);

/// Range from the King at which the teleporter begins its channel.
pub(super) const CHANNEL_RANGE: f32 = 1000.0;

/// Channel duration in seconds.
pub(super) const CHANNEL_DURATION: f32 = 10.0;

/// Channeling VFX — inward-implosion particles spawning on a growing sphere.
pub(super) const CHANNEL_PARTICLE_SPAWN_INTERVAL: f32 = 0.035;
pub(super) const CHANNEL_PARTICLE_COUNT_PER_SPAWN: usize = 4;
pub(super) const CHANNEL_PARTICLE_START_RADIUS: f32 = 20.0;
pub(super) const CHANNEL_PARTICLE_MAX_RADIUS: f32 = 200.0;
pub(super) const CHANNEL_PARTICLE_SIZE: f32 = 6.0;
pub(super) const CHANNEL_PARTICLE_LIFETIME: f32 = 0.9;

/// Number of allies to grab and teleport onto the king.
pub(super) const TELEPORT_GRAB_COUNT: usize = 20;

/// Fraction of max health granted as temporary HP to teleported allies (500% bonus).
pub(super) const TELEPORT_TEMP_HP_RATIO: f32 = 5.0;

/// Duration of the temporary HP buff granted to teleported allies.
pub(super) const TELEPORT_TEMP_HP_DURATION: f32 = 20.0;

/// Scatter radius around the king when dropping teleported allies.
pub(super) const DROP_JITTER_RADIUS: f32 = 60.0;

/// Visual ripple radius shared by the channel indicator and the completion warp.
pub(super) const TELEPORT_VFX_RADIUS: f32 = 240.0;

/// Cooldown between teleport channels.
pub(super) const CHANNEL_COOLDOWN: f32 = 15.0;

/// Tier at which teleporters begin spawning.
pub const TELEPORTER_START_TIER: u32 = 2;

pub(super) const TELEPORTER_BOLT_DAMAGE: f32 = 5.0;
pub(super) const TELEPORTER_BOLT_SPEED: f32 = 400.0;
pub(super) const TELEPORTER_BOLT_RADIUS: f32 = 4.0;
pub(super) const TELEPORTER_BOLT_LIFETIME: f32 = 3.0;
pub(super) const TELEPORTER_BOLT_COLOR: Color = Color::srgb(0.5, 0.7, 1.6);
pub(super) const TELEPORTER_ATTACK_RANGE: f32 = 500.0;
pub(super) const TELEPORTER_ATTACK_COOLDOWN: f32 = 2.0;
