use bevy::prelude::*;

/// The master seed for the current run. Persisted in GameConfig (in-memory).
/// A random seed is generated at run start; players can override with a custom seed.
#[derive(Resource, Debug, Clone)]
pub struct GameSeed(pub u64);

/// Purpose constants for deriving per-system sub-seeds.
pub const SEED_PURPOSE_TERRAIN: u64 = 7919;
pub const SEED_PURPOSE_FLORA: u64 = 7937;
pub const SEED_PURPOSE_STAGING: u64 = 7951;

/// Derives a deterministic sub-seed from a master seed, level, and purpose constant.
/// Ensures different systems get independent RNG streams without order-dependency.
pub fn derive_seed(master: u64, level: u32, purpose: u64) -> u64 {
    master
        .wrapping_mul(6364136223846793005)
        .wrapping_add(level as u64)
        .wrapping_mul(purpose)
}
