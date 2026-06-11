## game-crt_effect

**Scope:** `src/game/crt_effect/` — CRT post-processing pipeline, barrel-distortion cursor correction, and auxiliary visual effects (lensing, heat distortion, teleport ripple, colorblind, high-contrast).

---

### Mental model

The module is a self-contained post-processing stack. Six render passes (lensing → teleport distortion → heat distortion → CRT → high-contrast → colorblind) are wired into Bevy's render graph via `ViewNode` implementations. Each pass reads a `ShaderType` component extracted to the render world. Main-world GPU-settings sync systems live in `pipeline/accessibility_nodes.rs`. Main-world cursor correction and animation systems live in `systems.rs`. World-space-to-screen-UV projection runs in `distortion.rs`. Components are split into `components/effect_settings.rs`, `components/accessibility_settings.rs`, and `components/timers.rs`. The module is architecturally well-structured with a few concrete violations: `plugin.rs` defines a full `ViewNode` implementation (not just registration), `systems.rs` exceeds 300 LOC mixing two unrelated concerns, `accessibility_nodes.rs` hosts main-world game-systems (not node logic), a stale doc comment, a dead variable, a per-frame heap allocation, and a missing `run_if` guard.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F01 | ArchitecturalDecay | `plugin.rs:205–274` | Medium | S | `plugin.rs` defines `LensingLabel`, `HeatDistortionLabel`, `CrtEffectLabel` (three `RenderLabel` structs), `CrtEffectNode` struct, and a full `ViewNode` implementation body. Project rule: `plugin.rs` = registration only. `CrtEffectNode` and its label belong alongside `LensingNode`/`HeatDistortionNode` in `pipeline/effect_nodes.rs`. | Move `CrtEffectLabel`, `LensingLabel`, `HeatDistortionLabel`, and `CrtEffectNode` (with `ViewNode` impl) to `pipeline/effect_nodes.rs`. `plugin.rs` imports and registers them. |
| F02 | ArchitecturalDecay | `pipeline/accessibility_nodes.rs:163–254` | Medium | S | `accessibility_nodes.rs` hosts five main-world game systems: `update_crt_time`, `sync_colorblind_settings`, `sync_crt_enabled`, `sync_high_contrast`, `sync_flicker_intensity`. These are CPU-side config-sync systems, not render node or pipeline code. Placing them here makes them hard to find when debugging CRT behavior. | Move these five functions to `systems.rs` (or a new `sync.rs` sibling). `accessibility_nodes.rs` should contain only `ColorblindCorrectionNode`, `HighContrastNode`, and their `RenderLabel` types. |
| F03 | DocDrift | `systems.rs:332–335` | Low | S | The doc comment on `handle_screen_flash_message` reads "Projects active black hole positions to viewport-local UV space for gravitational lensing. The CRT shader operates in viewport-local UV…" — clearly a paste from the lensing function that was never updated. The actual function starts a screen-flash timer. | Replace with: "Reads `ScreenFlashMessage` and starts (or replaces) the screen-flash animation." |
| F04 | ArchitecturalDecay | `distortion.rs:49` | Low | S | `let count = 0u32;` in `update_lensing_positions` is a dead variable. Slots 0-1 were previously used for black-hole lensing and are now permanently zeroed. The variable is only referenced in the final `max_slot` expression as a fallback, where it could be replaced with a literal `0u32`. Will generate a compiler warning under stricter lints. | Remove the variable; inline `0u32` in the `max_slot` expression. Update the comment at line 98 to note slots 0-1 are permanently disabled. |
| F05 | Performance | `systems.rs:180–189` | Low | S | `correct_ui_interaction_for_barrel` allocates a `Vec<(Entity, Vec2)>` on the heap every PreUpdate frame it runs (each frame barrel is active). The game has one camera in practice. | Replace with `camera_query.iter().find_map(...)` to avoid heap allocation for the single-camera case. If multiple cameras are a real concern, use `SmallVec<[_; 2]>`. |
| F06 | Performance | `plugin.rs:142` | Low | S | `correct_cursor_for_barrel_distortion` runs every PreUpdate with no `run_if` guard. It short-circuits internally but still pays scheduler and query overhead before the camera is spawned. Every other Update/PreUpdate system in this module has an explicit `run_if` guard per project convention. | Add `.run_if(any_with_component::<CrtEffectSettings>)` — same guard used by `update_crt_time`. |
| F07 | ConsistencyRot | `pipeline/accessibility_pipeline.rs:17–72` | Low | S | `TeleportDistortionPipeline` and `init_teleport_distortion_pipeline` live in `accessibility_pipeline.rs` alongside colorblind-correction and high-contrast pipelines. Teleport distortion is a gameplay visual effect, not an accessibility feature. This grouping is misleading and was flagged as inconsistent in `pipeline/mod.rs` which re-exports `init_teleport_distortion_pipeline` from `accessibility_pipeline`. | Move `TeleportDistortionPipeline` + `init_teleport_distortion_pipeline` to `crt_pipeline.rs` (or a new `distortion_pipeline.rs`) alongside the other effect pipelines (lensing, heat). |
| F08 | ConsistencyRot | `components/timers.rs:1–112` | Low | M | All four timer types (`ChannelChangeTimer`, `ScreenFlashTimer`, `VignettePulseTimer`, `DesaturationTimer`) independently implement an identical sine-bell `intensity()` + `is_finished()` pattern (~8 lines each). Any future change to the timing curve (e.g., ease-out instead of sine) requires four edits. | Extract `struct SineTimer { elapsed: f32, duration: f32 }` with `tick()`, `t()`, `is_finished()` helpers. Each timer delegates to it. Reduces duplication to one location. |

---

### Oversized files

| File | LOC | Exempt | Reason / Split proposal |
|------|-----|--------|------------------------|
| `systems.rs` | 437 | No | Two clearly distinct concerns: (1) barrel-distortion cursor correction (`RawCursorPosition`, `CorrectedCursorPosition`, `barrel_correct`, `correct_cursor_for_barrel_distortion`, `correct_ui_interaction_for_barrel`) and (2) CRT animation systems (channel-change, screen-flash, vignette-pulse, desaturation). Propose split into `cursor_correction.rs` and `animations.rs`. |
| `distortion.rs` | 359 | Yes | Single concern: projecting world-space effect positions to screen UV for three distortion types (lensing, heat, teleport). All content is genuinely cohesive. Exempt. |
| `plugin.rs` | 274 | Partially | The `Plugin::build` body (~155 lines) is pure registration and is fine. Lines 205–274 define `LensingLabel`, `HeatDistortionLabel`, `CrtEffectLabel`, `CrtEffectNode`, and a `ViewNode` impl — not registration. Extracting those to `pipeline/effect_nodes.rs` (F01) brings this file to ~155 LOC. |

---

### Looks bad but is actually fine

- **`correct_ui_interaction_for_barrel` runs in every PreUpdate when barrel is active, with no game-state guard** — intentional and load-bearing. UI buttons exist in menu states too (main menu, pause menu, etc.), and barrel distortion must correct them everywhere. The `run_if` guard in `plugin.rs:152` already skips it when CRT is disabled (`is_barrel_active()` returns false). The missing lighter guard is F06.
- **`unwrap_or_default()` at `systems.rs:186`** — falls back to `Vec2::ZERO` as viewport offset when a camera has no physical viewport rect. Safe default matching Bevy's own behavior.
- **`unwrap_or(&FocusPolicy::Block)` at `systems.rs:270`** — mirrors the identical default Bevy's `ui_focus_system` applies. Intentionally conservative and correct.
- **`TeleportDistortionSettings::set_point` match-on-index** — looks like avoidable boilerplate, but the flat `f32` fields are required by `ShaderType` / WGSL 16-byte alignment constraints. Arrays in uniform structs are not cleanly supported at that boundary; the match is the correct approach.
- **`ndc_to_uv` is `pub(super)` in `systems.rs` and imported by `distortion.rs`** — both are siblings under `crt_effect/`, so `pub(super)` grants access. Valid and appropriately scoped.
- **Five `sync_*` systems gated by `resource_changed::<GameConfig>`** — they additionally use `Local` to track per-field previous values and bail early on no-op changes. The double guard is intentional: `resource_changed` fires on any field change (e.g., volume), so the `Local` guards prevent unnecessary GPU re-uploads on irrelevant config mutations.
- **`let count = 0u32` in `distortion.rs:49`** — the compiler will optimize this away, and the code is correct. Flagged as F04 only because it is dead code that could mislead a future contributor.
- **`plugin.rs` is 274 LOC** — the note anticipated this. The bulk is legitimate Bevy registration (12 `ExtractComponentPlugin`/`UniformComponentPlugin` pairs, 8 `add_systems` calls, 6 render graph nodes + edge wiring). Not a size violation per se; the violation is the `ViewNode` impl body at the end (F01).

---

### Open questions

1. Will black-hole lensing (slots 0-1) ever be re-enabled? If not, those slots and their zeroing in `distortion.rs:27–38` can be removed entirely, simplifying the function and the shader dead-branch.
2. The six render passes are always enqueued (the `ViewNode::run` bails early via `count < 0.5`, but the pass descriptor and bind group are still created). Is render-graph-level culling (skipping node registration when effects are entirely disabled) a future optimization target?
3. The `pub` (not `pub(crate)`) visibility on `CrtEffectSettings`, `LensingSettings`, etc. in `components/mod.rs` — is this required by Bevy's `ExtractComponent`/`ShaderType` machinery, or can it be narrowed to `pub(crate)`? A comment explaining the constraint would prevent future attempts to tighten visibility unnecessarily.
