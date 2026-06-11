## ui-root

**Scope:** `src/ui/*.rs` (root-level files only — `layout_helpers.rs`, `markdown.rs`, `components.rs`, `constants.rs`, `color_utils.rs`, `link_button.rs`, `systems.rs`, `scale.rs`, `mod.rs`, `plugin.rs`)

---

### Mental Model

The `ui-root` layer is the shared infrastructure that every screen plugin builds on. It provides:

- **`plugin.rs`** — top-level aggregation of all sub-plugins + registration of truly global systems (UI scale, button lifecycle, parchment/glass material init).
- **`button_systems.rs`** — the 3D button visual machinery (click detection, 3D structure injection, animation, gamepad focus tinting, active-state management). Also houses two marker components (`ParchmentPanel`, `FrostedGlassOverlay`) and the `on_message` run-condition helper.
- **`layout_helpers.rs`** — spawn helpers for shared layout patterns (page containers, two-panel layout, escape handlers, scroll handling, slider rows, shadowed text) plus the two `UiMaterial` types (`ParchmentMaterial`, `FrostedGlassMaterial`).
- **`systems.rs`** — a four-line re-export hub that glob-re-exports both `button_systems` and `layout_helpers` under the `ui::systems` path so callers use a single import namespace.
- **`components.rs`** — shared component types (ButtonColors, ButtonAnimState, ButtonFront/Edge, ButtonActive, ButtonStyle) and three asset-loading startup systems (fonts, spell icons, gun icons, unit compendium sprites).
- **`constants.rs`** — global color palette, button interaction constants, shared layout dimensions, slider constants, tab bar constants, and two utility functions (`spell_category_color`, `efficiency_color`).
- **`markdown.rs`** — self-contained parser + Bevy renderer for the markdown subset used in changelog/credits/instructions screens, with a test suite.
- **`color_utils.rs`** — three pure color manipulation functions (hover/bright border derivation, alpha compositing).
- **`link_button.rs`** — `LinkButton` marker component and its click handler (opens URLs via `webbrowser`).
- **`mod.rs`** — module declarations + one re-export (`UiPlugin`, `accumulate_mode_level_stats`).

The architecture is sound: shared concerns are extracted, sub-plugins are self-contained, and the button system uses proper change detection. The main issues are a misplaced doc comment creating doc drift, a handful of ungated Update systems, the GOLD_ACCENT naming lie, inline magic-number shadow colors, and the `systems.rs` indirection layer that is an unusual pattern.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| U1 | DocDrift | `layout_helpers.rs:32–35` | Medium | S | Doc comment `/// Scales a font size down…` is pasted above `apply_parchment_backgrounds`, not above any function that scales fonts. The described function (`scale_font_by_text_width`) lives only in `button_systems.rs`. The misleading doc will confuse any reader of `layout_helpers.rs`. | Delete lines 32–35 in `layout_helpers.rs` (the four-line doc block) and write a correct doc for `apply_parchment_backgrounds` in its place. |
| U2 | ConsistencyRot | `constants.rs:34` | Medium | S | `GOLD_ACCENT` is named "gold" but its value is `hsla(270, …)` — pure purple/violet. The comment says "Purple accent for active/selected states". This name-value mismatch causes confusion; other constants correctly use "purple" in their names (e.g. `BUTTON_BORDER`, `BUTTON_GLOW_INNER`). | Rename to `PURPLE_ACCENT` across all usages (`constants.rs`, `markdown.rs`, `spell_book/constants.rs`, `wizard_tower/wizard_cards.rs`, `wizard_tower/multiplayer_tab/panels.rs`). Six total sites, mechanical rename. |
| U3 | Performance | `plugin.rs:96` | Medium | S | `update_ui_scale` runs unconditionally every `Update` frame. It reads the `Window` and `CrtEffectSettings` every tick and has an internal float-diff guard, but there is no external `run_if` or `Changed<Window>` filter. The project rule is "every Update system must have a run_if() guard." | Add `.run_if(on_event::<bevy::window::WindowResized>())` or `.run_if(resource_changed::<BevyUiScale>().or(any_with_component::<PrimaryWindow>().map(…)))` to gate the query. Alternatively, use `Query<&Window, Changed<Window>>` so the system no-ops when the window hasn't changed. |
| U4 | Performance | `plugin.rs:104` | Low | S | `animate_button_3d` runs unconditionally every `Update` frame and iterates all `ButtonAnimState` entities. It has a per-entity early-exit when `current ≈ target`, but there is no system-level `run_if` gate, so the query loop still runs every frame even when no animations are active. | Add a custom run-condition that returns `true` only when any `ButtonAnimState` has `(current - target).abs() > 0.01`, or use change-detection on `ButtonAnimState`. |
| U5 | ArchitecturalDecay | `layout_helpers.rs:696–936` | Medium | M | `spawn_slider_row` hard-codes the value display as `format!("{}%", (current_value * 100.0) as u32)` (line 924). The roguelite tab uses sliders with range `0.2–3.0` (not 0–1), so the display reads "20%"–"300%". While this accidentally reads as "20% of normal speed", it is semantically incorrect for a multiplier and the API provides no way for a caller to override the format. | Add an optional `value_format: Option<Box<dyn Fn(f32) -> String>>` field (or a simpler `show_as_percent: bool`) to `SliderRowConfig` and use the caller-supplied formatter in the `Text::new(…)` call. |
| U6 | ArchitecturalDecay | `layout_helpers.rs:269–290` | Low | S | Three inline `Color::hsla(25.0, …)` literals in `spawn_page_container`'s `BoxShadow` block (the "tight contact shadow", "medium depth shadow", "wide ambient shadow") are not exported constants. They differ in lightness/alpha from the named `SHADOW_COLOR` constant. If the palette changes these orphaned literals won't be updated. | Extract three named constants (`FRAME_SHADOW_CONTACT`, `FRAME_SHADOW_MEDIUM`, `FRAME_SHADOW_AMBIENT`) into `constants.rs` and use them in `spawn_page_container`. |
| U7 | ConsistencyRot | `systems.rs:1–4` | Low | S | `systems.rs` is a four-line glob re-export hub (`pub use super::button_systems::*; pub use super::layout_helpers::*;`). This creates a second public path for every symbol in those two files, which causes rustdoc duplication and makes it harder to know which file actually owns a symbol. The pattern is justified only if there's a historical migration context ("Phase 16" comment implies there was), but it adds ongoing confusion. | Consolidate: either delete `systems.rs` and update all callers to import directly from `button_systems` or `layout_helpers`, or document in the file comment exactly why the re-export layer must stay. |
| U8 | ArchitecturalDecay | `button_systems.rs:37–42` | Low | S | `ParchmentPanel` and `FrostedGlassOverlay` marker components are defined in `button_systems.rs` but are semantically owned by `layout_helpers.rs` (which defines the corresponding `ParchmentMaterial`/`FrostedGlassMaterial` types and the systems that use those markers). The split means a reader of `layout_helpers.rs` must look in `button_systems.rs` to find the markers for the materials defined there. | Move `ParchmentPanel` and `FrostedGlassOverlay` to `layout_helpers.rs` alongside the material types they gate. |
| U9 | ConsistencyRot | `layout_helpers.rs:734` | Low | S | `spawn_dot_leader` inline `TextColor(Color::hsla(0.0, 0.0, 0.3, 1.0))` (the dot-leader filler color) is a magic number with no named constant. It is lighter than `TEXT_PLACEHOLDER` but darker than any named muted color, making future palette adjustments miss it. | Extract as `DOT_LEADER_COLOR` constant in `constants.rs`. |
| U10 | ArchitecturalDecay | `layout_helpers.rs:850` | Low | S | Inline `BackgroundColor(Color::srgb(0.2, 0.2, 0.2))` for the slider track background. No named constant. | Extract as `SLIDER_TRACK_BG` in `constants.rs`. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|------------------------|
| `layout_helpers.rs` | 936 | No | Mixes: escape-key systems, scroll handling, page/panel spawn helpers, 3D button spawn, text shadow helpers, two UiMaterial types (with their shader structs), and the full slider row spawner. Proposed split: `materials.rs` (ParchmentMaterial, FrostedGlassMaterial + apply_* systems), `panels.rs` (page container, left detail, right scroll panel helpers), `escape.rs` (escape_to_landing/pause_main/running, consume_mouse_on_exit), `scroll.rs` (handle_scroll), `slider.rs` (SliderRowConfig + spawn_slider_row + spawn_dot_leader), `button_spawn.rs` (spawn_button, spawn_shadowed_text, spawn_title_with_shadow, spawn_page_header). The cleanup system (cleanup_screen) could live in panels.rs. |
| `button_systems.rs` | 640 | No | Mixes: click detection, 3D structure injection, anim tick, active-state enforcement, deactivation reset, gamepad focus tinting (two variants), color sync, material insertion helper, edge_color/opaque utilities, on_message run-condition, and two misplaced marker components. Proposed split: `click.rs` (button_click_detection, ButtonPressedDown, on_message), `structure.rs` (apply_3d_button_structure, insert_material_background, edge_color, opaque), `animation.rs` (animate_button_3d), `active.rs` (enforce_active_button_state, reset_deactivated_buttons), `focus_tint.rs` (apply_gamepad_focus_tint, apply_flat_gamepad_focus_tint), `sync.rs` (sync_front_face_colors, ParchmentPanel, FrostedGlassOverlay markers), `interaction.rs` (button_interaction). |
| `markdown.rs` | 506 | Yes | Single coherent concern (markdown parse + render + tests). Every line is part of one feature. No split warranted. |
| `constants.rs` | 273 | Yes | Pure constant declarations with brief comments. Under 300 lines and every entry is a shared design token. No split warranted. |

---

### Looks Bad But Is Actually Fine

- **`animate_button_3d` using `Time<Real>`** — intentional. Animations should play even when game time is paused/scaled. This is correct Bevy practice.
- **`spawn_page_container` mutating the caller-provided `content_node` before using it** (line 222 `content_node.border_radius = BorderRadius::all(…)`) — the function takes by value so mutation is safe; this avoids forcing callers to set border-radius themselves.
- **`button_interaction` and `reset_deactivated_buttons` having no `run_if` system-level guard** — both use `Changed<Interaction>` / `RemovedComponents<ButtonActive>` query filters that cause the system body to be a no-op when nothing changed. The project rule for `run_if` is best-effort for performance; change-detection filters on the query are an acceptable alternative for reactive systems.
- **`apply_3d_button_structure` skipping transparent buttons** (bg_hsla.alpha < 0.01 check at line 222) — this guards utility/invisible buttons from getting the 3D structure injected. It is intentional, not dead code.
- **`escape_to_running` not using `MenuBackPressed` message** — the comment at line 333 explains why: the message persists one extra frame after entering a paused state, which would immediately unpause. Reading gamepad buttons directly avoids the race. This is correct.
- **`FrostedGlassMaterial::new()` having `Default`-like semantics without implementing `Default`** — the struct has padding fields that must be zero; `new()` is clearer than a `Default` impl that needs `..` fallback.
- **`on_message` consuming the reader even in `run_if` context** — the run-condition is called before the system runs, so consuming `read()` there would deprive the actual system of messages. The `ButtonActionSet` is configured with `run_if(on_message::<MouseClicked>)` but `button_click_detection` is NOT in that set — it is the producer of `MouseClicked`. Only consumers are gated by `ButtonActionSet`. This ordering is correct.
- **`systems.rs` glob re-export creating dual import paths** — flagged as a consistency rot issue above, but it is worth noting that it does not cause any compile errors or ambiguity errors because all public symbols have unique names across `button_systems` and `layout_helpers`.

---

### Open Questions

1. Will `spawn_slider_row`'s hard-coded `"{}%"` format ever need to render non-percentage values (e.g., integer difficulty levels, plain multiplier strings)? If yes, a format callback field should be added before more callers accumulate.
2. The `systems.rs` re-export hub references "Phase 16" in its comment. Is there an active plan to remove this indirection once all callers have been migrated, or is this a permanent alias?
3. `update_ui_scale` runs every frame. Would switching to a `Changed<Window>`-filtered query or `on_event::<WindowResized>()` gate cause any observable startup-frame issues (i.e., does the scale need to be computed on the very first frame before any resize event fires)?
4. The `GOLD_ACCENT` constant is named gold but is purple. If renaming would touch player-facing strings or asset paths (not just Rust symbols), the scope increases — worth confirming all sites are Rust-only before scheduling the rename.
