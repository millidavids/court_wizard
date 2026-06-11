## boss-root

**Scope:** `src/game/units/boss/*.rs` — shared boss components, utils, plugin wiring, and mod.rs.

---

### Mental model

The boss root layer is a thin glue layer: `components.rs` exposes a single `Boss` marker component; `utils.rs` is a well-factored shared-helper library consumed by all five boss sub-modules (hags, ogre, lich, dark_mage, ray); `plugin.rs` is registration-only; and `mod.rs` is a clean re-export hub. The layer is small (~100 LOC total), clearly factored, and follows project conventions well. The main issues are two dead/over-exported constants in `utils.rs`, one utility (`is_on_screen`) that is only used at a single call-site and may not warrant shared-helper status, and negligible style nits.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| B-01 | ArchitecturalDecay | utils.rs:7-8 | Low | S | `EYE_SHEET_WIDTH` and `EYE_SHEET_HEIGHT` are exported `pub(in crate::game)` but are never imported outside `utils.rs`. Only `EYE_FRAME_UV` (which is computed from them) is consumed by callers. The two raw dimension constants leak internal derivation detail. | Make them `const` (private/module-level, not `pub(in crate::game)`) since they are only used to define `EYE_FRAME_UV` in the same file. |
| B-02 | ArchitecturalDecay | utils.rs:84-94 | Low | S | `is_on_screen` is a general camera-NDC utility but is consumed at exactly one call-site (`dark_mage/ai.rs:547`). Keeping a zero-reuse helper in the shared utils module adds surface area without benefit. | Either inline into `dark_mage/ai.rs` (where it is the only consumer) or document the intent to reuse it, keeping it in utils. |

---

### Oversized files

All four root-level files are well under 300 LOC. No oversized file concerns.

| File | LOC | Exempt | Reason |
|------|-----|--------|--------|
| utils.rs | 94 | true | Small, cohesive shared-helper file |
| components.rs | 6 | true | Single-component file, minimal by design |
| plugin.rs | 21 | true | Registration-only, idiomatic |
| mod.rs | 11 | true | Declarations and re-exports only |

---

### Looks bad but is actually fine

- **`pub(in crate::game)` visibility on utils helpers** — looks unusual compared to `pub(crate)` but is intentional: these helpers are game-internal and should not escape the `game` subtree. Correct choice.
- **`try_despawn` in `despawn_indicators`** — looks like swallowed errors, but `try_despawn` is the idiomatic Bevy pattern for "despawn if it still exists", not an error-hiding `.unwrap()`. Fine.
- **`atan2` in `indicator_rotation`** — negating both args looks suspicious, but this is the correct XZ-plane rotation encoding for laying a rectangle flat with long axis along `direction`. Intentional.
- **`back_facing_for_velocity` returns `Option<FacingDirection>` with only two enum variants used** — looks like it could be a bool, but returning `None` for stationary movement lets callers preserve the previous facing without an extra `if speed > threshold` check at the call site. Good design.
- **`sinusoidal_bob` takes raw `frequency_hz` as f32 but multiplies by TAU internally** — caller passes Hz, internal math converts. Consistent with the rest of the codebase and clearly documented.

---

### Open questions

1. Should `is_on_screen` be promoted to a true cross-cutting utility in `src/game/shared_systems.rs` or `src/game/units/utils.rs` if other modules (e.g., future bosses, projectile culling) will need off-screen detection? Or should it be inlined into `dark_mage/ai.rs` now given its single-use status?
2. `EYE_SHEET_*` constants describe a sprite atlas layout. If the atlas is ever resized, `EYE_FRAME_UV` must be recomputed. Is there a centralized visual-asset manifest where this belongs, or is `utils.rs` the right home for boss-shared atlas metadata?
