## ui-wt-rest

**Scope:** `src/ui/wizard_tower/` — everything EXCEPT `study_tab/` and `roguelite_tab*`

---

### Mental model

The wizard tower is a tabbed meta-game hub (Endless, Roguelite, VS, Multiplayer, Study). This scope covers the shared scaffolding — plugin registration, layout setup, tab routing, panel rebuild orchestration, arcane rune background, wizard card grid, endless tab content, multiplayer tab (all panels, lobby state machine, sync systems), and shared constants/components/materials/graph-layout data.

The panel-rebuild architecture is event-driven at the resource level: `WizardTowerTab`, `RightPanelView`, `MultiplayerLobby`, and `NetworkConnection` changes trigger `rebuild_panels_on_tab_change` or `rebuild_multiplayer_on_lobby_change` in `plugin.rs`, which despawn children and re-spawn panel content by dispatching to per-tab builder functions. The multiplayer lobby state machine (`LobbyPhase`) is a rich enum covering the full iroh + Steam P2P lifecycle. Run conditions are consistently applied across all Update systems.

Overall this is well-structured code — the multiplayer lobby is the most complex piece and it has notably good observability (structured `info!`/`warn!` logging, no panics). The chief issues are: a helper function in `constants.rs`, `components.rs` mixing two conceptual concerns at 363 LOC, `layout/setup.rs` exceeding 300 LOC and hosting system bodies, `{:?}`-string-comparison for enum identity repeated at multiple call sites, duplicated stat-row builders that mirror `ui/compendium/rows.rs`, and a single unconditional `Update` system.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F01 | ArchitecturalDecay | `constants.rs:219` | Medium | S | `spawn_coop_gated_button` is a UI builder function living in `constants.rs`, violating the "constants only" contract. `constants.rs` should hold `const` values; helper functions belong in a peer module. | Move `spawn_coop_gated_button` (and its associated `GUEST_NOT_READY_BUTTON_STYLE` constant) to a new `src/ui/wizard_tower/coop_button.rs` or inline into the callers (`endless_tab.rs`, `roguelite_tab.rs`). |
| F02 | ArchitecturalDecay | `layout/setup.rs:548` | Medium | S | `layout/setup.rs` (852 LOC) hosts system bodies, resource definitions, components, and a `SystemParam` struct alongside layout setup code — well above 300 LOC and mixing concerns. `plugin.rs` comment says `//! Re-export hub for Phase 16` but the actual bodies are still here. | Split into: `layout/resources.rs` (tab + view enums, markers), `layout/panel_rebuild.rs` (`rebuild_panels_on_tab_change`, `compute_guest_pending`, `MultiplayerPanelData`), `layout/tab_state.rs` (`update_tab_active_state`, `update_mp_connected_indicator`), keeping `setup.rs` for `setup_wizard_tower_layout` + `handle_tab_click` + `handle_back_button` + `escape_to_main_menu`. |
| F03 | ArchitecturalDecay | `multiplayer_tab/plugin.rs:92` | Medium | S | `plugin.rs` contains four non-trivial system function bodies (`route_pending_rematch_from_menu`, `handle_pending_rematch_on_enter`, `reset_lobby_on_exit`, `multiplayer_tab_active`). Convention: plugin.rs = registration only. | Move the four system bodies to `multiplayer_tab/lifecycle.rs` (rematch routing + lobby reset) and `multiplayer_tab/run_conditions.rs` (or inline). |
| F04 | ConsistencyRot | `wizard_cards.rs:167`, `wizard_cards.rs:453`, `layout/setup.rs:28`, `multiplayer_tab/state.rs:186` | Medium | M | Four call-sites compare enum identity by formatting `{:?}` strings (e.g. `format!("{:?}", wizard_type)`) against save-file string names. This is brittle: a rename of any enum variant silently breaks the lookup without a compile error. | Add `fn save_key(&self) -> &'static str` to `WizardType` (and `Spell`) that returns a stable string key, OR derive/impl `Display` with a stable representation. Replace all `format!("{:?}", …)` comparisons with the stable key. |
| F05 | ArchitecturalDecay | `endless_tab.rs:582–623` | Low | M | `spawn_stat_row`, `spawn_stat_text_row`, `spawn_stat_value_row` in `endless_tab.rs` are near-identical duplicates of the same helpers in `src/ui/compendium/rows.rs`. Three distinct copies exist across the codebase (compendium, endless_tab, pause_menu). | Promote `spawn_stat_value_row` + wrappers to `src/ui/layout_helpers.rs` (the cross-cutting UI helper file owned by `ui-root`) with `pub(crate)` visibility, then delete the local copies. (Coordinate with `ui-root` owner.) |
| F06 | ArchitecturalDecay | `components.rs:323–363` | Low | S | `InsightAllocation` (a runtime resource with business logic: `total_allocated`, `get`, `set`, `get_bonus`, `set_bonus`) lives in `components.rs` alongside pure ECS component structs. `components.rs` should hold only component/resource marker types, not domain logic. | Move `InsightAllocation` and its `impl` block to `study_tab/allocation.rs` (or `study_tab/resources.rs`) since it is exclusively owned by the study tab and already at `pub(super)` scope. |
| F07 | Performance | `multiplayer_tab/state.rs:191` | Low | S | `load_my_unlocked_content()` calls `Spell::all().to_vec()` unconditionally, allocating a `Vec` of all spells every call. This function is called in system hot-paths (`sync_lobby_with_connection`, `process_lobby_messages`, `commit_host_start`). | Return `Spell::all()` as a `&'static [Spell]` slice or cache the unlocked spell list; the caller that needs a `Vec` can convert. Alternatively, compute unlocked spells once on `PlayerInfo` receive and store in the lobby resource. |
| F08 | ErrorObservability | `plugin.rs:244` | Low | S | `cleanup_study_cursor_on_area_removed` is registered in `Update` with no `run_if` guard at all (intentionally — it must survive state transitions). However it lacks a comment explaining why, which creates the appearance of a missing guard. Other unconditional systems in the file have inline comments. | Add a doc comment on the `add_systems` call explaining why the unconditional guard is intentional (as done for the sibling `spawn_study_cursor_on_area_added` block at line 232). |
| F09 | ConsistencyRot | `wizard_cards.rs:37–59` | Low | S | `SELECT_BUTTON_STYLE` and `EXPAND_BUTTON_STYLE` are defined in `wizard_cards.rs` using fully-qualified paths (`crate::ui::components::ButtonStyle`) instead of importing the type. This is inconsistent with every other file in the module that uses `use crate::ui::components::ButtonStyle`. | Add `use crate::ui::components::ButtonStyle;` and `use crate::ui::constants::{BUTTON_BG, BUTTON_BORDER, TEXT_PRIMARY, TEXT_MUTED};` imports and remove the inline path qualifications. |
| F10 | ArchitecturalDecay | `components.rs:240` | Low | S | `TalentProgressBarFill.spell` is annotated `#[allow(dead_code)]`, indicating the field is defined but never read. | Remove the field if it truly isn't read, or connect it to the system that reads progress bar fills. |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|-------------------------|
| `layout/setup.rs` | 852 | No | Mixed concerns: layout setup, resource definitions, component markers, system bodies, `SystemParam`. Split into `layout/resources.rs`, `layout/panel_rebuild.rs`, `layout/tab_state.rs`, keeping `setup.rs` for the one large `setup_wizard_tower_layout` function. |
| `endless_tab.rs` | 624 | Yes | Two cohesive halves (left-panel builder + time-travel builder + action handler) each internally cohesive. `build_endless_right_panel` at 147 LOC dominates; the file reads as a single feature slice. Borderline exempt. |
| `graph.rs` | 598 | Yes | Nearly all of this is a single graph-construction algorithm (`build_spell_graph`) with tightly coupled edge/node helpers (`compute_edge_waypoints`, `smooth_waypoints`, `catmull_rom`, `separate_overlapping_nodes`). This is a genuine single-algorithm monolith; splitting it would fracture internal call relationships. |
| `wizard_cards.rs` | 464 | Yes | One feature (expandable wizard card grid): builder + animation + selection system. All lines are cohesive. Borderline but justified given the animation logic requires the same component types as the builders. |
| `plugin.rs` | 462 | Yes | Pure system registration with rich `run_if` composition. Large but registration-only (with the exception of `rebuild_multiplayer_on_lobby_change` at line 431 which is a system body — a minor violation). The inline function body should be moved (see F03 analog). |
| `components.rs` | 363 | No | Two distinct concerns: (1) shared UI markers/components for the whole wizard tower, (2) `InsightAllocation` resource with domain logic. Split: keep shared UI markers here; move `InsightAllocation` to `study_tab/allocation.rs`. See F06. |
| `constants.rs` | 313 | No | Mostly constants (exempt), but contains `spawn_coop_gated_button` (a function with branching logic). Remove the function; then it is a legitimate constants file. See F01. |
| `multiplayer_tab/sync.rs` | 306 | Yes | Two tightly coupled systems (`sync_lobby_with_connection`, `broadcast_host_mode_to_guest`) that share `CoopHostSelection` and `LobbyPhase` mutation logic. Splitting would create more cross-file coupling than it removes. |

---

### Looks bad but is actually fine

- **`plugin.rs:431` `rebuild_multiplayer_on_lobby_change` defined in plugin.rs** — this is a system body in a plugin, which looks like a violation of the plugin-registration-only rule. It is actually justified: this is the only place that wires the `rebuild_multiplayer_on_lobby_change` system to both panels, and extracting it would require exposing additional query-access APIs. The comment on line 429 explains the design intent. Still, it could be moved to `layout/panel_rebuild.rs` without much pain (F03 covers the broader plugin.rs violation for the multiplayer sub-plugin).

- **`handle_join_code_input` with a hand-rolled key table** (`text_input.rs:72–113`) — looks like it should use Bevy's `ReceivedCharacter` / IME. In fact, native connection codes are ASCII-only (iroh ticket strings), and this approach is intentional for maximal portability; a full character-event pipeline would be overkill for a ~36-char hex/base32 input box.

- **`MultiplayerPanelData` SystemParam** (`layout/setup.rs:548`) — `CLAUDE.md` says not to use `SystemParam` bundles just to reduce argument counts; flag only if fewer than 3 systems use the same set. `MultiplayerPanelData` is used in two systems (`rebuild_panels_on_tab_change` + `rebuild_multiplayer_on_lobby_change`). Borderline, but both systems access the same four fields in the same roles, and the comment explicitly says it keeps the system under Bevy's 16-param limit. This is a legitimate use.

- **`escape_to_main_menu` without a run_if at the system level** — it's inside an `add_systems` block that has `.run_if(in_state(MetaGameState::WizardTower))`, so it is correctly gated.

- **`update_mp_connected_indicator` runs unconditionally (no change detection on the query)** — the system manually guards writes via `if *visibility != want_vis` and `if text.0 != desired`, which is the correct Bevy pattern for "always compute but write-through change-detect". Not a perf issue.

- **`components.rs` mixing resource + component types** — the `GraphViewState`, `GraphDragState`, `GraphViewAnimation`, `GraphBounds` resources are functionally UI-layer graph resources and belong alongside graph-related components in this file. The only clear misfit is `InsightAllocation` (F06).

---

### Open questions

1. **Stable save-key API**: Is there an existing `to_save_key()` / `from_save_key()` pattern elsewhere in the codebase for `WizardType` and `Spell`? If so, `wizard_cards.rs` and `state.rs` should use it rather than `{:?}` format strings. If not, adding one is the right fix for F04.

2. **`spawn_stat_value_row` ownership**: The `ui-root` auditor owns `layout_helpers.rs`. If they add a shared `spawn_stat_value_row` there, `endless_tab.rs` can delete its local copy. Is this cross-scope refactor tracked anywhere, or should `endless_tab.rs` just alias the compendium version?

3. **`roguelite_tab.rs` scope**: At 1682 LOC it is by far the largest file, but it is excluded from this scope. The `roguelite_tab` auditor should flag it; this scope only interacts with it through builder function imports.

4. **`TalentProgressBarFill.spell` dead field** (F10): Is this field planned for future use (e.g. to color-code bars per spell), or is it a leftover from an earlier implementation? If future use, leave it and remove `#[allow(dead_code)]` when the system is added.
