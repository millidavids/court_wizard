## game-misc

**Scope:** `src/game/drops/`, `src/game/benchmarking/`, `src/game/seeded_rng/`

---

### Mental model

Three small, independent utility modules. **drops/** is the ingredient pickup system: enemies emit `SpawnIngredientDropMessage` on death, a bobbing billboard quad spawns, and the Telekinesis spell can fling it to the wizard where it unlocks and saves an ingredient. A `LockedIngredients` resource weights the pool toward ingredients the player does not yet own. **benchmarking/** is a `#[cfg(feature = "benchmarking")]`-gated perf overlay: F4 toggles a sampled logger that emits `BENCH …` lines every 2 s via Bevy's standard diagnostic plugins. **seeded_rng/** is the deterministic RNG foundation: one seed per run (from config or random), one `StdRng` resource, and `derive_seed()` helpers for terrain, flora, and staging to pull independent streams without order-dependency.

All three modules are small, correctly factored, and lightly coupled. Issues are shallow.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| GM-01 | DocDrift | `benchmarking/plugin.rs:9` | Medium | S | Plugin doc comment says "the F3 toggle" but `systems.rs:14` binds `KeyCode::F4`. Wrong key in docs misleads anyone using benchmarking builds. | Change doc comment to "F4 toggle". |
| GM-02 | DocDrift | `drops/components.rs:10` | Low | S | `IngredientDrop::time_alive` says "for despawn and fade" but no fade or timeout-despawn logic exists; `time_alive` is used only for bobbing/pulse animation. | Update field doc to "for bobbing and pulse animation". If a timeout-despawn is desired, add `DROP_MAX_LIFETIME` constant and a despawn branch to `tick_drop_lifetimes`. |
| GM-03 | Performance | `benchmarking/plugin.rs:27` | Low | S | `toggle_diagnostics` registers unconditionally (no `run_if`). Per project conventions every Update system needs a guard. Only relevant in benchmarking builds (feature-gated), so impact is zero in shipping code. | Add `.run_if(not(diagnostics_enabled))` or at minimum a state guard so the convention is not broken in the special build. |
| GM-04 | ArchitecturalDecay | `seeded_rng/systems.rs:8` | Low | S | `init_game_seed` is declared `pub` but is only ever called from the sibling `plugin.rs`. `pub(super)` is the correct visibility per project conventions. | Change `pub fn init_game_seed` to `pub(super) fn init_game_seed`. |
| GM-05 | ArchitecturalDecay | `seeded_rng/resources.rs:28–33` | Low | S | `derive_seed` multiplies by `purpose` as the last step, so `purpose = 0` collapses the entire output to 0 regardless of master seed or level. All three current constants are prime and non-zero, but there is no guard preventing a future zero constant from silently producing a degenerate seed. | Add a `debug_assert_ne!(purpose, 0, "derive_seed: purpose constant must be non-zero")` or reorder the multiply so purpose is not the final factor. |

---

### Oversized files

No file in scope exceeds 300 LOC. Largest is `drops/systems.rs` at 159 lines.

| File | LOC | Exempt | Reason |
|------|-----|--------|--------|
| (none over threshold) | — | — | — |

---

### Looks bad but is actually fine

- **`toggle_diagnostics` polls keyboard every frame with no run_if** — Feature-gated to `benchmarking` builds only; the function returns immediately unless `F4` is `just_pressed` and has no heap allocation. Zero production impact.
- **`FlyingToWizard` has its own `ingredient` field, duplicating `IngredientDrop`** — `drop_ops.rs:21` removes `IngredientDrop` before inserting `FlyingToWizard`, so the flying entity has no other way to know which ingredient to unlock. The field is necessary, not redundant.
- **`spawn_ingredient_drops` calls `materials.add()` and `meshes.add()` inside the message loop** — Only fires on enemy death events (0.5% chance each), not every frame. Not a hot-path allocation.
- **`unlock_ingredient` called inside Update system `fly_drops_to_wizard`** — Synchronous disk write per collected drop. Given the rarity of collection events, this is intentional and consistent with the achievement write pattern.
- **`GameRng` referenced by full path `crate::game::seeded_rng::resources::GameRng` in `drops/systems.rs:39`** — Uses full path instead of a `use` import. Style inconsistency only; no functional issue.
- **`init_locked_ingredients` calls `load_unified_save()` on every `OnEnter(Loading)`** — A redundant disk read relative to the main save load pass, but disk reads at game start are acceptable; this is not a hot path.

---

### Open questions

1. **Drop timeout:** Were fade-out and timed despawn for uncollected drops intentionally deferred or silently forgotten? The component doc referenced both, but neither was implemented. If drops persist indefinitely until `OnGameplayScreen` cleanup, that should be documented.
2. **`derive_seed` purpose-zero edge case:** Is a compile-time or debug assert appropriate, or are the prime constants considered a sufficient invariant by convention?
3. **`GameRng` in MP context:** Multiplayer visual-only systems must not draw from `GameRng` to avoid desyncing. Is there a convention or lint preventing accidental draws from the shared stream in visual systems?
