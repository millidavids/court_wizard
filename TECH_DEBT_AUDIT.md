# Tech Debt Audit — Court Wizard

Generated: 2026-06-10 · Branch: `tech-debt-overhaul` · Baseline: check ✅ clippy ✅ fmt ✅ test ✅ (52 passed)

> **Living document.** Stage 1 captured the mental model + tooling + cross-cutting findings.
> **Stage 2 (DONE):** 59 module auditors wrote `audit/modules/*.md`. **Stage 3 (DONE):**
> consolidated into `audit/CONSOLIDATED_BACKLOG.md` (+ machine-readable `audit/findings.json`).
> Resolved findings get tagged `RESOLVED` as remediation waves land.

## Stage 2/3 results (2026-06-10)

**481 findings: 2 Critical · 66 High · 151 Medium · 262 Low.** 123 non-exempt files >300 LOC;
64 correctly exempt (asset registries / match-monoliths). By category: ArchitecturalDecay 194,
ConsistencyRot 85, Performance 65, DocDrift 59, TypeContract 43, ErrorObservability 21,
Security 5, TestDebt 5, Multiplayer 4.

**⚠️ The audit surfaced real gameplay bugs, not just debt.** Both Criticals + 4 Highs are
**ghost-gating violations** (the project's #1 MP bug class): single-player systems running on
networked ghost entities. These are low-effort correctness fixes promoted to **Priority 0**
(pre-wave), independent of the refactor work:
- `battlefield/systems.rs:544` `apply_lava_damage` — missing `Without<GhostEntity>`, corrupts guest CRDT (Critical) — **RESOLVED**
- `spells/dispel/bolt.rs:606` — guest Dispel never forwards to host (Critical) — **DEFERRED** (2-part netcode fix diagnosed; needs co-op smoke-test)
- `healing_plume`, `mind_control` (×2), `squall` — SP spell systems hit ghosts (High) — **RESOLVED**
- +3 sibling ghost-gating bugs in `mind_control/effects.rs` found by the gate's cold review — **RESOLVED**

> **P0 pre-wave status (2026-06-10):** 8 ghost-gating bugs fixed across 6 systems; F133 deferred
> with full diagnosis. Gate green (check/clippy/test/native build). Awaiting maintainer co-op
> smoke-test. Details + open questions in `audit/CONSOLIDATED_BACKLOG.md`.

**Wave assignment:** A (cross-cutting foundations) 32 · B (file-splits + intra-module dedup) 107
· C (dead code + deps) 39 · D (Medium/Low quick-wins) 303. See `audit/CONSOLIDATED_BACKLOG.md`.

> The "Executive summary (Stage 1, preliminary)" below is the original orientation pass; the
> numbers above supersede it where they differ.

## Remediation progress (2026-06-11)

- **P0 correctness (ghost-gating):** 8 fixed + F133 dispel fixed (host-authority approach). Gated.
- **Wave A (cross-cutting foundations):** 14 of 32 done (`game/plugin.rs` now registration-only;
  const dedup; naming). 18 routed to B/C/maintainer-design (see backlog).
- **Wave B (file-splits):** **90 of 110 oversized files split** across 11 gated batches — every
  batch passed check/clippy(-D warnings)/test(52)/native-build with LOC-integrity verified (no
  dropped code). 300-LOC offenders 180→126 (remaining = ~64 exempt registries + ~20 deferred
  high-risk + cohesive single-fn siblings). Version 0.10.25.
- **Deferred (need individual careful handling, NOT bulk fan-out):** MP wire-format files,
  `save_data.rs` (back-compat), `units/systems.rs`+`components.rs` (dedup-coupled), the 2 `plugin.rs`,
  `shared_systems.rs`/`constants.rs`/`movement_systems.rs`/`config/resources.rs`/`combat_systems/melee.rs`
  (cross-cutting), `crt_effect/*` (shader), and files already edited for P0/F133.
- **Not yet started:** Wave C (dead code + deps incl. the 3 CVEs), Wave D (Medium/Low quick-wins).
- **Branch `tech-debt-overhaul`, nothing committed.** MP fixes await a 2-client co-op smoke-test.

---

## Executive summary (Stage 1, preliminary)

- **Scale:** 155,516 LOC across 852 `.rs` files. `src/game/units/` is half the codebase (77k);
  `units/wizard/` alone is 54k (33 spell modules + archetypes + talents).
- **Largest debt category by far: 180 files exceed the project's own 300-LOC hard limit.** This
  is the dominant structural debt and the main target of Wave B.
- **Error handling is healthy, not a debt area:** only 3 `.unwrap()` in non-test code (all in
  `src/steam/`), 0 `TODO/FIXME`, no `styles.rs`, no `events.rs` — conventions are largely
  respected. Do not pad findings here.
- **3 dependency CVEs + 3 unmaintained-crate warnings** (cargo audit): `steamworks` 0.12.2 (P2P
  DoS), transitive `hickory-dns` (O(n²) DoS), plus unmaintained `bincode 1.x` / `paste` /
  `atomic-polyfill`.
- **2 likely-unused direct deps** (cargo machete): `anyhow`, `serde_json` — needs confirmation.
- **65 `#[allow(dead_code)]` suppressions** and ~72 commented-out code lines — dead-code review
  target for Wave C.
- **Plugin/registration bloat:** 7 `plugin.rs` files >150 LOC (`multiplayer` 552, `wizard_tower`
  462, `game` 449, `achievements` 334). Most are pure registration (e.g. `multiplayer/plugin.rs`
  has 1 fn) — *not automatically* violations. `game/plugin.rs` (7 fns) holds logic and IS a
  violation candidate. Per-module auditors must judge case-by-case.
- **Debt concentrates where size × churn intersect:** `units/systems.rs` (1878 LOC, 70 changes
  /6mo), `units/components.rs` (1096, 73), `config/save_data.rs` (1603, 53),
  `game/shared_systems.rs` (88 changes). These are the highest-priority Wave A/B targets.

## Architectural mental model

Court Wizard is a Bevy 0.18 ECS real-time strategy game, native-desktop only (WASM was removed
— see project memory). `main.rs` (290 LOC) boots `App`, adds `SteamPlugin` *before*
`DefaultPlugins` (render-ordering requirement), then layers the top-level plugins:
`ConfigPlugin`, `StatePlugin`, `GamePlugin`, `MultiplayerGamePlugin`, `UiPlugin`, `MusicPlugin`,
`NetworkingPlugin`. State is an `AppState`/`MenuState`/`PauseMenuState`/`MultiplayerGameState`
machine; nearly every `Update` system is `run_if`-gated on state (a convention that appears
well-followed).

The code is **feature-sliced**: each unit/spell/UI-screen is its own module with a
registration-only `plugin.rs`, a re-export-only `mod.rs`, and concern-named feature files. The
convention set is unusually explicit (granular files ≤300 LOC, no `styles.rs`, Messages-not-
Events, shared helpers in `units/` cross-cutting files + `spells/utils.rs` + `boss/utils.rs`).
The debt is therefore **not architectural rot** — the architecture is coherent and the team
mostly follows its own rules — but **erosion at the edges**: a handful of files that grew past
the line limit and were never split, dead-code suppressions that accumulated, and dependency
drift. This matches the README/CLAUDE.md claims; no contradiction found.

**Multiplayer is the structural risk center.** `multiplayer/` (9.2k LOC) shadows the entire
single-player gameplay layer with host/guest snapshot sync, and the project's #1 recurring bug
class is SP systems not being `Without<GhostEntity>`-gated. Any remediation touching `units/`,
gameplay, lifecycle, or pathfinding must preserve those filters.

## Repo-wide tooling results (Stage 1)

| Tool | Result |
|------|--------|
| `cargo fmt --check` | ✅ clean |
| `cargo check` (win-gnu) | ✅ clean |
| `cargo clippy -D warnings` (win-gnu) | ✅ clean |
| `cargo test` | ✅ 52 passed |
| `cargo audit` | ⚠️ 3 vulns + 3 unmaintained (table below) |
| `cargo machete` | ⚠️ `anyhow`, `serde_json` flagged unused (verify) |
| `cargo udeps` | ❌ unavailable — needs rustc ≥1.93 (have 1.91.1) + nightly. Gap noted. |

### Dependency advisories (cargo audit)

| Crate | Version | ID | Severity | Fix |
|-------|---------|----|----------|-----|
| steamworks | 0.12.2 | RUSTSEC-2026-0121 | Vuln (P2P auth DoS) | ≥0.13.1 |
| hickory-dns (transitive) | — | RUSTSEC-2026-0119 | Vuln (O(n²) DoS) | ≥0.26.1 |
| atomic-polyfill (transitive) | 1.0.3 | RUSTSEC-2023-0089 | Unmaintained | — |
| bincode | 1.3.3 | RUSTSEC-2025-0141 | Unmaintained | 2.x (save-format risk; see memory) |
| paste (transitive) | 1.0.15 | RUSTSEC-2024-0436 | Unmaintained | — |

## Cross-cutting findings (Stage 1)

These span multiple top-level modules and feed **Wave A / Wave C**. `file:line`-cited per-module
findings are added by Stage 2.

| ID | Category | Evidence | Severity | Effort | Recommendation |
|----|----------|----------|----------|--------|----------------|
| X01 | Dependency/security | `Cargo.toml` steamworks 0.12.2 | High | M | Bump steamworks ≥0.13.1 (P2P DoS); verify achievements/leaderboards API. |
| X02 | Dependency/security | transitive hickory-dns | High | S | Bump via `cargo update`; confirm iroh pulls ≥0.26.1. |
| X03 | Dependency hygiene | `cargo machete`: anyhow, serde_json | Low | S | Confirm truly unused (machete false-positives on macro use); remove if so. |
| X04 | Dependency hygiene | bincode 1.x unmaintained | Medium | L | Deferred per memory (save-format risk). Track, don't action this campaign unless scoped. |
| X05 | Architectural decay | 180 files >300 LOC | High | L | Wave B per-module splits. Exempt match/registry monoliths (`visual_assets.rs`). |
| X06 | Dead code | 65 `#[allow(dead_code)]` + ~72 commented lines | Medium | M | Wave C: audit each suppression; delete genuinely dead code, justify the rest. |
| X07 | Consistency | `game/plugin.rs` holds 7 fns (logic in a registration file) | Medium | M | Move system bodies/run-conditions to sibling files; keep plugin.rs registration-only. |
| X08 | Logging hygiene | 3 `println!`/`dbg!` in src | Low | S | Replace with `bevy::log` macros. |
| X09 | Consistency | 8 `panic!`/`unreachable!`/`todo!` | Low | S | Review each; convert to graceful handling or document as invariant. |

### Things that look bad but are actually fine (Stage 1)

- **`spells/mod.rs` at 78 lines** — pure `mod` + `pub use` re-export list for 33 spells. The
  convention explicitly permits `mod.rs` to be declarations + re-exports only; line count from
  a long re-export list is **not** a violation. Same for `talents/definitions/mod.rs` (60),
  `units/mod.rs` (42).
- **`multiplayer/plugin.rs` at 552 lines with 1 fn** — it is almost entirely `add_systems`
  registration. The rule is "one plugin.rs per module, registration only" — it does *not* cap
  registration line count, and micro-plugins are explicitly forbidden. Flag for a *readability*
  look in Stage 2, but it is not a clear-cut violation the way a logic-bearing plugin.rs is.
- **`wizard/spells/visual_assets.rs` at 1478 lines** — almost certainly a single asset registry,
  which the 300-LOC rule explicitly exempts. Stage 2 must confirm before flagging.
- **Only 3 `.unwrap()`s** — all in `src/steam/`, a graceful-degradation boundary. Not debt.

## Module debt-concentration ranking (drives Stage 2 tiering)

Ranked by (size × churn × oversized-file density). Stage 2 dispatches deep auditors in this
priority order; high-tier modules get finer-grained recursion.

**Tier 1 (deepest audit — biggest, churniest, most oversized files):**
- `game/units/` root files (`systems.rs` 1878/churn-70, `components.rs` 1096/churn-73)
- `game/units/wizard/spells/` (33 modules; `arcane_crystal/auto.rs` 1039, `visual_assets.rs` 1478)
- `game/multiplayer/` (`spell_sync.rs` 1496, `guest_snapshot.rs` 1364, `host_systems.rs` 1067)
- `config/` (`save_data.rs` 1603/churn-53 — back-compat-critical)
- `ui/wizard_tower/` (`study_tab/interaction.rs` 1969, `roguelite_tab.rs` 1682, `plugin.rs` 462)
- `game/units/boss/` (`hags/core.rs` 1120, `ray/movement.rs` 1102, `hags/abilities.rs` 1024)

**Tier 2:** `ui/main_menu/settings/` (`builders.rs` 1650), `ui/compendium/` (`setup.rs` 1435),
`game/shared_systems.rs` (churn-88), `game/achievements/` (`checks.rs` 1062), `game/cauldron/`,
`game/terrain/`, `game/pathfinding/`, `game/crt_effect/`, `game/loading/`.

**Tier 3 (per-unit + small modules):** archer, infantry, king, healer, dispeller, teleporter,
shielder, aerialist, brute, assassin, commander, elite, undead; `game/input/`, `game/game_mode/`,
`game/combat_systems/`, `game/battlefield/`, `game/drops/`, `steam/`, `networking/`, `state/`,
`music/`, `game/benchmarking/`, `game/seeded_rng/`.

## Open questions for the maintainer

- Is the `bincode 1.x → 2.x` bump (X04) in scope for this campaign, or stays deferred per the
  existing save-format-risk decision? (Plan currently parks it.)
- Are the 65 `#[allow(dead_code)]` items intentional API-surface-for-later, or accreted cruft?
  Stage 2 will propose per-item, but maintainer intent helps.

---

*Per-module findings (Stage 2) and the ranked wave backlog (Stage 3) follow in
`audit/modules/*.md` and `audit/CONSOLIDATED_BACKLOG.md`.*

---

## Repeat-run audit (2026-06-11, post-campaign)

Re-ran all 59 module auditors against the restructured codebase. **445 findings** (1 Critical,
53 High, 115 Medium, 276 Low) vs the 481 baseline.

**RESOLVED by the campaign (confirmed by the re-audit):**
- **Oversized files 123 → 75 non-exempt** (~48 god-files decomposed; the 1969/1878/1682-line
  monsters are gone). Remaining 75 = cohesive single-system siblings + a few not-yet-split
  (status_effects.rs 570, shared_systems.rs 536) + new siblings.
- **All 14 plugin.rs registration-only** (no longer flagged).
- **P0 + F133 MP fixes intact** (verified: dispel queries GhostSpellEffect, squall/lava/healing/
  mind_control retain their Without<Ghost*> filters). The re-audit's dispel "Critical" and the
  squall/lava/healing/mind_control MP findings are FALSE POSITIVES — auditors re-flagged the
  original wording without re-reading the fixed code.
- Dead-code addressed (allows 65→39), unused deps removed.

**⚠️ KEY NEW FINDING — 24 pre-existing ghost-gating MP bugs (the #1 bug class), in spells the
capped original audit missed:** SP spell systems run on networked ghost entities in co-op,
double-applying damage/CC/effects. Verified-real across: finger_of_death, grease, wall_of_fire,
fireball, plague_wind, spike_growth, chain_lightning, lightning_rod, fog_cloud, sleep, haste,
battle_hymn, boss-hags mind-control. Full list in `audit/ghost_gating_findings.json`. Fix pattern
identical to P0 (add Without<GhostEntity>/Without<GhostSpellEffect> to the flagged queries).
Needs a 2-client co-op smoke-test after.

**Other notable still-open (Wave-D-deferred):** missing any_with_component perf guards
(units/plugin.rs), status_effects.rs + shared_systems.rs splits, run_conditions.rs match-arm dedup,
WaveSpawnedMessage dead channel, 16 Security-category + 69 DocDrift items. See `audit/findings.json`.

---

## Campaign resolution summary (2026-06-12)

The tech-debt overhaul (`tech-debt-overhaul` branch) is substantially complete. Status by category:

**RESOLVED**
- **Structural (file-splits):** 300-LOC offenders 180 → 116. Remaining are exempt registry/match
  monoliths (visual_assets, spell_enum, achievement_id) + ~110 cohesive single-concern siblings.
- **plugin.rs purity:** all plugin.rs are registration-only (incl. multiplayer/plugin.rs 552→40 LOC).
- **MP ghost-gating (the #1 bug class):** 24 re-audit bugs fixed (81 filters); all spell/boss systems
  verified gated.
- **Correctness bugs:** 7 fixed (teleport mana double-spend, wall-of-stone MP sync hole, banishment
  team perspective, cauldron buff stacking, ogre melee range, insight slider over-allocate, school-flare
  wire encoding) + datagram fragment overflow + staging panic.
- **Perf:** 5 hot-path wins (meteo particle asset caching, commander Local map, crystal HashSet,
  dispeller compute-once, archer unused query write-lock).
- **Dedup:** distance helpers + run_conditions match consolidated onto shared fns.
- **Dead code / deps:** removed; CVE upgrades documented as upstream-blocked.

**DEFERRED (need a decision or carry risk — not bugs to fix blindly)**
- Squall Absolute-Zero per-frame slow is framerate-dependent; fixing needs a balance retune.
- O(n²) calculate_effectiveness, healer ally-snapshot dedup, host message-target lookup: behavioral/
  MP-correctness risk — need profiling + deliberate design, not a mechanical fix.
- Talent save-key uses format!("{:?}", spell): fixing requires a save migration (back-compat).

**REMAINING (low ROI)** — ~250 Low-severity naming/doc-drift + cosmetic items (color literals,
keyboard-hint strings, a few structural clones). Best done incrementally per-module.

**NEEDS:** a 2-client co-op smoke-test before promotion (MP wire-format, ghost-gating, and
plugin-ordering changes can't be exercised by cargo test).

## MP-critical verification (2026-06-12) — Ship
- **Boot smoke:** Windows build launches, builds the full plugin graph (no ordering
  panic/ambiguity), loads assets, reaches MainMenu, saves config. Split modules
  (config/systems/{window,persist}) run correctly.
- **Consolidated cold review (fresh-context):** wire format byte-identical (all
  snapshot structs/enums, fragment headers self-consistent); ghost-gating correct on
  all sampled mutating systems, no VFX over-gated; plugin-ordering + run-condition
  locals preserved verbatim. Verdict: Ship — 0 Critical, 0 Major.
- **Known pre-existing Minor (NOT fixed — exists on main):** emit_spike_growth_rings
  + emit_fog_cloud_particles tick timers on ghost zones → doubled VFX particles on the
  guest (cosmetic). Left intentionally; gating risks invisible remote VFX depending on
  the particle-sync model — needs the live co-op test to decide.
- **Still requires** the user's manual 2-client co-op smoke-test (guest-path damage /
  remote-spell rendering) before promotion.
