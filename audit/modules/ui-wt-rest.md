## ui-wt-rest

**Scope:** `src/ui/wizard_tower/` excluding `study_tab/` and `roguelite_tab*` — covers `plugin.rs`, `systems.rs`, `endless_tab.rs`, `wizard_cards.rs`, `graph.rs`, `materials.rs`, `run_conditions.rs`, `layout/`, `multiplayer_tab/`, `components/`, `constants/`.

---

## Mental Model

The Wizard Tower is the pre-game hub screen. It presents a tabbed layout (Endless, Roguelite, VS, Multiplayer, Study) with a persistent left/right split panel pair. Panels are torn down and respawned on every tab change via `rebuild_panels_on_tab_change`; a second rebuild path (`rebuild_multiplayer_on_lobby_change` in `systems.rs`) handles mid-tab lobby/connection changes for the multiplayer panels only.

The `plugin.rs` (377 LOC) is a pure registration hub with thorough `run_if` guards — one of the cleaner plugin files in the project. The multiplayer subsystem is well-sliced into phase-specific panel builders (`panel_connect.rs`, `panel_hosting.rs`, etc.) and a clean state machine (`state.rs`). The `graph.rs` (598 LOC) is a math-heavy layout algorithm: node placement, repulsion, Catmull-Rom spline routing — genuinely cohesive and exempt from the 300-LOC rule.

The main tech-debt themes are:
1. `Debug`-format `{:?}` comparisons against save-file name strings in three separate places inside this scope — a fragile identity convention with no validation.
2. `spawn_coop_gated_button` helper buried in `constants/dimensions.rs` — a function in the wrong file.
3. `panel_styles.rs` in `multiplayer_tab/` has duplicate color literals that already appear as named constants in the same file (used in `text_input.rs`).
4. `MultiplayerPanelData` `SystemParam` wraps only 4 resources and is used by a single caller — project conventions discourage this.
5. `broadcast_host_mode_to_guest` allocates `Vec<String>` every frame on the roguelite-no-run path.

---

## Findings Table

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| F1 | ConsistencyRot | `wizard_cards.rs:167`, `wizard_cards.rs:453,457`, `multiplayer_tab/state.rs:186`, `layout/setup/resources.rs:14` | High | M | `format!("{:?}", wizard_type)` used as a stable identifier for save-file name matching in four in-scope sites. `Debug` output is not a stable serialisation contract — if a variant is renamed or `#[debug_handler]` is added, comparisons silently break with no compile error. `load_unified_save()` returns the player's on-disk data keyed by these strings, making this a data-integrity risk. | Add `fn save_key(&self) -> &'static str` to `WizardType` and `Spell` returning the canonical name string. Replace all `format!("{:?}", wt)` comparisons with `wt.save_key()`. |
| F2 | ArchitecturalDecay | `constants/dimensions.rs:135` | Medium | S | `spawn_coop_gated_button` is a UI helper function living in `constants/dimensions.rs`. Project rules forbid mixing behaviour into constant files; constants files hold only `const` and `use` declarations. The function is shared by `endless_tab.rs` and two `roguelite_tab/` files. | Move `spawn_coop_gated_button` and its `GUEST_NOT_READY_BUTTON_STYLE` companion to a small `coop_button.rs` sibling file, or into `ui/systems.rs` if truly cross-cutting. |
| F3 | ConsistencyRot | `multiplayer_tab/text_input.rs:16,18` | Medium | S | `text_input.rs` hardcodes `Color::hsla(270.0, 0.65, 0.55, 1.0)` and `Color::hsla(270.0, 0.35, 0.35, 1.0)` as inline literals. The identical colors are already named `CODE_BOX_BORDER_FOCUSED` and `CODE_BOX_BORDER_UNFOCUSED` in `panel_styles.rs:20-21`. Two sources of truth for the same visual value. | Import and use the constants from `panel_styles.rs` in `text_input.rs`. |
| F4 | ArchitecturalDecay | `layout/setup/resources.rs:142` | Low | S | `MultiplayerPanelData` is a `#[derive(SystemParam)]` used by exactly one system (`rebuild_panels_on_tab_change`). Project conventions say: "Do NOT create `#[derive(SystemParam)]` bundles just to reduce argument counts… Only use `SystemParam` when a group is reused across 3+ systems." Here it wraps 4 resources and exists only for one caller. | Inline the four resources directly into `rebuild_panels_on_tab_change`. The function has ~13 current parameters total, fitting within Bevy's 16-parameter limit. |
| F5 | Performance | `multiplayer_tab/sync.rs:267-282` | Low | S | `broadcast_host_mode_to_guest` runs every Update frame. On the roguelite-no-run path (`is_roguelite_no_run == true`) the early-return at line 267 is skipped unconditionally, so `roguelite_summary_lines()` is called and a `Vec<String>` is allocated each frame even when nothing changed, compared then discarded. | Gate the roguelite-no-run rebuild on `resource_changed::<RogueliteModifiers>.or(resource_changed::<PendingToggles>)`, or cache `detail_lines` in a `Local` and rebuild only on actual change. |
| F6 | ArchitecturalDecay | `multiplayer_tab/panel_styles.rs` (whole file) | Low | S | File is named `panel_styles.rs` — a near-variant of the forbidden `styles.rs` name. Content is fine (only constants), but the name signals the forbidden pattern and will confuse future reviewers checking "is styles.rs present?". | Rename to `panel_constants.rs` for unambiguity. |
| F7 | DocDrift | `endless_tab.rs:579` | Low | S | A `use crate::ui::constants::efficiency_color;` import appears mid-file between two function bodies rather than at the top-level imports block. Reads as a stale comment or accidental placement. | Move the `use` to the top of the file with other imports. |

---

## Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|------------------------|
| `src/ui/wizard_tower/plugin.rs` | 377 | true | Pure Bevy registration: only `add_systems` / `run_if` chains. No system bodies or helpers. Long because the tower has many guarded subsystems, not mixed concerns. |
| `src/ui/wizard_tower/graph.rs` | 598 | true | Single cohesive algorithm: spell-graph node placement, repulsion, edge clipping, Catmull-Rom smoothing, insight constellation layout. Splitting would sever tightly coupled math helpers from their callers. |
| `src/ui/wizard_tower/endless_tab.rs` | 624 | false | Mixes panel builders, time-travel section spawn, time-travel interaction systems, action handler, and stat helpers. Split into: `panel_builders.rs`, `time_travel.rs` (handle_time_travel_level_clicks/hover + init), `action_handler.rs` (handle_endless_actions), `stat_helpers.rs` (aggregate_endless_stats + spawn_stat_*). |
| `src/ui/wizard_tower/wizard_cards.rs` | 464 | false | Mixes grid builder, card spawn functions, interaction systems, and helpers. Split into: `grid_builder.rs` (build_wizard_card_grid, spawn_compact_card, spawn_locked_card), `card_interactions.rs` (handle_wizard_card_actions, animate_card_expand, expand_card, collapse_card, get_wizard_status). |
| `src/ui/wizard_tower/multiplayer_tab/sync.rs` | 306 | false | Contains two distinct concerns: `sync_lobby_with_connection` (transport→lobby bridge, ~167 lines) and `broadcast_host_mode_to_guest` + `sync_mp_wizard_selection` (host broadcast + wizard sync, ~140 lines). Split into: `lobby_sync.rs` and `broadcast.rs`. |

---

## Looks Bad But Is Actually Fine

- **`cleanup_study_cursor_on_area_removed` runs unconditionally in `Update`** (`plugin.rs:250`): Intentional and documented — it must run outside `MetaGameState::WizardTower` to clear `FocusNavInhibit` when the study area is despawned by leaving the tower entirely. Without this guard the main-menu focus navigation stays suppressed.

- **`MultiplayerLobby` is `PartialEq` with a heap-allocated `String` in `Failed { reason }`**: Used as a Bevy resource change-detection guard. String comparison here is infrequent (only when `Failed` is set), so not a hot path.

- **`process_lobby_messages` allocates two `Vec`s on every message** (`lobby_messages.rs:48-49`): The early-return at line 43 guards against empty queues. Allocation only happens on actual incoming messages, which are event-driven and rare.

- **`graph.rs` uses `unwrap_or(Vec2::ZERO)` at lines 491, 496**: Defensive guards against an edge referencing a non-existent node. The graph is static and self-consistent, so these never fire in practice. Acceptable per project conventions; `.expect("…")` would be more communicative.

- **`animate_card_expand` runs every Update frame**: Intentional — card height must animate continuously to target, so it cannot be change-gated. Already tightly guarded by `WizardTower + WizardSelect + SelectedWizard exists`.

- **`broadcast_host_mode_to_guest` does not use `resource_changed` guard**: The system has internal dedup via `last_sent: Local` plus scalar comparison. A `resource_changed` guard would miss the "guest just arrived" re-send case. The manual dedup is correct.

---

## Open Questions

1. **`{:?}` save-key convention (F1)**: Is `WizardType`'s `Debug` representation guaranteed stable across releases? A variant rename would silently break all existing saves. Is there a migration layer in `save_data.rs` handling this?

2. **`broadcast_host_mode_to_guest` roguelite-no-run allocation (F5)**: Does `roguelite_summary_lines` do heavy work (e.g., O(n) iteration over toggles)? If it is trivially cheap, the per-frame allocation may be acceptable as-is.

3. **`spawn_coop_gated_button` in `dimensions.rs` (F2)**: Was this placed there to avoid a circular import with `ui/systems.rs`? If so, a `coop_helpers.rs` inside `wizard_tower/` would resolve it without touching the shared module.
