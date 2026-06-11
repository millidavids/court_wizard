## ui-wt-study_tab

**Scope:** `src/ui/wizard_tower/study_tab/` — study tab UI (spell research graph, insight bonus constellation, talent cards, sliders, gamepad cursor).

---

### Mental model

The study tab is a two-panel layout: a zoomable/pannable spell-research graph on the right and a scrollable detail panel on the left. Two separate entity clusters live inside the same graph area: the spell web (nodes + radial-progress rings + edges) and the insight-bonus constellation (concentric-ring nodes + edges). A gamepad reticle cursor drives navigation when no node is selected; selecting a node hands focus to the detail panel's `+/-` buttons, commit button, and talent cards.

The module was refactored from two monolithic files (`interaction.rs` ~1969 LOC and `panels.rs` ~1403 LOC) into a feature-sliced subtree. The split is largely clean, but several structural issues remain.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | TypeContract | `interaction/slider_interaction/slider_systems.rs:100-105` | High | S | **Bug: spell drag-slider ignores bonus allocations in `others` sum.** `handle_detail_slider_interaction` computes `others` by summing only `allocation.allocations` (spell targets), missing any insight already allocated to bonus stats. The `+/-` button path (`handle_alloc_adjust_buttons`) correctly calls `allocation.total_allocated()`. When a player drags the spell slider while bonus insight is queued, the slider can overshoot available insight. | Replace the `others` sum with `allocation.total_allocated() - allocation.get(&spell)` to match the `+/-` button logic. |
| F2 | Performance | `panels/helpers.rs:18-65` | Medium | M | **Multiple `load_unified_save()` disk reads per system call.** `is_spell_unlocked`, `is_prereq_met`, and `count_researched_spells` each independently call `load_unified_save()`. During `spawn_study_panels` they are called per-node (for every edge and every spell node in the graph), producing O(N) file reads. `update_graph_node_borders` also calls both per node on every selection change. | Accept the `UnifiedSave` once at the callsite and thread the unlocked list down. Alternatively, add a `load_unified_save()` wrapper that caches the result for one frame. |
| F3 | ConsistencyRot | `interaction/slider_interaction/slider_systems.rs:16-421` | Medium | M | **Spell-slider and insight-bonus-slider have near-identical bodies duplicated across four function pairs.** `handle_detail_slider_interaction` / `handle_insight_bonus_slider_interaction`, `update_detail_sliders` / `update_insight_bonus_sliders`, `spawn_detail_unified_slider` / `spawn_insight_bonus_slider`, and `update_allocation_text` / `update_insight_bonus_allocation_text` are structurally identical with only component-type differences. | Extract a generic drag-tracking helper and a shared `spawn_unified_slider_body` that accepts the track/fill/handle components as a small enum or via a closure. |
| F4 | ArchitecturalDecay | `interaction/slider_interaction/slider_systems.rs:363-420` | Low | S | **`update_graph_node_label_scale` and `update_star_sky_time` are placed in `slider_systems.rs` but have nothing to do with sliders.** Label scaling is a graph-view concern; star-sky time is a material animation concern. | Move `update_graph_node_label_scale` to `positions.rs` (it operates on `GraphNodeLabel` driven by `GraphViewState`). Move `update_star_sky_time` to `graph_nav.rs` or a dedicated `graph_animation.rs`. |
| F5 | ArchitecturalDecay | `panels/helpers.rs:83-148` | Low | S | **Pure geometry helpers mixed with save-data accessors in `helpers.rs`.** `graph_to_screen`, `clip_line_to_rect`, and `compute_slider_fracs` are pure math functions sharing a file with `is_spell_unlocked`, `is_prereq_met`, `count_researched_spells` (save I/O). | Split into `helpers.rs` (save-data helpers only) and `geometry.rs` (pure math). |
| F6 | Performance | `panels/positions.rs:40-64` | Low | S | **`update_graph_node_borders` calls `is_spell_unlocked` and `is_prereq_met` (each loading the save) per graph node.** The system is gated on `resource_changed::<SelectedStudySpell>` so it doesn't run every frame, but when it fires it issues 2×N disk reads. | Pre-load the save once at the top of the system and pass the unlocked list directly. |
| F7 | ConsistencyRot | `interaction/detail_panel/insight_detail.rs:36` and `interaction/detail_panel/spell_detail.rs:44` | Low | S | **Placeholder "Select a spell or bonus..." Text node is spawned identically in two separate detail-panel update functions.** Any change to the placeholder text must be made in two places. | Extract a `spawn_placeholder_text(panel)` helper in `detail_panel/mod.rs`. |

---

### Oversized files

| File | LOC | Exempt? | Reason / Proposed split |
|------|-----|---------|--------------------------|
| `panels/spawn_content.rs` | 498 | No | Mixed spawn concerns: graph-area background + spell nodes + insight constellation nodes + HUD overlays. Split into `graph_spawn.rs` (spell nodes + free node + edges), `constellation_spawn.rs` (insight bonus nodes + edges), `hud_spawn.rs` (insight balance label, pending label, debug button). |
| `interaction/detail_panel/spell_detail.rs` | 448 | No | Two distinct responsibilities: `update_study_detail_panel` (system) and `spawn_talent_section` (spawner + tier-card layout). Split into `spell_panel.rs` (the system) and `talent_section.rs` (the spawner). |
| `interaction/slider_interaction/slider_systems.rs` | 421 | No | Three distinct concerns: drag interaction logic, visual sync (fill/handle/text update), and misplaced graph-view utilities. Split into `drag_interaction.rs`, `visual_sync.rs`, and relocate graph utilities to `positions.rs` / `graph_nav.rs`. |
| `interaction/cursor.rs` | 387 | Yes | Single cohesive concern: gamepad study cursor state + all cursor systems (spawn, move, hover detect, confirm, edge-scroll, trigger-zoom, reticle appearance, cleanup). Every line is directly related. |

---

### Looks bad but is actually fine

- **`super::super::super::super::components::*` chains** in deeply nested files — correct, stable module paths; Rust handles them without runtime cost.
- **`cleanup_study_cursor_on_area_removed` has no outer state `run_if` guard** — intentional per `plugin.rs:244-250` comment: must clean up `FocusNavInhibit` even when leaving `MetaGameState::WizardTower` entirely.
- **`update_reticle_appearance` runs without `FocusNavInhibit` guard** — intentional so the reticle can hide when the detail panel opens (inhibit absent).
- **`spawn_study_panels` and `rebuild_study_ui` with `#[allow(clippy::too_many_arguments)]`** — all arguments are Bevy asset resources; idiomatic in Bevy 0.18 spawner functions.
- **`process_pending_graph_layout_refresh` calling `view.set_changed()` unconditionally** — the system is gated on `resource_exists::<PendingGraphLayoutRefresh>` which is removed once the graph area reports a non-zero size, so this never runs on idle frames.
- **Both detail-panel systems doing `despawn_related::<Children>` on every trigger** — deliberate panel-rebuild pattern; both have early-return guards on `is_changed()` so they do not run every frame.

---

### Open questions

1. Does `InsightAllocation::total_allocated()` sum both spell and bonus allocations? If so, it should replace the inline spell-only sum in `handle_detail_slider_interaction` (F1 fix is a one-liner).
2. Is `load_unified_save()` already reading from an in-memory cache (e.g., file system once per frame), or does each call hit the filesystem? If it's cached, F2 and F6 drop to Low.
3. Line 181 of `plugin.rs` references `update_graph_node_label_scale` in a `.before()` ordering constraint on `process_pending_graph_layout_refresh`, and line 225 registers it as an actual system. Verify Bevy is not running it twice per frame.
