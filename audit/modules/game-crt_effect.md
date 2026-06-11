## game-crt_effect

**Scope:** `src/game/crt_effect/` — CRT post-processing pipeline, barrel-distortion cursor correction, and auxiliary visual effects (lensing, heat distortion, teleport ripple, colorblind, high-contrast).

---

### Mental model

The module is a self-contained post-processing stack. Six render passes (lensing → teleport distortion → heat distortion → CRT → high-contrast → colorblind) are wired into Bevy's render graph via `ViewNode` implementations. Each pass reads a `ShaderType` component extracted to the render world. On the main-world side, `systems.rs` owns the barrel-distortion cursor-correction resources and the four animation systems (channel-change, screen-flash, vignette pulse, desaturation). `distortion.rs` projects world-space effect positions to screen UV each frame. `components.rs` holds both the shader-extracted settings structs and four timer resource types. The module is architecturally sound and self-contained; the main issues are a plugin.rs purity violation, a misplaced system category in `pipeline.rs`, a doc-comment paste error, a dead variable, and a per-frame heap allocation in the hot-path barrel UI correction system.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F01 | ArchitecturalDecay | `plugin.rs:205–274` | Medium | S | `plugin.rs` defines `LensingLabel`, `HeatDistortionLabel`, `CrtEffectLabel`, `CrtEffectNode`, and a full `ViewNode` implementation — substantial logic beyond Bevy registration. The project rule is plugin.rs = registration only. `CrtEffectNode` + `CrtEffectLabel` are already in `plugin.rs`; the sibling nodes live correctly in `pipeline.rs`. | Move `LensingLabel`, `HeatDistortionLabel`, `CrtEffectLabel`, and `CrtEffectNode` (with its `ViewNode` impl) into `pipeline.rs` alongside the other nodes. `plugin.rs` should only import and register them. |
| F02 | ArchitecturalDecay | `pipeline.rs:453–792` | Medium | S | `pipeline.rs` hosts `update_crt_time`, `sync_colorblind_settings`, `sync_crt_enabled`, `sync_flicker_intensity`, `sync_high_contrast` — five main-world game systems that are not pipeline initialization code. Placing main-world sync systems next to GPU pipeline init logic conflates two very different concerns and will confuse future contributors looking for CRT setting-sync logic. | Extract those five functions to `systems.rs` (or a new `sync.rs` sibling). `pipeline.rs` should contain only render pipeline resources, `init_*_pipeline` fns, and `ViewNode` implementations. |
| F03 | DocDrift | `systems.rs:332–338` | Low | S | The doc comment on `handle_screen_flash_message` reads "Projects active black hole positions to viewport-local UV space for gravitational lensing…" — it was clearly copied from the lensing update function and never updated. The actual function starts a screen-flash timer. | Replace with correct doc: "Reads `ScreenFlashMessage` and starts the screen-flash animation, replacing any in-progress flash." |
| F04 | ArchitecturalDecay | `distortion.rs:49` | Low | S | `let count = 0u32;` in `update_lensing_positions` is a dead variable. Slots 0-1 were previously used for black-hole lensing and are now always zero. The comment explains the intent, but the variable itself contributes nothing — `max_slot` can use a literal `0u32` directly. This will generate a `dead_code`/`unused_variable` warning in stricter lint runs. | Replace `let count = 0u32;` with a `const SLOT_BASE: u32 = 0;` or inline `0u32` in the `max_slot` expression, and update the comment to reflect that slots 0-1 are permanently unused. |
| F05 | ConsistencyRot | `components.rs:433–544` | Low | M | Four timer types (`ChannelChangeTimer`, `ScreenFlashTimer`, `VignettePulseTimer`, `DesaturationTimer`) each implement an identical `fn intensity() -> f32` using a sine-bell curve, `fn is_finished() -> bool`, and `fn new(…)`. The only differences are the constructor signature (some take a `color` or `peak_intensity`) and whether `is_finished` uses `>=`. This is four copies of the same ~10-line pattern. | Define a private `SineTimer { elapsed: f32, duration: f32 }` struct with `tick()`, `t()`, and `is_finished()` helpers, and have each timer delegate to it. Reduces duplication to a single place and makes future timing-curve changes (e.g., ease-in instead of sine) a one-line edit. |
| F06 | Performance | `systems.rs:180–189` | Low | S | `correct_ui_interaction_for_barrel` allocates a `Vec<(Entity, Vec2)>` on the heap every frame it runs (which is every PreUpdate frame when barrel is active). The game has exactly one camera in practice. | Use `camera_query.iter().find_map(…)` or a `SmallVec<[_; 2]>` to avoid the heap allocation for the common single-camera case. Alternatively, cache the camera entity in a `Local` since it rarely changes. |
| F07 | Performance | `plugin.rs:142` | Low | S | `correct_cursor_for_barrel_distortion` runs every `PreUpdate` frame unconditionally — no `run_if` guard. The function does bail out early when CRT is disabled, but the system still pays the scheduler overhead and queries the window+CRT component unconditionally. All other Update-schedule systems in this module have explicit `run_if` guards per project convention. | Add `.run_if(any_with_component::<CrtEffectSettings>)` (same guard as `update_crt_time`) to keep it consistent with project convention and skip the calls entirely in the menu/loading states before the camera is spawned. |

---

### Oversized files

| File | LOC | Exempt | Reason / Split proposal |
|------|-----|--------|------------------------|
| `components.rs` | 544 | No | Mixes shader-extracted settings structs (5 types) with 4 timer resource types. Propose split: `shader_settings.rs` (CrtEffectSettings, LensingSettings, HeatDistortionSettings, TeleportDistortionSettings, ColorblindCorrectionSettings, HighContrastSettings + their Default/impl) and `timers.rs` (ChannelChangeTimer, ScreenFlashTimer, VignettePulseTimer, DesaturationTimer). |
| `pipeline.rs` | 792 | Partially | Six pipeline structs + six `init_*` fns + six `ViewNode` impls is genuinely cohesive render-pipeline registry code — that portion is exempt as a registry monolith. However, 150+ lines of main-world sync systems (`update_crt_time`, `sync_*`) should be extracted (see F02), which would bring the file to ~640 LOC and keep it a clean render-only registry. Mark as not fully exempt pending F02 fix. |
| `systems.rs` | 437 | No | Two clearly distinct concerns: (1) cursor correction (barrel math, `RawCursorPosition`, `CorrectedCursorPosition`, `correct_cursor_for_barrel_distortion`, `correct_ui_interaction_for_barrel`) and (2) CRT animation systems (channel-change, flash, pulse, desaturation). Propose split: `cursor_correction.rs` and `animations.rs`. |

---

### Looks bad but is actually fine

- **`correct_ui_interaction_for_barrel` runs unconditionally in PreUpdate** (no state guard, not limited to InGame) — this is intentional and load-bearing. UI buttons exist in menu states too, and barrel distortion must correct them everywhere. The system already has a `run_if` guard in plugin.rs that checks `is_barrel_active()`, so it does skip when CRT is off. It just lacks the lighter "CrtEffectSettings exists" guard (F07 is the residual concern).
- **`unwrap_or_default()` at systems.rs:186** — this is a `Vec2` default (`Vec2::ZERO`) used as a viewport offset fallback when the camera has no physical viewport rect. Genuinely safe; `unwrap_or_default` on `Option<Vec2>` is correct here.
- **`unwrap_or` at systems.rs:270** — `focus_policy.unwrap_or(&FocusPolicy::Block)` mirrors the same default Bevy's `ui_focus_system` uses. Intentionally conservative.
- **`plugin.rs` is 274 LOC** — the scope note correctly anticipates this. The bulk is the `Plugin::build` impl body with six `ExtractComponentPlugin` pairs, eight `add_systems` calls, and six render graph node registrations. That's dense but all registration; it is not a logic violation. Only the `CrtEffectNode` and label structs after line 205 are violations (F01).
- **`TeleportDistortionSettings::set_point` match on index** — looks like repeated boilerplate but is a ShaderType constraint: fields must be individually named flat `f32`s (no arrays in WGSL uniform structs at the 16-byte-alignment boundary). The match is the correct approach given the GPU layout requirement.
- **`ndc_to_uv` is `pub(super)` in `systems.rs` but imported by `distortion.rs`** — both files are siblings under the same module, so `pub(super)` grants access to all siblings. The import is valid and the visibility is appropriately scoped.
- **`count = 0u32` in `update_lensing_positions`** — flagged as F04 (dead variable), but the comment intention is clear and it doesn't affect correctness; compiler likely eliminates it. Still worth cleaning.

---

### Open questions

1. Will slots 0-1 of lensing ever be re-enabled for black-hole lensing in multiplayer? If not, the shader dead-branches could be simplified, and `distortion.rs:49` can be removed entirely. If yes, the dead variable should become a named constant with clearer intent.
2. The six render passes run sequentially per frame even when their effects are idle (the ViewNode bails early via `count < 0.5` checks, but the pass is still enqueued). Is there a plan to cull passes at the render graph level when disabled, or is the early-out in `run()` considered sufficient?
3. `components.rs` re-exports `pub` items (no `pub(crate)`) for `CrtEffectSettings`, `LensingSettings`, etc. — these are `pub` because `ShaderType` extraction may require it in some Bevy versions. Confirm this is actually required and document it with a comment, otherwise narrow visibility.
