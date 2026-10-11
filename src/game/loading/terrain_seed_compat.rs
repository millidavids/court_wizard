//! Pins seeded terrain generation. Players share roguelite seeds expecting the
//! same map, so an engine or `rand` upgrade that moves terrain must be a
//! conscious decision, not a surprise.

use crate::config::GameConfig;
use crate::game::loading::terrain_generation::generate_terrain;

/// Folds every placed object's exact bits into one value.
fn fingerprint(config: &GameConfig) -> u64 {
    let mut acc: u64 = 0xCBF2_9CE4_8422_2325;
    let mut mix = |bits: u32| {
        acc ^= u64::from(bits);
        acc = acc.wrapping_mul(0x0000_0100_0000_01B3);
    };
    for t in &config.saved_trees {
        mix(t.x.to_bits());
        mix(t.z.to_bits());
        mix(t.scale.to_bits());
        mix(u32::from(t.sprite_index));
    }
    for p in &config.saved_ponds {
        mix(p.x.to_bits());
        mix(p.z.to_bits());
        mix(p.radius.to_bits());
    }
    for b in &config.saved_bushes {
        mix(b.x.to_bits());
        mix(b.z.to_bits());
        mix(b.scale.to_bits());
        mix(u32::from(b.sprite_index));
    }
    for b in &config.saved_boulders {
        mix(b.x.to_bits());
        mix(b.z.to_bits());
        mix(b.scale.to_bits());
        mix(u32::from(b.sprite_index));
    }
    acc
}

fn generate(seed: u64, level: u32, mirror: bool) -> GameConfig {
    let mut config = GameConfig {
        seed: Some(seed),
        ..Default::default()
    };
    generate_terrain(&mut config, level, 1.0, mirror);
    config
}

// Captured under Bevy 0.19.1 / rand 0.9. Counts are [trees, ponds, bushes, boulders].
#[test]
fn seeded_terrain_is_stable() {
    let cases: [(u64, u32, bool, [usize; 4], u64); 3] = [
        (42, 1, false, [45, 1, 24, 11], 0x272e_83f3_e54e_cd6a),
        (1_234_567, 7, false, [46, 1, 25, 11], 0xc597_3227_08ca_c801),
        (99, 3, true, [87, 1, 44, 11], 0x0bc1_5754_92cc_4c7b),
    ];
    for (seed, level, mirror, counts, print) in cases {
        let c = generate(seed, level, mirror);
        let actual = [
            c.saved_trees.len(),
            c.saved_ponds.len(),
            c.saved_bushes.len(),
            c.saved_boulders.len(),
        ];
        assert_eq!(actual, counts, "seed {seed} level {level}");
        assert_eq!(fingerprint(&c), print, "seed {seed} level {level}");
    }
}
