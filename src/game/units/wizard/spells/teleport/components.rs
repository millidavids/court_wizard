//! Components for the Teleport spell.

use bevy::prelude::*;

use crate::game::units::components::TimedModifier;

/// Marker component indicating the wizard is actively managing Teleport spell state.
///
/// Tracks the destination circle entity and whether we're in phase 1 or 2.
#[derive(Component)]
pub struct TeleportCaster {
    /// Entity ID of the destination circle (None if no destination placed).
    pub destination_circle: Option<Entity>,
    /// Position of the destination circle.
    pub destination_position: Option<Vec3>,
    /// Entity ID of the source circle during second cast (None otherwise).
    pub source_circle: Option<Entity>,
    /// Whether a lingering gate is currently active (Lingering Gate talent).
    pub lingering_gate_active: bool,
}

impl TeleportCaster {
    /// Creates a new TeleportCaster with no circles placed.
    pub const fn new() -> Self {
        Self {
            destination_circle: None,
            destination_position: None,
            source_circle: None,
            lingering_gate_active: false,
        }
    }

    /// Returns true if a destination has been placed.
    pub const fn has_destination(&self) -> bool {
        self.destination_position.is_some()
    }
}

/// Visual indicator for the destination circle (persists between casts).
#[derive(Component)]
pub struct TeleportDestinationCircle {
    /// Time this indicator has been active (for animations).
    pub time_alive: f32,
    /// Base radius for pulse animation (already includes empowerment).
    pub base_radius: f32,
}

impl TeleportDestinationCircle {
    /// Creates a new destination circle indicator.
    pub const fn new(base_radius: f32) -> Self {
        Self {
            time_alive: 0.0,
            base_radius,
        }
    }

    /// Returns the current scale factor for pulse animation.
    ///
    /// Pulsates between 0.95 and 1.05.
    pub fn pulse_scale(&self) -> f32 {
        let pulse_freq = 2.0; // Hz
        let pulse_amplitude = 0.05;
        1.0 + (self.time_alive * pulse_freq * std::f32::consts::TAU).sin() * pulse_amplitude
    }
}

/// Visual indicator for the source circle (only during second cast).
#[derive(Component)]
pub struct TeleportSourceCircle {
    /// Position of the circle center.
    pub position: Vec3,
    /// Time this indicator has been active (for animations).
    pub time_alive: f32,
    /// Whether this teleport is empowered.
    pub empowerment: f32,
}

impl TeleportSourceCircle {
    /// Creates a new source circle indicator.
    pub const fn new(position: Vec3, empowerment: f32) -> Self {
        Self {
            position,
            time_alive: 0.0,
            empowerment,
        }
    }

    /// Returns the current scale factor for pulse animation.
    pub fn pulse_scale(&self) -> f32 {
        let pulse_freq = 2.0;
        let pulse_amplitude = 0.05;
        1.0 + (self.time_alive * pulse_freq * std::f32::consts::TAU).sin() * pulse_amplitude
    }
}

// ===== Talent Components =====

/// Talent parameters computed at cast time from active talent selections.
pub(crate) struct TeleportTalentParams {
    // Tier 1: numeric modifiers
    /// Wide Aperture: multiplier on source circle radius.
    pub radius_mult: f32,
    /// Hasty Translocation: multiplier on second cast time.
    pub cast_time_mult: f32,
    /// Lingering Gate: destination persists for a second teleport.
    pub lingering_gate: bool,
    // Tier 2: behavioral flags
    /// Disorienting Arrival: stun enemies, haste allies on arrival.
    pub disorienting_arrival: bool,
    /// Swap: swap units between source and destination.
    pub swap_mode: bool,
    /// Emergency Recall: instant-cast to teleport allies to castle.
    pub emergency_recall: bool,
    // Tier 3: transformative flags
    /// Dimensional Rift: persistent two-way portal.
    pub dimensional_rift: bool,
    /// Up: teleport units straight up into the sky (single-phase cast).
    pub teleport_up: bool,
    /// Scatterport: scatter enemies to random locations.
    pub scatterport: bool,
}

impl Default for TeleportTalentParams {
    fn default() -> Self {
        Self {
            radius_mult: 1.0,
            cast_time_mult: 1.0,
            lingering_gate: false,
            disorienting_arrival: false,
            swap_mode: false,
            emergency_recall: false,
            dimensional_rift: false,
            teleport_up: false,
            scatterport: false,
        }
    }
}

/// Tier 2: Disorienting Arrival — attack speed buff applied to units after teleportation.
#[derive(Component)]
pub(crate) struct DisorientingHaste {
    pub attack_speed: f32,
    pub time_remaining: f32,
}

impl DisorientingHaste {
    pub const fn new(attack_speed: f32, duration: f32) -> Self {
        Self {
            attack_speed,
            time_remaining: duration,
        }
    }
}

impl TimedModifier for DisorientingHaste {
    fn tick(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Tier 2: Lingering Gate — destination marker that persists after the first teleport,
/// allowing a second teleport to the same spot.
#[derive(Component)]
pub(crate) struct LingeringGateMarker {
    /// Time remaining before the lingering gate expires.
    pub time_remaining: f32,
}

impl LingeringGateMarker {
    pub const fn new(duration: f32) -> Self {
        Self {
            time_remaining: duration,
        }
    }
}

/// Tier 3: Dimensional Rift — persistent portal between source and destination.
/// One-way (source→dest) by default; becomes two-way when Swap talent is also slotted.
#[derive(Component)]
pub(crate) struct DimensionalRift {
    /// Source end of the portal.
    pub source_pos: Vec3,
    /// Destination end of the portal.
    pub dest_pos: Vec3,
    /// Walk-through detection radius.
    pub walk_radius: f32,
    /// Time remaining before the rift closes.
    pub time_remaining: f32,
    /// Whether the rift is two-way (requires Swap talent).
    pub two_way: bool,
}

/// Tier 3: Dimensional Rift — cooldown on a unit after being teleported by a rift.
/// Prevents ping-ponging back and forth.
#[derive(Component)]
pub(crate) struct RiftCooldown {
    pub time_remaining: f32,
}

impl TimedModifier for RiftCooldown {
    fn tick(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}
