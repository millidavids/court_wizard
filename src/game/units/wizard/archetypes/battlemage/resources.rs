use bevy::prelude::*;

/// Tracks the battlemage's field state.
#[derive(Resource, Debug, Clone, Default)]
pub struct BattlemageState {
    /// Animation phase for entering/exiting the field.
    pub phase: BattlemagePhase,
    /// Whether the battlemage has retreated (died on the field). Cannot re-enter.
    pub retreated: bool,
}

/// Current phase of the battlemage enter/exit sequence.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum BattlemagePhase {
    /// Idle on the castle wall, not in combat.
    #[default]
    Idle,
    /// Player clicked "Enter the Fray" and is choosing where to spawn.
    ChoosingLocation,
    /// Battlemage is actively on the field.
    OnField,
}
