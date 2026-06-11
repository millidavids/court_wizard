## boss-root

**Scope:** `src/game/units/boss/*.rs` (4 root-level files: `mod.rs`, `plugin.rs`, `components.rs`, `utils.rs`)

---

### Mental model

The boss root is a thin coordination layer for five boss sub-modules (ogre, hags, lich, dark_mage, ray). It owns:

- `components.rs` — a single shared `Boss` marker component used widely across the entire codebase (UI, pathfinding, wizard spells, flocking) to identify boss entities.
- `plugin.rs` — a pure aggregator that delegates to each boss's own plugin.
- `utils.rs` — a shared utility library (~94 lines) providing cross-boss helpers: telegraph material animation, indicator rotation/despawn, eye sprite-sheet constants, sinusoidal hover bob, two-direction camera-relative facing, and a viewport visibility test.
- `mod.rs` — declarations and a single `pub(super)` re-export of `BossPlugin`.

All four files are well under the 300-line threshold. The module is lean, well-commented, and the shared helpers are genuinely reused across 2–4 boss sub-modules. There is no logic duplication within scope. The main issues are minor: two private intermediate constants that are unnecessarily named (`EYE_SHEET_WIDTH`/`EYE_SHEET_HEIGHT`), a single-use utility (`is_on_screen`) that may be better inlined, a visibility inconsistency across `mod.rs` sibling declarations, and a style drift in how callers import `utils` symbols.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| B-01 | ArchitecturalDecay | `utils.rs:7-8` | Low | S | `EYE_SHEET_WIDTH` and `EYE_SHEET_HEIGHT` are private `const` helpers used only to compute `EYE_FRAME_UV` in the same file. They are not exported. The names leak raw pixel dimensions as named constants when the meaningful public surface is only `EYE_FRAME_UV`. | Inline the arithmetic: `Vec2::new(64.0 / 256.0, 1.0)` with a comment `// 64-px square frame within a 256x64 sheet`. Remove the two intermediate constants. |
| B-02 | ArchitecturalDecay | `utils.rs:84-94` | Low | S | `is_on_screen` has exactly one call-site (`dark_mage/ai/teleport.rs:147`). A zero-reuse helper in the shared utils module adds surface area without benefit. | Either inline into `dark_mage/ai/teleport.rs` where it is the only consumer, or add a comment documenting intent to reuse in future bosses so readers understand why it lives here. |
| B-03 | ConsistencyRot | `mod.rs:1-8` | Low | S | Visibility inconsistency across sibling module declarations: `dark_mage`, `hags`, `lich`, `ray`, and `components` are `pub(crate)` (correct — UI accesses them), but `ogre` and `utils` are `pub(in crate::game)` (correct — no UI access). The narrower visibility for `ogre`/`utils` is right but silently surprising without explanation. | Add brief inline comments on the narrower declarations, e.g. `// no direct UI access; keep game-internal`. |
| B-04 | ConsistencyRot | `lich/combat/animation.rs:56,77` | Low | S | The lich animation file calls `back_facing_for_velocity` and `sinusoidal_bob` using the full crate path inline rather than a `use` import. Every other consumer of `utils` uses a `use` import at the top of the file. | Add `use crate::game::units::boss::utils::{back_facing_for_velocity, sinusoidal_bob};` and use short names. (Fix owned by lich auditor.) |
| B-05 | ConsistencyRot | `ogre/charge/charge_attack.rs:12,256` | Low | S | `charge_attack.rs` imports `despawn_indicators` and `indicator_rotation` via `use` at line 12 but calls `animate_telegraph_material` via a full inline path at line 256. Mixed import style in the same file. | Add `animate_telegraph_material` to the existing `use` import at line 12. (Fix owned by ogre auditor.) |

---

### Oversized files

| File | LOC | Exempt | Reason |
|------|-----|--------|--------|
| `utils.rs` | 94 | true | Well under 300 LOC; cohesive shared-helper library |
| `components.rs` | 5 | true | Trivially small single-component file |
| `plugin.rs` | 21 | true | Pure registration aggregator, idiomatic |
| `mod.rs` | 10 | true | Declarations and re-exports only |

---

### Looks bad but is actually fine

- **`pub(in crate::game)` visibility on utils helpers** — looks unusual compared to `pub(crate)` but is intentional: these helpers are game-internal and should not escape the `game` subtree. Correct choice.
- **`try_despawn` in `despawn_indicators`** — looks like swallowed errors, but `try_despawn` is the idiomatic Bevy 0.18 pattern for "despawn if still alive". Not an error-hiding `.unwrap()`.
- **Negated args in `indicator_rotation` (`(-direction.x).atan2(-direction.z)`)** — looks suspicious but is the correct XZ-plane heading for a flat ground indicator where the long axis aligns with `direction`. Intentional math.
- **`back_facing_for_velocity` returns `Option<FacingDirection>` with only two enum variants** — could be a bool, but returning `None` for stationary movement lets callers preserve the previous facing without a guard at the call site. Good API design.
- **`sinusoidal_bob` takes `frequency_hz` but multiplies by TAU internally** — caller passes Hz, internal math converts to radians/sec. Consistent and documented.
- **`EYE_SHEET_WIDTH`/`EYE_SHEET_HEIGHT` are private (no `pub`)** — they look like candidates for findings but are already correctly private. The only concern (B-01) is cosmetic naming noise, not a visibility leak.

---

### Open questions

1. `is_on_screen` is currently dark_mage-only. If a future boss needs viewport culling, it will naturally belong here. If no second consumer appears, should it be moved into `dark_mage/` at the next refactor pass?
2. `EYE_SHEET_*` constants describe a sprite atlas layout. If the atlas is ever resized, `EYE_FRAME_UV` must be recomputed. Is there a centralized visual-asset manifest for boss sprite sheets, or is `utils.rs` the right permanent home?
3. `TELEGRAPH_PULSE_FREQUENCY` is shared between ogre and dark_mage with a fixed 2.5 Hz. If a future boss needs a different pulse tempo, the `animate_telegraph_material` signature will need an extra parameter. Accept or parameterize now?
