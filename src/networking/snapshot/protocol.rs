//! Protocol constants and the bidirectional spell visual snapshot type.

use serde::{Deserialize, Serialize};

use super::cast_event::CastEventSnapshot;
use super::game::{BeamSnapshot, MagicMissileSnapshot};
use super::spell_effect::{SpellArcSnapshot, SpellEffectSnapshot, SpellProjectileSnapshot};

/// Type prefix bytes for unreliable channel messages.
///
/// Each unreliable message starts with a 1-byte prefix so receivers can
/// distinguish between different payload types.
pub const UNRELIABLE_GAME_SNAPSHOT: u8 = 0;
pub const UNRELIABLE_SPELL_SNAPSHOT: u8 = 1;
pub const UNRELIABLE_CRDT_SNAPSHOT: u8 = 2;

/// Spell visual data sent bidirectionally between host and guest.
///
/// Each client collects their local spell visuals (effects, projectiles, arcs,
/// missiles, beams) and sends them so the other client can render ghosts.
#[derive(Serialize, Deserialize, Default)]
pub struct SpellVisualSnapshot {
    /// Persistent spell effects (zones, walls, black holes, explosions, etc.).
    pub spell_effects: Vec<SpellEffectSnapshot>,
    /// Ephemeral spell projectiles (fireballs, ice, meteors in flight).
    pub spell_projectiles: Vec<SpellProjectileSnapshot>,
    /// Ephemeral spell arcs/beams (chain lightning, finger of death, etc.).
    pub spell_arcs: Vec<SpellArcSnapshot>,
    /// Magic missile positions.
    pub magic_missiles: Vec<MagicMissileSnapshot>,
    /// Beam positions (disintegrate, etc.).
    pub beams: Vec<BeamSnapshot>,
    /// One-shot cast VFX events fired this tick (school flares, aura
    /// bubbles, smoke poofs, motes, sparks, dust). The receiver iterates
    /// these and spawns the matching local VFX via the existing
    /// `vfx::systems::spawn_*` helpers. Drained once per send tick.
    pub cast_events: Vec<CastEventSnapshot>,
}
