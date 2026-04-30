# Shader Effects Reference

Guide for adding new screen-space shader effects (post-processing) to Court Wizard.

## Architecture Overview

All shader effects are integrated into the existing **CRT post-processing pipeline** (`src/game/crt_effect/`). There is a single fullscreen shader pass that runs after the UI pass and before upscaling. New effects are added as additional fields on the `CrtEffectSettings` uniform and additional steps in `assets/shaders/crt_effect.wgsl`.

**Key principle:** All game shader effects (lensing, distortion, color grading, etc.) MUST render **underneath** the CRT TV effects (scanlines, RGB grid, vignette, barrel distortion, rounded corners). The CRT filter is the final visual layer — it simulates the player looking at a CRT monitor, so everything the player sees should appear filtered through it.

## File Locations

The `src/game/crt_effect/` module is feature-sliced (post-Phase-8c):

| File | Purpose |
|------|---------|
| `src/game/crt_effect/components.rs` | `CrtEffectSettings` struct (Rust ↔ GPU uniform) |
| `src/game/crt_effect/constants.rs` | Default values and tuning constants |
| `src/game/crt_effect/barrel_correction.rs` | Cursor barrel-correction + UI hit-test override |
| `src/game/crt_effect/settings.rs` | Per-frame uniform-update systems |
| `src/game/crt_effect/messages_handlers.rs` | Flash, vignette pulse, channel-change handlers |
| `src/game/crt_effect/render_node.rs` | View-node impl for the post-process pass |
| `src/game/crt_effect/pipeline.rs` | Pipeline cache, bind-group layout, shader binding |
| `src/game/crt_effect/plugin.rs` | Plugin registration only |
| `assets/shaders/crt_effect.wgsl` | The WGSL fragment shader |

When adding a new effect, place its uniform-update system in `settings.rs` (or a new feature file if the effect is large).

## Adding a New Effect — Step by Step

### 1. Add uniform fields to `CrtEffectSettings`

Add new `f32` fields to the struct in `components.rs`. This struct derives `ShaderType` (from encase) and `ExtractComponent`, so it's automatically sent to the GPU each frame.

```rust
// In CrtEffectSettings struct:
pub my_effect_intensity: f32,
pub my_effect_param: f32,
```

Also add matching defaults in the `Default` impl.

**Alignment rules:**
- Use only `f32` fields — the struct is flat scalars, no `Vec2`/`Vec4` (avoids 16-byte alignment issues between encase and WGSL)
- `ShaderType` derive handles trailing padding automatically
- The Rust struct and WGSL struct MUST have fields in the exact same order

### 2. Mirror the struct in the WGSL shader

Add the same fields in the same order to the `CrtSettings` struct in `crt_effect.wgsl`:

```wgsl
struct CrtSettings {
    // ... existing fields ...
    my_effect_intensity: f32,
    my_effect_param: f32,
}
```

### 3. Add a system to update the uniforms

In `systems.rs`, add a system that writes your effect's data to `CrtEffectSettings` each frame:

```rust
pub(super) fn update_my_effect(
    sources: Query<&MyEffectSource>,
    mut crt_query: Query<&mut CrtEffectSettings>,
) {
    let Ok(mut settings) = crt_query.single_mut() else { return };
    settings.my_effect_intensity = 0.0; // Reset each frame
    // ... compute and set values from sources ...
}
```

### 4. Register the system in `plugin.rs`

```rust
app.add_systems(
    Update,
    update_my_effect.run_if(in_state(AppState::InGame)),
);
```

Import the system in the plugin's `use` block. Every Update system MUST have a `run_if()` guard.

### 5. Add the effect to the shader

**Critical ordering:** Add your effect's UV/color modifications BEFORE the CRT effects in the fragment function. The shader is structured in numbered steps:

```
Steps 1-5:   Texture sampling (MUST come before any branches)
Step 6:      Letterbox masking
Step 7:      CRT disabled early-out
Steps 8-14:  CRT effects (barrel bounds, scanlines, RGB grid, flicker, vignette, corners)
Steps 15-16: Channel change, desaturation
Steps 17+:   Game shader effects (lensing darkening, black hole center, etc.)
```

**Where to add your effect depends on what it does:**

- **UV distortion** (bending, warping): Add between steps 1 and 2, modifying `distorted_local` or `safe_local` BEFORE texture sampling. Must be **branchless** (use `step()`, `smoothstep()`, `mix()` — no `if` statements).
- **Color modification** (tinting, darkening): Add after step 16 (after all CRT effects). Can use `if` branches since all texture samples are done.
- **Both**: Split — UV part before sampling (branchless), color part after CRT effects.

## WGSL Constraints

### No `textureSample` after branches
All `textureSample()` calls MUST appear before any non-uniform control flow (`if`, `switch`, early `return`). This is a WebGPU/WGSL requirement. If your effect needs to sample at a modified UV, compute the UV branchlessly and sample in the existing sampling block.

### Reserved keywords
WGSL reserves many common identifiers. Known pitfalls:
- `active` — use `is_active` instead
- `input`, `output`, `texture`, `sampler` — avoid these as parameter names
- Full list: https://www.w3.org/TR/WGSL/#keyword-summary

### Branchless techniques for pre-sampling code
```wgsl
// Instead of: if (count >= 1) { offset = compute(); }
// Use:
let mask = step(0.5, count);  // 0.0 or 1.0
let offset = compute() * mask;

// Instead of: if (dist < radius) { pull toward center; }
// Use:
let t = smoothstep(radius, 0.0, dist);  // 1 at center, 0 at edge
let pull = direction * t * strength;
```

### Module-scope variables
Functions can access `settings` (the uniform) directly — no need to pass it as a parameter.

## World-to-Screen Projection

To pass world-space positions to the shader (e.g., spell effect locations), project them to **viewport-local UV** space:

```rust
// 1. World → NDC via camera
let ndc = camera.world_to_ndc(camera_transform, world_pos)?;

// 2. NDC → full-window UV
let full_uv_x = (ndc.x + 1.0) * 0.5;
let full_uv_y = 1.0 - (ndc.y + 1.0) * 0.5; // Flip Y

// 3. Full-window UV → viewport-local UV (accounts for letterboxing)
let local_x = (full_uv_x - settings.viewport_x) / settings.viewport_w;
let local_y = (full_uv_y - settings.viewport_y) / settings.viewport_h;
```

The shader's `distorted_local` variable is in this viewport-local UV space. All effect positions must be in this coordinate system.

## Screen-Space Radius from World Radius

To convert a world-space radius to screen-space UV radius:

```rust
let edge_point = world_pos + camera_transform.right() * world_radius;
let edge_ndc = camera.world_to_ndc(camera_transform, edge_point)?;
let edge_uv_x = (edge_ndc.x + 1.0) * 0.5;
let screen_radius = ((edge_uv_x - full_uv_x) / settings.viewport_w).abs();
```

## Existing Effects Reference

### Gravitational Lensing (Black Hole)
- **Uniforms:** `lensing_count`, `lensing_strength`, `lensing_darkening`, `lensing_0_x/y/radius`, `lensing_1_x/y/radius`
- **UV distortion:** Branchless `lensing_offset()` function pulls pixels toward black hole centers with ring-shaped falloff (inner dead zone + outer fade)
- **Color:** Screen darkening grows gradually with black hole lifetime; center area faded to pure black
- **Supports:** Up to 2 simultaneous black holes (Twin Stars talent)
- **System:** `update_lensing_positions` runs during `AppState::InGame`, resets all values each frame

## Performance Tips

- **Reset every frame:** Always zero out your effect's uniforms at the start of your update system. This ensures effects disappear cleanly when their source entity despawns, without needing a separate cleanup system.
- **Guard with run_if:** Use `in_state(AppState::InGame)` or `any_exist::<Component>()` to skip systems when irrelevant. But if your system resets values to zero, don't guard it so tightly that stale values persist after the source despawns.
- **Minimize projections:** `camera.world_to_ndc()` involves matrix multiplication. Cache or batch when projecting multiple positions.
- **Shader branches are cheap for color ops:** Post-sampling `if` blocks are fine on modern GPUs. Only pre-sampling UV code needs to be branchless.
- **Use constants:** Put tuning values in `constants.rs`, not hardcoded in the shader. Pass them through the uniform so they can be adjusted without recompiling shaders.
