## music

**Scope:** `src/music/` — background music crossfade system (4 files, 221 LOC total)

---

### Mental model

The music module is a lean, self-contained crossfade manager. It loads two audio tracks at `Startup` (menu and gameplay), then watches `AppState` transitions to decide which track should be playing. Transitions trigger a fade-out on the old entity and a fade-in on a freshly-spawned entity. Volume is driven by `GameConfig` (master × music slider). Three `Update` systems handle the runtime loop, all correctly guarded with `run_if`. All files are well under 300 LOC and the module has no external dependents besides `main.rs`.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| M1 | TypeContract | `systems.rs:108` | Medium | S | `process_music_fade` queries `&mut AudioSink` directly. Bevy adds `AudioSink` to an `AudioPlayer` entity one frame after spawn (the audio backend processes it asynchronously). If `process_music_fade` runs in the same frame the entity is spawned, the entity is silently skipped (missing from the query) and the first timer tick is lost with no volume set. For a 1.5 s fade this is visually imperceptible, but the implicit timing assumption is undocumented. | Add a comment explaining the one-frame delay. Optionally use `Option<&mut AudioSink>` and `continue` explicitly so the skip is visible in code. |
| M2 | Performance | `systems.rs:104-134` | Low | S | `process_music_fade` re-reads `game_config.effective_music_volume()` every tick per fading entity. The multiply is trivial, but if the user scrubs the volume slider during a 1.5 s crossfade the fade target silently shifts mid-interpolation. A fade-out could become louder before quieter if volume is raised. | Add a `target_volume: f32` field to `MusicFade`, snapshot it at spawn time in `check_music_transition`, and use that value in `process_music_fade` so the fade arc is stable. |
| M3 | ConsistencyRot | `systems.rs:21` | Low | S | `track_for_state` has no explicit visibility modifier (defaults to `pub(self)`). The rest of the module explicitly marks internal helpers `pub(super)`. | Change to `fn track_for_state` → `pub(super) fn track_for_state` to match the module's explicit-visibility convention. |
| M4 | ErrorObservability | `systems.rs:46-48` | Low | S | When `GameConfig` is `None` inside `check_music_transition`, the function silently returns early with no log output. This can cause music to not start if timing between the `Startup` config load and the first `MusicAssets` resource-added trigger is unexpectedly off. | Add `warn!("check_music_transition: GameConfig not yet available")` before the early return. |

---

### Oversized files

_(No files in scope exceed 300 LOC.)_

---

### Looks bad but is actually fine

- **`resource_changed::<State<AppState>>.or(resource_added::<MusicAssets>)` run condition (plugin.rs:21-23):** The `.or(resource_added)` clause looks redundant since `AppState` changes during startup, but it is the safety net for the case where state is already `Splash` (default) when assets load — `resource_changed` would not fire in that case. Intentional and correct.
- **`try_despawn` instead of `despawn` (systems.rs:130):** Defensive, but correct — the entity could theoretically be removed by a scene teardown between scheduling and execution, making `try_despawn` panic-safe.
- **`Local<Option<MusicTrack>>` for previous track (systems.rs:40):** Using a `Local` instead of a `Resource` is the right call here — single-manager state, no persistence needed, avoids resource proliferation. Idiomatic Bevy.
- **`Option<Res<MusicAssets>>` and `Option<Res<GameConfig>>` (systems.rs:38-39):** Both are inserted via deferred `commands.insert_resource` in `Startup` systems, so they may not be present on the first schedule tick. The `Option` guards are required, not defensive noise.
- **`AudioPlayer::new(handle)` with no spatial sink (systems.rs:81-95):** Background music does not need spatial audio. Correct.
- **`sync_music_volume` fires on any `GameConfig` change (plugin.rs:27):** Every settings change (display mode, key bindings, etc.) triggers this. The system iterates at most a handful of `AudioSink` components with a trivial `set_volume` call — cost is negligible in practice.

---

### Open questions

1. If a third music zone is added (boss fight stinger, credits), the `MusicTrack` enum and `track_for_state` match both need updating — and `MusicAssets` needs a third `Handle`. Is there a scalability plan, or will two-track remain the ceiling?
2. Should `FADE_DURATION_SECS` be exposed in `GameConfig` for accessibility (players who want instant transitions)?
3. `MusicEntity` is a zero-size marker. If multiplayer preview snippets or UI sound-beds are ever added using the same bundle pattern, will they accidentally be caught by `check_music_transition`'s fade-out loop?
