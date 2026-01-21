//! Shared unit constants.
//!
//! Contains constants used across multiple unit types.

/// Primary frequency for melee random force sine wave.
pub const MELEE_RANDOM_FREQ_PRIMARY: f32 = 3.7;

/// Secondary frequency for melee random force cosine wave.
pub const MELEE_RANDOM_FREQ_SECONDARY: f32 = 2.3;

/// Amplitude multiplier for primary random force wave.
pub const MELEE_RANDOM_AMPLITUDE_PRIMARY: f32 = 2.0;

/// Frequency multiplier for secondary random force seed.
pub const MELEE_RANDOM_SEED_FREQ_MULTIPLIER: f32 = 1.7;

/// X-axis multiplier for position-based random seed.
pub const MELEE_RANDOM_SEED_X_MULTIPLIER: f32 = 0.1;

/// Z-axis multiplier for position-based random seed.
pub const MELEE_RANDOM_SEED_Z_MULTIPLIER: f32 = 0.13;
