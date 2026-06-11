## game-misc

**Scope:** `src/game/drops/`, `src/game/benchmarking/`, `src/game/seeded_rng/`

---

### Mental model

Three small, focused subsystems that are each self-contained:

- **drops/** — Manages ingredient-drop pickups. Enemy deaths fire a `SpawnIngredientDropMessage`; the plugin spawns bobbing billboard quads, ticks their lifetime for animation, and lets the telekinesis spell convert them to `FlyingToWizard` entities that arc toward the wizard and unlock/persist the ingredient on arrival. A `LockedIngredients` resource caches which ingredients still need to be unlocked so the pool stays weighted toward unowned drops.
- **benchmarking/** — A `#[cfg(feature = "benchmarking")]`-gated module that attaches Bevy's standard diagnostic plugins and an F4-toggled sampled logger that dumps FPS, frame-time, entity count, and CPU/memory once every two seconds.
- **seeded_rng/** — Provides a global `GameRng` (`StdRng` under a seed) and a small `derive_seed` utility for producing independent per-system RNG streams. All gameplay randomness draws from the shared resource; terrain/flora/staging systems derive sub-seeds rather than pulling from `GameRng` directly to stay order-independent.

The modules are clean and appropriately scoped. The most notable issue is a stale doc-string in the benchmarking plugin (says F3, code uses F4) and a misleading comment in the drops component about fade/despawn that was never implemented.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| GM-01 | DocDrift | `benchmarking/plugin.rs:9` | Medium | S | Plugin doc comment says "the F3 toggle" but the system binds `KeyCode::F4` (`systems.rs:14`). Wrong key will confuse anyone using benchmarking builds. | Change the doc comment to say "F4 toggle". |
| GM-02 | DocDrift | `drops/components.rs:10` | Low | S | `IngredientDrop::time_alive` is documented as "for despawn and fade" but no fade or timeout-despawn logic exists anywhere in the drops module; `time_alive` is used only for animation math. | Update the field doc to "for animation (bobbing and pulse)". If a fade/despawn on timeout is desired, add a `DROP_MAX_LIFETIME` constant and despawn branch to `tick_drop_lifetimes`. |
| GM-03 | ArchitecturalDecay | `drops/systems.rs:20–34` | Low | S | `init_locked_ingredients` calls `load_unified_save()` (a synchronous disk read) inside an `OnEnter(AppState::Loading)` system, on the main thread, every run. The drop pool is a pure cache derived from save data that the loading queue already reads elsewhere. | Not critical at current scale, but worth noting. Could be folded into the existing save-load pass during loading to avoid a redundant disk read. |
| GM-04 | Performance | `benchmarking/plugin.rs:26–28` | Low | S | `toggle_diagnostics` runs every `Update` frame unconditionally (no `run_if` guard). It is gated by the `benchmarking` feature flag so it never ships in production, but it still polls keyboard input every frame even when diagnostics are already off. | Add `.run_if(not(diagnostics_enabled))` to `toggle_diagnostics` and a second branch `.run_if(diagnostics_enabled)` for the "turn off" case, or simply add a state-based guard. Alternatively, the early-return on `just_pressed` is cheap enough to be acceptable given the feature-flag gating — see lookBadButFine. |
| GM-05 | ArchitecturalDecay | `seeded_rng/resources.rs:28–33` | Low | S | `derive_seed` uses a hand-rolled LCG-style hash mixing `wrapping_mul` with fixed magic constants. This is not obviously correct (e.g., passing `purpose = 0` collapses the output to 0). | Document the mixing strategy and add a guard/assert that none of the `SEED_PURPOSE_*` constants are zero. Alternatively, use a proper hash function (`FxHasher`, `AHash`, or even `u64::rotate_left` mixing) to avoid degenerate cases. |

---

### Oversized files

No files in scope exceed 300 LOC. Largest is `drops/systems.rs` at 159 lines.

| File | LOC | Exempt | Reason |
|------|-----|--------|--------|
| (none) | — | — | — |

---

### Looks bad but is actually fine

- **`toggle_diagnostics` unconditional Update with keyboard poll** — The function is (a) only compiled under the `benchmarking` feature, (b) returns immediately unless `F4` is `just_pressed`, and (c) has zero heap allocation. The cost is negligible. Flagged as Low anyway because the pattern violates the project's run_if convention, but it is not a real performance risk.
- **`FlyingToWizard.ingredient` duplicating info from `IngredientDrop`** — The comment on `FlyingToWizard` explains why: `IngredientDrop` is *removed* when `FlyingToWizard` is inserted (telekinesis `systems.rs:391`). The ingredient field must be carried over because the source component is gone. Not redundant.
- **`drop_events` read in `spawn_ingredient_drops` without a `run_if(any_messages::<…>())` guard** — Messages accumulate and the inner `for event in …` loop simply does nothing if the reader is empty. The outer `is_gameplay_running` guard is sufficient; adding a messages-exist guard would be premature.
- **`unlock_ingredient` called inside an Update system `fly_drops_to_wizard`** — This is a synchronous disk write per collected drop. Given the 0.5% drop chance and infrequent collection events, this is intentional and acceptable. The project does this pattern elsewhere for achievements.
- **`GameRng` accessed as `crate::game::seeded_rng::resources::GameRng`** in `drops/systems.rs:39` — Using the full path instead of a `use` import looks messy, but it is correct and the module is accessible. No functional issue.

---

### Open questions

1. **Drop timeout:** Were fade-out and timed despawn intentionally deferred or silently forgotten? The component doc references both but neither is implemented. If drops that aren't picked up by telekinesis persist indefinitely (only cleaned up by `OnGameplayScreen` despawn on game exit), that seems intentional but should be documented explicitly.
2. **`derive_seed` edge case with `purpose = 0`:** If a future purpose constant is ever set to 0 (or `level` wraps in a way that produces 0 mid-formula), the entire seed collapses. Is this a known acceptable risk or should a compile-time assert be added?
3. **`GameRng` contention in multiplayer:** Several MP-gated systems reference `GameRng`, but the meteorologist archetype comment (`visuals.rs:157`) notes it must NOT draw from the shared stream to avoid desyncing. Is there a lint or doc convention preventing accidental `GameRng` usage in visual-only systems?
