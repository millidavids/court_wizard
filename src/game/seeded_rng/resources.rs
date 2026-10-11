use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

/// Exclusive upper bound for generated seeds. Ten digits fit the roguelite seed
/// box, and staying under `i64::MAX` keeps saves readable by toml 0.8 builds.
pub(crate) const MAX_SEED: u64 = 10_000_000_000;

/// A fresh seed in `0..MAX_SEED`.
pub(crate) fn random_seed() -> u64 {
    rand::rng().random_range(0..MAX_SEED)
}

/// The master seed for the current run. Persisted in GameConfig (in-memory).
/// A random seed is generated at run start; players can override with a custom seed.
#[derive(Resource, Debug, Clone)]
pub struct GameSeed(pub u64);

/// Shared seeded RNG for the current run. All gameplay randomness draws from this
/// so that identical seeds produce identical runs.
#[derive(Resource)]
pub struct GameRng(pub StdRng);

impl GameRng {
    pub fn new(seed: &GameSeed) -> Self {
        Self(StdRng::seed_from_u64(seed.0))
    }
}

/// Starts a run's randomness from `seed`.
pub(crate) fn insert_game_seed(commands: &mut Commands, seed: u64) {
    let game_seed = GameSeed(seed);
    commands.insert_resource(GameRng::new(&game_seed));
    commands.insert_resource(game_seed);
}

/// Purpose constants for deriving per-system sub-seeds.
pub const SEED_PURPOSE_TERRAIN: u64 = 7919;
pub const SEED_PURPOSE_FLORA: u64 = 7937;
pub const SEED_PURPOSE_STAGING: u64 = 7951;

/// Derives a deterministic sub-seed from a master seed, level, and purpose constant.
/// Ensures different systems get independent RNG streams without order-dependency.
///
/// # Warning
/// `purpose` must never be 0. Passing `purpose = 0` collapses the entire output to 0
/// regardless of `master` or `level`, producing identical seeds across all systems.
/// All `SEED_PURPOSE_*` constants are non-zero primes; always use one of them.
pub fn derive_seed(master: u64, level: u32, purpose: u64) -> u64 {
    master
        .wrapping_mul(6364136223846793005)
        .wrapping_add(level as u64)
        .wrapping_mul(purpose)
}

#[cfg(test)]
mod tests {
    use rand::RngExt;
    use rand::seq::{IndexedRandom, SliceRandom};

    use super::*;

    // Players share seeds and peers sync them, so a rand upgrade that changes
    // these values silently changes every run. Captured under rand 0.9.
    #[test]
    fn seeded_output_is_stable_across_rand_upgrades() {
        let mut rng = GameRng::new(&GameSeed(0xC0FFEE)).0;
        let ints: Vec<i32> = (0..4).map(|_| rng.random_range(-50..50)).collect();
        let floats: Vec<f32> = (0..3).map(|_| rng.random_range(0.0..10.0)).collect();
        let unit: f32 = rng.random();
        let word: u32 = rng.random();
        let wide: u64 = rng.random();
        let mut deck: Vec<u8> = (0..8).collect();
        deck.shuffle(&mut rng);
        let pick = *[10, 20, 30, 40, 50].choose(&mut rng).expect("non-empty");

        assert_eq!(ints, [24, 10, 1, 47]);
        assert_eq!(floats, [7.4439917, 0.94806075, 5.5029583]);
        assert_eq!(unit, 0.4992742);
        assert_eq!(word, 2916177737);
        assert_eq!(wide, 7949506307846458434);
        assert_eq!(deck, [6, 4, 2, 3, 0, 7, 5, 1]);
        assert_eq!(pick, 40);
    }
}
