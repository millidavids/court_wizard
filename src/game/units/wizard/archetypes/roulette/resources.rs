use bevy::prelude::*;

use crate::game::units::wizard::components::Spell;

/// The current phase of the roulette wheel.
#[derive(Debug, Clone, PartialEq)]
pub enum RoulettePhase {
    /// Wheel is idle, waiting for player to trigger a spin (spacebar).
    Idle,
    /// Wheel is spinning, visually cycling through spells.
    Spinning {
        /// Total time elapsed since spin started.
        elapsed: f32,
        /// Index of the spell currently highlighted (visual only during spin).
        current_highlight: usize,
        /// Time since the highlight last moved to the next slot.
        tick_timer: f32,
        /// The pre-determined result index (chosen at spin start via RNG).
        result_index: usize,
    },
    /// Wheel has landed on a spell. Player can now cast it.
    Selected {
        /// The spell that was selected.
        spell: Spell,
    },
}

/// Resource tracking the roulette wheel state for the Randomancer.
#[derive(Resource)]
pub struct RouletteState {
    pub phase: RoulettePhase,
    /// Ordered list of spells on the wheel.
    pub wheel_spells: Vec<Spell>,
}

impl Default for RouletteState {
    fn default() -> Self {
        Self {
            phase: RoulettePhase::Idle,
            wheel_spells: Spell::all().to_vec(),
        }
    }
}
