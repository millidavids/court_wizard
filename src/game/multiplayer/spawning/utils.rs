//! Shared spawn helpers used across the MP spawning sub-modules.

use crate::game::units::components::AttackTiming;

/// Returns an `AttackTiming` whose `last_attack_time` is randomised across
/// the cycle. In SP, the staging phase spaces out first contact so units
/// don't all swing on the same frame; MP has no staging, so without this
/// pre-stagger every unit spawned with `last_attack_time = None` would
/// `can_attack` on the very first frame of melee contact — letting 20+
/// units one-shot a defender in a single frame. By seeding the cycle
/// offset, first-contact damage is naturally distributed over ~2s.
pub(super) fn staggered_attack_timing() -> AttackTiming {
    use rand::Rng;
    let mut rng = rand::rng();
    // `f32::EPSILON..` excludes exactly 0.0 — combined with
    // `can_attack`'s strict `attack_time > last_time`, a recorded slot of
    // 0.0 paired with the cycle's `last_time` also being 0.0 (on the very
    // first frame after game start) would silently block that unit for a
    // full cycle. Vanishingly rare with random_range, but easy to exclude.
    let offset = rng.random_range(f32::EPSILON..crate::game::constants::ATTACK_CYCLE_DURATION);
    let mut timing = AttackTiming::new();
    timing.last_attack_time = Some(offset);
    timing
}
