//! Compact binary snapshot format for multiplayer state synchronization.
//!
//! The host serializes the game state into a `GameSnapshot` each frame and
//! sends it over the unreliable WebRTC data channel. The guest deserializes
//! and renders ghost entities from the snapshot data.

mod cast_event;
mod crdt;
mod game;
mod protocol;
mod spell_effect;
mod unit;

pub use cast_event::{
    AuraBubbleVariant, CastEventKind, CastEventSnapshot, MoteMaterial, PoofVariant, SparkMaterial,
    SpellSchoolWire, SpellSoundId,
};
pub use crdt::{CrdtSnapshot, CrdtUnitUpdate};
pub use game::{ArrowSnapshot, BeamSnapshot, GameSnapshot, MagicMissileSnapshot};
pub use protocol::{
    SpellVisualSnapshot, UNRELIABLE_CRDT_SNAPSHOT, UNRELIABLE_GAME_SNAPSHOT,
    UNRELIABLE_SPELL_SNAPSHOT,
};
pub use spell_effect::{
    SpellArcSnapshot, SpellEffectKind, SpellEffectSnapshot, SpellProjectileSnapshot,
    SpellSnapshotData,
};
pub use unit::{SnapshotTick, UnitFlags, build_unit_snapshot, u8_to_team};
