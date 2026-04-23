//! Wave staging plan — determines which staging points each wave uses
//! and how attackers are distributed across them.

use std::collections::HashMap;

use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

use crate::game::seeded_rng::resources::{SEED_PURPOSE_STAGING, derive_seed};

/// Which tunnel an attacker spawned from.
/// Determines which subset of staging points the attacker may stage at so
/// the two streams never cross.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpawnTunnel {
    /// Upper/left tunnel (spawn z ≈ -375). Stages at indices 0..=3.
    Left,
    /// Lower/right tunnel (spawn z ≈ -1575). Stages at indices 3..=6.
    Right,
}

/// Staging indices allowed for the left tunnel.
pub const LEFT_TUNNEL_STAGING: &[u8] = &[0, 1, 2, 3];
/// Staging indices allowed for the right tunnel.
pub const RIGHT_TUNNEL_STAGING: &[u8] = &[3, 4, 5, 6];

/// Minimum staging points picked per tunnel per wave.
const MIN_POINTS_PER_TUNNEL: usize = 1;
/// Maximum staging points picked per tunnel per wave.
const MAX_POINTS_PER_TUNNEL: usize = 2;

/// Tracks which staging points are active for each (wave, tunnel) and provides
/// round-robin assignment of attackers to staging points.
#[derive(Resource, Default)]
pub struct WaveStagingPlan {
    /// Active staging point indices per (wave, tunnel).
    pub wave_points: HashMap<(u32, SpawnTunnel), Vec<u8>>,
    /// Round-robin counter per (wave, tunnel).
    wave_counters: HashMap<(u32, SpawnTunnel), usize>,
}

impl WaveStagingPlan {
    /// Removes the staging plan for a completed wave, freeing memory.
    pub fn remove_wave(&mut self, wave: u32) {
        self.wave_points.retain(|(w, _), _| *w != wave);
        self.wave_counters.retain(|(w, _), _| *w != wave);
    }

    /// Returns true if the plan has any staging points for the given wave.
    pub fn has_wave(&self, wave: u32) -> bool {
        self.wave_points.keys().any(|(w, _)| *w == wave)
    }

    /// Returns the next staging point index for a given wave and tunnel,
    /// cycling through that tunnel's active points in round-robin order.
    pub fn next_staging_point(&mut self, wave: u32, tunnel: SpawnTunnel) -> u8 {
        let key = (wave, tunnel);
        let points = self.wave_points.get(&key).expect(
            "WaveStagingPlan: no staging points for wave/tunnel — compute_wave_staging must be called first",
        );
        let counter = self.wave_counters.entry(key).or_insert(0);
        let idx = *counter % points.len();
        *counter += 1;
        points[idx]
    }
}

/// Computes staging points for both tunnels of a wave, based on the game seed.
/// Each tunnel independently picks 1-2 points from its allowed range so the
/// two streams never cross.
pub fn compute_wave_staging(plan: &mut WaveStagingPlan, seed: u64, level: u32, wave: u32) {
    let wave_seed = derive_seed(seed, level, SEED_PURPOSE_STAGING).wrapping_add(wave as u64 * 100);
    let mut rng = StdRng::seed_from_u64(wave_seed);

    for (tunnel, range) in [
        (SpawnTunnel::Left, LEFT_TUNNEL_STAGING),
        (SpawnTunnel::Right, RIGHT_TUNNEL_STAGING),
    ] {
        let count = rng.gen_range(MIN_POINTS_PER_TUNNEL..=MAX_POINTS_PER_TUNNEL);
        let mut candidates: Vec<u8> = range.to_vec();
        candidates.shuffle(&mut rng);
        let chosen: Vec<u8> = candidates.into_iter().take(count).collect();
        plan.wave_points.insert((wave, tunnel), chosen);
        plan.wave_counters.insert((wave, tunnel), 0);
    }
}
