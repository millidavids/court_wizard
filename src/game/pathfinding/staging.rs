//! Wave staging plan — determines which staging points each wave uses
//! and how attackers are distributed across them.

use std::collections::HashMap;

use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

use crate::game::constants::{
    MAX_WAVE_STAGING_POINTS, MIN_WAVE_STAGING_POINTS, STAGING_POINT_COUNT,
};
use crate::game::seeded_rng::resources::{derive_seed, SEED_PURPOSE_STAGING};

/// Tracks which staging points are active for each wave and provides
/// round-robin assignment of attackers to staging points.
#[derive(Resource, Default)]
pub struct WaveStagingPlan {
    /// Active staging point indices for each wave number.
    pub wave_points: HashMap<u32, Vec<u8>>,
    /// Round-robin counter for distributing attackers within each wave.
    wave_counters: HashMap<u32, usize>,
}

impl WaveStagingPlan {
    /// Removes the staging plan for a completed wave, freeing memory.
    pub fn remove_wave(&mut self, wave: u32) {
        self.wave_points.remove(&wave);
        self.wave_counters.remove(&wave);
    }

    /// Returns the next staging point index for a given wave, cycling through
    /// the active points in round-robin order.
    pub fn next_staging_point(&mut self, wave: u32) -> u8 {
        let points = self.wave_points.get(&wave).expect(
            "WaveStagingPlan: no staging points for wave — compute_wave_staging must be called first",
        );
        let counter = self.wave_counters.entry(wave).or_insert(0);
        let idx = *counter % points.len();
        *counter += 1;
        points[idx]
    }
}

/// Computes which staging points a wave should use, based on the game seed.
/// Inserts the result into the `WaveStagingPlan` for the given wave number.
pub fn compute_wave_staging(plan: &mut WaveStagingPlan, seed: u64, level: u32, wave: u32) {
    let wave_seed = derive_seed(seed, level, SEED_PURPOSE_STAGING).wrapping_add(wave as u64 * 100);
    let mut rng = StdRng::seed_from_u64(wave_seed);

    // Pick how many staging points to use (between MIN and MAX inclusive)
    let count = rng.gen_range(MIN_WAVE_STAGING_POINTS..=MAX_WAVE_STAGING_POINTS);

    // Pick which points from the available 7 (random selection without replacement)
    let mut candidates: Vec<u8> = (0..STAGING_POINT_COUNT as u8).collect();
    candidates.shuffle(&mut rng);
    let chosen: Vec<u8> = candidates.into_iter().take(count).collect();

    plan.wave_points.insert(wave, chosen);
    // Reset the counter for this wave
    plan.wave_counters.insert(wave, 0);
}
