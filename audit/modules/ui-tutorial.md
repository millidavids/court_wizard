## ui-tutorial

**Scope:** `src/ui/tutorial/` — all `.rs` files (15 files, ~1309 LOC total)

---

### Mental Model

The tutorial module is a self-contained overlay system that guides new players through the game's UI screens. It works by inserting/removing `ActiveTutorial` as a resource (start/stop signal), maintaining a `PendingTutorials` FIFO queue so concurrent trigger calls don't stomp each other, and rendering a modal panel with "Next" / "Skip Tutorial" buttons. Highlights are applied as absolute-positioned child entities (`HighlightOverlay`) spawned onto pre-tagged (`TutorialHighlightable`) UI entities. Text supports inline `{token}` placeholders that render as controller glyphs or keyboard fallback words. Tutorials are persisted to the unified save file. The system supports modality enforcement: KBM-only tutorials are automatically dismissed when the player switches to a gamepad.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| T01 | Performance | `plugin.rs:100–109` | Medium | S | `enforce_tutorial_modality` and `drain_pending_tutorials` run unconditionally every `Update` frame — no outer `run_if` guard. Both do have cheap `Option<Res<ActiveTutorial>>` early returns, but Bevy still schedules, dispatches, and enters both systems on every frame even when no tutorial has ever been shown. | Add `.run_if(resource_exists::<ActiveTutorial>.or(resource_exists::<PendingTutorials>))` around the chain, or at minimum `.run_if(resource_exists::<ActiveTutorial>)` for `enforce_tutorial_modality`. |
| T02 | ArchitecturalDecay | `lifecycle.rs:34–63` | Medium | S | `try_start_tutorial` accepts a `pending: Option<&PendingTutorials>` parameter but immediately discards it with `let _ = pending;` (line 53). The function then accesses `PendingTutorials` through a deferred world command instead. The parameter is misleading — it implies the caller's borrow is used, but it isn't. | Remove the `pending` parameter from `try_start_tutorial` and update all six call sites. The world-command path already safely acquires `PendingTutorials`. |
| T03 | ArchitecturalDecay | `tagging.rs:31` | Low | S | `_wt_buttons: Query<(Entity, &WizardTowerButtonAction), Without<TutorialHighlightable>>` is declared (with underscore prefix to silence the warning) but its body is never iterated. Bevy still resolves this query every frame `ActiveTutorial` exists. The `StartBattleButton` and `TimeTravelButton` `HighlightTarget` variants (deprecated `WIZARD_TOWER_STEPS` / `TIME_TRAVEL_STEPS`) are never tagged by any tagging system. | Remove the dead `_wt_buttons` parameter and the unused `WizardTowerButtonAction` import. If highlighting these buttons is needed in the future, it should be added intentionally then. |
| T04 | Performance | `resources.rs:36–38` | Low | S | `TutorialProgress::is_completed` allocates a `String` via `tutorial.id().to_string()` on every call and then performs an O(n) linear scan on `Vec<String>`. Called on every trigger invocation. `mark_completed` has the same allocation pattern. | Change `completed` to `HashSet<String>` for O(1) lookup, or compare `&str` slices directly by iterating `self.completed.iter().any(|c| c == id)` to avoid the allocation. |
| T05 | Performance | `lifecycle.rs:771–775` | Medium | S | `save_tutorial_progress` loads the entire unified save file from disk, mutates one field, and writes it back. Since this is called on every tutorial completion/skip (not just at session end), repeated quick-skips or completions incur redundant disk I/O. `load_tutorial_progress` also reads from disk at startup separately. | The save is already owned by the Bevy `SaveData` resource elsewhere in the codebase. Thread the mutation through the existing `ResMut<SaveData>` resource so the on-disk file is written once per session boundary rather than on every tutorial event. |
| T06 | DocDrift | `lifecycle.rs:164` | Low | S | The doc comment on `trigger_kbm_menus_tutorial` says "Mirrors the controller variant" but no controller-specific menus tutorial variant (`TutorialId::ControllerMenusIntro` or similar) exists in the codebase. The comment is a leftover from a planned feature. | Update the comment to remove the reference to a non-existent controller variant, or note that the gamepad path is intentionally skipped via the modality system. |
| T07 | ArchitecturalDecay | `definitions.rs:268–355` | Low | S | Three static step arrays (`WIZARD_TOWER_STEPS`, `TIME_TRAVEL_STEPS`, `STUDY_STEPS`) belong to deprecated `TutorialId` variants (`WizardTowerIntro`, `TimeTravelIntro`, `StudyIntro`). They are reachable only through `TutorialId::steps()` but no live trigger fires those IDs. They add ~88 lines of dead content and their `HighlightTarget` variants `StartBattleButton` and `TimeTravelButton` are never tagged by `tagging.rs`. | Add `#[allow(dead_code)]` already present on the enum; also document explicitly that the *step arrays* are dead content, or move them behind a `#[cfg(test)]` guard. Alternatively: retain the ID variants for save-compat (correct) but replace the step arrays with `&[]` empty slices to make the dead content obvious and trimmed. |
| T08 | ConsistencyRot | `lifecycle.rs:344–347 + 737–740` | Low | S | The "Got it" / "Next" label logic is duplicated between `spawn_tutorial_overlay` (line 344) and `update_tutorial_content` (line 737). If the wording changes, both sites must be updated. | Extract `fn next_button_label(step: usize, total: usize) -> &'static str` and call it from both sites. |
| T09 | TypeContract | `lifecycle.rs:675–758` | Medium | M | `update_tutorial_content` walks the `Children` tree two levels deep to update the "Next" button's text label (lines 743–756). This relies on the internal node structure of the spawned button (`spawn_button` → child text node → optional grandchild text node) remaining stable. Any layout refactor of `spawn_button` that adds or removes a wrapper node silently breaks the label update. | Tag the "Next" button's direct text child with a purpose-specific marker component (e.g. `TutorialNextButtonLabel`) at spawn time and query it directly, removing the tree-walk entirely. |
| T10 | ArchitecturalDecay | `lifecycle.rs:1` | High | M | `lifecycle.rs` is 782 lines and contains trigger systems, overlay spawn/despawn, highlight logic, glow animation, button handlers, save/load helpers, content update, and cleanup — seven distinct concerns. The project convention requires files >300 LOC to be split unless they are a single cohesive match-on-enum or asset registry. | Split into concern-focused siblings: `triggers.rs` (trigger_*, try_start_tutorial, drain/enforce), `overlay.rs` (spawn_tutorial_overlay, position_tutorial_panel, despawn_overlay, anchor_to_alignment), `highlight.rs` (apply_highlight, remove_all_highlights, animate_glow), `handlers.rs` (handle_next_button, handle_skip_button, complete_tutorial, cleanup_tutorial, update_tutorial_content), `persistence.rs` (load/save/reset helpers). |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|-------------------------|
| `lifecycle.rs` | 782 | No | Seven distinct concerns in one file. Split into: `triggers.rs`, `overlay.rs`, `highlight.rs`, `handlers.rs`, `persistence.rs` |
| `definitions.rs` | 505 | Yes | Pure data: enum definitions + static `&[TutorialStep]` arrays — one large static registry. No logic. Exempt per project rule. |

---

### Looks Bad But Is Actually Fine

- **`spawn_tutorial_overlay` checking `overlay_query.is_empty()`** — looks like a redundant guard, but it correctly prevents double-spawning when `ActiveTutorial` changes mid-frame before the old overlay is despawned via command.
- **`commands.queue(world closure)` in `try_start_tutorial`** — looks like an anti-pattern (bypassing the system param), but it is the correct way to mutate `PendingTutorials` from a helper function that has no direct `ResMut` access. The deferred world closure is safe.
- **`TutorialId` enum with `#[allow(dead_code)]`** — the deprecated variants (`WizardTowerIntro`, `TimeTravelIntro`, `StudyIntro`) must remain for save-backwards-compatibility. The allow is justified.
- **Five separate tagging systems instead of one** — looks like fragmentation, but the plugin comment explains this is intentional: Bevy has a system-parameter count limit. Splitting avoids hitting that limit on `tag_wizard_tower_entities` which already has 12 parameters.
- **`update_tutorial_content` running even when `ActiveTutorial` hasn't changed** — it has an explicit `is_changed()` guard at line 688 so the actual work only happens when needed.
- **`apply_highlight` not removing stale highlights when target changes** — `remove_all_highlights` is called by `handle_next_button` before advancing the step, so highlights are cleaned up correctly before `apply_highlight` sees the new step.
- **`TutorialHighlightable` never removed from entities** — these are permanent tags on UI entities that persist for the session. Since the UI screens are already cleaned up when states exit, the tags go with them. No leak.

---

### Open Questions

1. **No controller (gamepad) in-game tutorial.** `InGameIntro` is `TutorialModality::MouseKeyboard` so gamepad-first players never see any in-game guidance. Is this a deliberate product decision (controller players are assumed to be experienced), or a gap?
2. **`save_tutorial_progress` does a full save-file round-trip per tutorial event.** If the unified save is large, this could cause hitches when skipping multi-step tutorials rapidly. Should tutorial completions be batched at session-end instead?
3. **`PendingTutorials` queue is cleared in `cleanup_tutorial`** — this fires on `OnExit` for WizardTower, SpellBook, CauldronMenu, and Tutorial states. If the player opens the SpellBook while several tower tutorials are queued, the queue is wiped. Is this the intended behavior?
