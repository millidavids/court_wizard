## music

**Scope:** `src/music/` — background music crossfading system (4 files, 221 LOC total)

---

### Mental model

The music module is a self-contained background-music manager. On startup it loads two `AudioSource` handles (menu track and gameplay track) into a `MusicAssets` resource. A `track_for_state` mapping function converts the current `AppState` to one of two `MusicTrack` variants. Three Update systems drive the runtime loop:

1. `check_music_transition` — fires when `AppState` or `MusicAssets` changes; fades out any non-fading `MusicEntity` entities and spawns the new track (at volume 0 with a `MusicFade` component, or at full volume for the very first track).
2. `process_music_fade` — ticks timers on `MusicFade` entities, interpolates `AudioSink` volume linearly, then removes the component (fade-in) or despawns the entity (fade-out).
3. `sync_music_volume` — reacts to `GameConfig` changes and snaps non-fading music to the new target volume.

All three Update systems have correct `run_if` guards. The module is small, clean, and architecturally sound. The only genuine issues are a minor implicit contract risk with `AudioSink` availability, a leaked first-play volume jump when `previous_track` resets across hot-restarts, and the fact that `process_music_fade` reads `GameConfig` every tick to determine the fade target volume rather than snapshotting it at transition time.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| M1 | TypeContract | `systems.rs:108` | Medium | S | `process_music_fade` queries `AudioSink` directly, but Bevy may not add `AudioSink` to an `AudioPlayer` entity until the audio backend processes it (next frame or later). If `process_music_fade` runs in the same frame the entity is spawned, the entity will be silently skipped (not in the query) and the first tick of fade progress is lost. For a 1.5 s fade this is imperceptible, but it is an implicit timing assumption with no comment. | Add a comment documenting the one-frame delay assumption, or filter the spawn + fade to `PostUpdate` so the sink is guaranteed present. |
| M2 | Performance | `systems.rs:104-134` | Low | S | `process_music_fade` re-reads `game_config.effective_music_volume()` on every tick for every fading entity. `effective_music_volume` is a multiply of two floats (trivial), but the pattern means the fade target can silently change mid-fade if the user scrubs the volume slider during a transition. This produces a correct but potentially jarring volume jump mid-crossfade. | Snapshot `target_volume` into `MusicFade` at spawn time (add a `target_volume: f32` field) so the fade is stable regardless of concurrent volume changes. |
| M3 | ArchitecturalDecay | `systems.rs:21-28` | Low | S | `track_for_state` is a plain free function but is not `pub(super)` — it is implicitly private to the file (no visibility modifier). This is fine Rust, but the project convention uses explicit `pub(super)` for module-internal helpers to make intent explicit. | Add `pub(super)` to `track_for_state`. |
| M4 | ErrorObservability | `systems.rs:75` | Low | S | `game_config.effective_music_volume()` is called inside `check_music_transition` via `Option<Res<GameConfig>>`. If `GameConfig` is missing at transition time (i.e., during the very first `resource_added::<MusicAssets>` trigger before config loads), the function silently returns without spawning any music. There is no log warning to aid debugging. | Add `warn!("check_music_transition: GameConfig not yet available, skipping")` before the early return so this silent skip is observable. |

---

### Oversized files

| File | LOC | Exempt | Reason |
|------|-----|--------|--------|
| `systems.rs` | 146 | — | Well under the 300-LOC threshold; no split needed. |

_(No files in scope exceed 300 LOC.)_

---

### Looks bad but is actually fine

- **`resource_changed::<State<AppState>>` used as run condition (plugin.rs:22):** This looks like it might miss the initial state, but the `.or(resource_added::<MusicAssets>)` clause covers the boot case — on first availability of assets the transition fires regardless of state change. Intentional and correct.
- **`try_despawn` instead of `despawn` (systems.rs:130):** Looks defensive/sloppy at first glance, but `try_despawn` is correct here because a fade-out entity could theoretically be despawned by another system (e.g., a scene teardown) between scheduling and execution. Using `try_despawn` avoids a panic with no behavioral downside.
- **`Local<Option<MusicTrack>>` as previous-track state (systems.rs:40):** Using a `Local` instead of a component or resource looks unconventional, but it is the right choice here — there is exactly one music manager, it needs no persistence across save/load, and a `Local` prevents resource proliferation. Intentional and idiomatic Bevy.
- **`Option<Res<MusicAssets>>` and `Option<Res<GameConfig>>` (systems.rs:38-39):** Making resources optional in a system that could be called before they exist is the correct pattern for startup sequencing in Bevy. Not a design smell.
- **`AudioPlayer::new(handle)` bundle with no `SpatialAudioSink` (systems.rs:81-95):** Music does not need spatial audio. This is correct.

---

### Open questions

1. When the game eventually adds a third music zone (e.g., a boss arena or credits screen), the `track_for_state` match and `MusicTrack` enum both need updating. Is there a plan to extend `MusicTrack` to more than two variants, and if so, would a `Handle<AudioSource>` map keyed on `MusicTrack` (or on `AppState`) be cleaner than the current match arms?
2. If the volume slider is changed during a crossfade, the fading-out entity's volume is driven to `target * fraction` (finding M2), but `target` is the *new* target. This means the fade-out could get louder before it gets quieter if the user raises volume mid-transition. Is this acceptable UX?
3. `MusicEntity` is currently a zero-size marker component. If multiplayer "ghost" tracks or preview snippets are ever added, will they share the same marker and accidentally be caught by `check_music_transition`'s fade-out loop?
