use bevy::prelude::*;

/// Marker for the currently active Mark of Death target, so we can remove old marks.
#[derive(Component)]
pub struct ActiveMarkOfDeath;

/// Talent flags attached alongside the mark to track which talent effects apply.
#[derive(Component, Clone)]
pub struct MarkTalentFlags {
    /// Stored amplification value (for spreading blight to propagate after death removes the modifier).
    pub amplification: f32,
    /// T1-2: Swift Hex — refund mana on death
    pub swift_hex_refund: f32,
    /// T2-0: Spreading Blight — mark jumps on death
    pub spreading_blight: bool,
    /// T2-1: Executioner's Brand — burst damage at low HP
    pub executioner_brand: bool,
    /// T2-2: Focal Point — defenders prioritize this target
    pub focal_point: bool,
    /// T3-1: Death's Ledger — explode on death proportional to max HP
    pub deaths_ledger: bool,
    /// T3-2: Doom — amp increases over time, mark can't be removed
    pub doom: bool,
}

impl Default for MarkTalentFlags {
    fn default() -> Self {
        Self {
            amplification: 0.0,
            swift_hex_refund: 0.0,
            spreading_blight: false,
            executioner_brand: false,
            focal_point: false,
            deaths_ledger: false,
            doom: false,
        }
    }
}

/// Marker to prevent Executioner's Brand from triggering more than once per target.
#[derive(Component)]
pub struct ExecutionerTriggered;

/// Visual indicator that floats above a marked target.
#[derive(Component)]
pub struct MarkVisualIndicator {
    /// The entity this indicator is tracking.
    pub target: Entity,
}

/// Death's Ledger explosion burst — expands and deals AoE damage on death.
#[derive(Component)]
pub struct DeathsLedgerBurst {
    pub time_alive: f32,
    pub lifetime: f32,
    pub max_radius: f32,
    pub damage: f32,
    pub damage_applied: bool,
}
