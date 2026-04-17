use bevy::prelude::*;

use crate::game::units::DamageType;

/// Broadcast by every damage-dealing spell system when it applies damage to a region
/// of the world. Terrain systems (pond, boulder) listen and apply reactive effects
/// (evaporation fog, frozen, shocked, boulder heat → explosion).
///
/// For area spells, `radius` is the current AoE radius. For point sources (lightning
/// bolt endpoints, beam tips), use `radius: 0.0` — the overlap check becomes a pure
/// distance-to-center test against terrain radii.
#[derive(Message, Debug, Clone, Copy)]
pub struct TerrainDamageMessage {
    /// World-space center of the damaging region. Only XZ matters.
    pub position: Vec3,
    /// Radius of the effect (0.0 for point sources).
    pub radius: f32,
    /// Damage applied at this tick at this position.
    pub damage: f32,
    /// Damage type determines which reactive effects trigger.
    pub damage_type: DamageType,
}
