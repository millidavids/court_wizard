/// How long the wheel visually spins before landing (seconds).
pub const SPIN_DURATION: f32 = 2.0;

/// How fast the visual highlight cycles through spells at start of spin (seconds per slot).
pub const INITIAL_TICK_INTERVAL: f32 = 0.05;

/// Empowerment multiplier for roulette-selected spells.
/// Higher than rune's 1.25x to compensate for loss of spell choice.
pub const ROULETTE_EMPOWERMENT: f32 = 1.75;
