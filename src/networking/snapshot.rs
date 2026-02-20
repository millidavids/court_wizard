//! Compact binary snapshot format for multiplayer state synchronization.
//!
//! The host serializes the game state into a `GameSnapshot` each frame and
//! sends it over the unreliable WebRTC data channel. The guest deserializes
//! and renders ghost entities from the snapshot data.

use serde::{Deserialize, Serialize};

use bevy::prelude::*;

use crate::game::units::components::{Health, Team};

use super::entity_map::NetworkEntityId;

/// Complete game state snapshot sent from host to guest each frame.
#[derive(Serialize, Deserialize)]
pub struct GameSnapshot {
    /// Monotonically increasing tick counter for ordering.
    pub tick: u32,
    /// State of every tracked unit.
    pub units: Vec<UnitSnapshot>,
    /// State of every in-flight arrow projectile.
    pub arrows: Vec<ArrowSnapshot>,
}

/// Compact per-unit state (~21 bytes).
#[derive(Serialize, Deserialize)]
pub struct UnitSnapshot {
    /// Network entity ID assigned by the host.
    pub id: u32,
    /// World position X.
    pub x: f32,
    /// World position Y (height above battlefield).
    pub y: f32,
    /// World position Z.
    pub z: f32,
    /// Team encoded as u8: 0=Defenders, 1=Attackers, 2=Undead.
    pub team: u8,
    /// Health as a 0-100 percentage.
    pub health_pct: u8,
    /// Bitfield flags (see `UnitFlags`).
    pub flags: u8,
}

/// Bitfield constants for `UnitSnapshot::flags`.
pub struct UnitFlags;

impl UnitFlags {
    pub const CORPSE: u8 = 1 << 0;
    pub const KING: u8 = 1 << 1;
    pub const ARCHER: u8 = 1 << 2;
    pub const KINGS_GUARD: u8 = 1 << 3;
}

/// Encodes a `Team` component into a u8.
pub fn team_to_u8(team: &Team) -> u8 {
    match team {
        Team::Defenders => 0,
        Team::Attackers => 1,
        Team::Undead => 2,
    }
}

/// Decodes a u8 back into a `Team`.
pub fn u8_to_team(val: u8) -> Team {
    match val {
        0 => Team::Defenders,
        1 => Team::Attackers,
        _ => Team::Undead,
    }
}

/// Builds a `UnitSnapshot` from an entity's components.
pub fn build_unit_snapshot(
    net_id: &NetworkEntityId,
    transform: &Transform,
    team: &Team,
    health: &Health,
    is_corpse: bool,
    is_king: bool,
    is_archer: bool,
    is_kings_guard: bool,
) -> UnitSnapshot {
    let health_pct = if health.max > 0.0 {
        ((health.current / health.max) * 100.0).clamp(0.0, 100.0) as u8
    } else {
        0
    };

    let mut flags = 0u8;
    if is_corpse {
        flags |= UnitFlags::CORPSE;
    }
    if is_king {
        flags |= UnitFlags::KING;
    }
    if is_archer {
        flags |= UnitFlags::ARCHER;
    }
    if is_kings_guard {
        flags |= UnitFlags::KINGS_GUARD;
    }

    UnitSnapshot {
        id: net_id.0,
        x: transform.translation.x,
        y: transform.translation.y,
        z: transform.translation.z,
        team: team_to_u8(team),
        health_pct,
        flags,
    }
}

/// Compact per-arrow state (~12 bytes).
#[derive(Serialize, Deserialize)]
pub struct ArrowSnapshot {
    /// World position X.
    pub x: f32,
    /// World position Y (height).
    pub y: f32,
    /// World position Z.
    pub z: f32,
}

/// Monotonically increasing tick counter for snapshot ordering.
#[derive(Resource, Default)]
pub struct SnapshotTick(pub u32);
