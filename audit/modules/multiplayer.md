## multiplayer

**Scope:** `src/game/multiplayer/` — P2P multiplayer game logic (versus + co-op), host/guest networking, CRDT health sync, ghost rendering, spell sync, score screen, pause/disconnect overlays. ~12,641 LOC across 39 files.

---

### Mental model

The multiplayer module bridges two topologies sharing most of the codebase:

1. **Versus**: both peers in `AppState::MultiplayerGame`. Host runs the authoritative simulation; guest receives `GameSnapshot` unreliable packets and renders ghost units. Spell visuals and status effects flow bidirectionally via a layered system: direct CRDT health deltas, `SpellHitUnit` messages, `ApplyStatusEffect` messages, and a dedicated spell-visual snapshot channel.

2. **Co-op**: host stays in `AppState::InGame` running native SP gameplay; guest enters `AppState::MultiplayerGame`. `init_coop_host` mirrors the resource setup that `init_mp_game` does for versus, and `cleanup_coop_host` mirrors `cleanup_mp_game`. Ghost entities on the guest represent the host's authoritative army.

The ghost-gating contract is: SP gameplay/lifecycle/pathfinding systems must carry `Without<GhostEntity>` or `Without<GhostSpellEffect>` in their queries so they do not double-apply on the guest. Evidence across spell files (entangle, polymorph, lightning_rod, plague_wind, dispel, squall, etc.) shows this is consistently applied. The module itself does not violate the contract.

`plugin.rs` (552 LOC) is pure Bevy registration with detailed comments; it contains zero system bodies. All `Update` registrations carry `run_if` guards.

---

### Findings table

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| M-01 | ArchitecturalDecay | `guest_snapshot/apply_state_snapshot.rs:35` | High | L | `apply_state_snapshot` is a 726-LOC single function handling spawn, update, despawn, material swap, status-flag sync, polymorph restore, smelly/melee/mark mirroring, arrow despawn+respawn, and combat animation — all in one body. Every new ghost-side flag extension adds another branch here. | Split into concern-named helpers: `update_ghost_transforms`, `sync_ghost_status_flags`, `sync_ghost_visuals` (polymorph/combat/corpse), `despawn_stale_ghosts`, `respawn_arrows`. The outer snapshot loop orchestrates them. |
| M-02 | ArchitecturalDecay | `guest_snapshot/status_forwarding.rs:73` | High | M | `forward_status_effects_to_host` is a 440-LOC function with 12 separate query parameters and 12 independent iteration loops, one per status effect. Every new status spell requires touching this function. The structure is entirely mechanical repetition alongside the already-generic `cleanup_forwarded_marker::<T>` pattern in `plugin.rs`. | Extract a `ForwardableStatus` trait with `to_network_message()`, then drive a generic `forward_status<T: ForwardableStatus>` system registered once per type — mirroring the `cleanup` registration block. |
| M-03 | ArchitecturalDecay | `coop.rs:106–114` vs `systems/lifecycle.rs:101–108` | Medium | S | Two nearly-identical resource-init lists: `insert_host_networking_resources` (co-op host, 8 resources) and the block inside `init_mp_game` (versus/guest, same 8 resources). Currently they diverge only in extras (`CoopGuestConnected`, `LocalSpellOrigin`). The comment at `coop.rs:104` says "keep the two in sync" — a manual contract that already nearly diverged (`LocalSpellOrigin` missing from coop path). | Extract `insert_shared_mp_networking_resources(commands: &mut Commands)` and call it from both. Each site then only adds its own extras. |
| M-04 | ArchitecturalDecay | `systems/disconnected.rs:18` and `systems/pause_menu.rs:22` | Low | S | `PAUSE_BUTTON_STYLE` is defined identically in both files (identical struct literal: 250×65, border 3, font 20, same colors). | Move to `systems/constants.rs` (or inline in the module-level `components.rs`) and import in both files. |
| M-05 | Performance | `loading.rs:132` | Low | S | `MpSpawnQueue::pop_next` calls `Vec::remove(0)`, which is O(n) in the queue length. MP queues can hold 100+ tasks during loading (terrain + units). | Change `tasks: Vec<MpSpawnTask>` to `tasks: VecDeque<MpSpawnTask>` and use `pop_front()`. |
| M-06 | Performance | `host_systems/message_receive.rs:143,207`, `host_systems/status_receive.rs:71`, `host_systems/dispel_receive.rs:47` | Medium | M | Four receive systems resolve `target_network_id → Entity` with `units.iter().find_map(|...)` — O(n) over all networked units per incoming message. With 50–100 units and multiple per-frame messages (spell hit + status + dispel), these are three to four separate O(n) scans every frame. | Maintain a `HashMap<u32, Entity>` resource updated by `assign_network_ids` (insert on assign) and unit despawn (remove). The four receive systems then do O(1) lookup. |
| M-07 | DocDrift | `coop.rs:193–195` | Medium | S | `send_coop_level_over` hard-codes `host_spell_damage: 0.0` and `host_spell_healing: 0.0` with comment "WS9 fills these from the host's `LocalWizardStats`". `LocalWizardStats` IS inserted for the co-op host (line 114) and accumulates real data — it is just never read here. The guest's score silently receives zeros for the host's spell contribution. | Add `Res<LocalWizardStats>` to `send_coop_level_over`'s parameters and populate the fields. Remove the WS9 stub comment. |
| M-08 | ConsistencyRot | 15 call sites across `host_systems/`, `coop.rs`, `coop_pause.rs`, `guest_visuals/`, `systems/` | Medium | M | Every reliable-message receive system repeats the identical 8-line drain-then-re-queue boilerplate: `let msgs: Vec<…> = connection.incoming_messages.drain(..).collect(); … connection.incoming_messages.extend(unhandled);`. 15 instances across the module. | Extract a helper `drain_messages(connection, |msg| -> Option<NetworkMessage>)` that drives one loop and re-queues `Some(msg)` returns. All 15 call sites collapse to one closure. |
| M-09 | TypeContract | `host_systems/status_receive.rs:242–244` | Low | S | Fragile Form polymorph talent reduces sheep HP via the cast-time `params.sheep_hp` field (not a component), so the `ApplyStatusEffect` payload cannot represent it. The host spawns a full-HP sheep when the guest uses Fragile Form. Documented in the code comment. | Add a `fragile_form` bit to the polymorph flags byte in `status_flags`, set it in `forward_status_effects_to_host`, and apply `FRAGILE_SHEEP_HP` in `apply_status_to_entity`. One-line protocol addition, backward-compatible. |
| M-10 | TestDebt | `guest_snapshot/apply_state_snapshot.rs`, `host_systems/status_receive.rs` | Low | M | The two most complex dispatch functions have no unit tests. The ghost-gating contract and material-handle ownership rules (the "do NOT call `pick_material` on every snapshot" invariant) are intricate enough that future edits could introduce regressions. | Add unit tests for: (a) corpse→alive transition does not swap to a shared handle, (b) `apply_status_to_entity` Polymorph with `polymorph_capture=None` falls through to sleep fallback, (c) `attach_crdt_health` skips entities already having `CrdtHealth`. |

---

### Oversized files

| File | LOC | Exempt? | Reason / Split proposal |
|------|-----|---------|-------------------------|
| `guest_snapshot/apply_state_snapshot.rs` | 760 | No | Single monolithic function body; not a match/registry monolith. Split into: `ghost_spawn.rs` (new entity creation + initial component insertion), `ghost_update.rs` (transform/CRDT/status-flag updates on existing ghosts), `ghost_material.rs` (polymorph/corpse material transitions), `arrow_sync.rs` (arrow despawn+respawn) |
| `guest_snapshot/status_forwarding.rs` | 549 | No | 12 repetitive per-status iteration loops that should be generic. The `StatusEffectForwarded<T>` marker and `cleanup_forwarded_marker<T>` can stay here; extract the forwarding dispatch into `status_forward_generic.rs` |
| `loading.rs` | 722 | No | Mixes: queue-type definitions (`MpSpawnTask`, `MpSpawnQueue`, `MpLoadingSync`, `MpConfigBackup`), queue-building logic (`init_mp_loading`), and per-frame processing (`process_mp_spawn_queue`). Split into `loading/queue.rs` (types), `loading/init.rs` (building), `loading/process.rs` (per-frame execution) |
| `plugin.rs` | 552 | Yes | Pure Bevy registration + inline doc comments. Zero system bodies or helper functions. Long but all registration — exempt per project convention. |
| `host_systems/status_receive.rs` | 383 | No | `receive_apply_status_effect` (message drain + dispatch) and `apply_status_to_entity` (private match dispatch) are separate concerns. Move `apply_status_to_entity` to `host_systems/status_apply.rs`. |
| `spell_sync/ghost_spawn.rs` | 418 | Yes | Large `match kind { SpellEffectKind::X => … }` — classic registry monolith. Exempt. |
| `spell_sync/effect_collect.rs` | 425 | Yes | Query-tuple accumulator for every networked spell effect kind. All cohesive. Exempt. |
| `systems/pause_menu.rs` | 331 | No | Mixes: escape-key handler, pause UI spawn/cleanup, forfeit-confirm UI, button handlers, and co-op relabel logic. Split into `systems/pause_ui.rs` (spawn/cleanup) and `systems/pause_handlers.rs` (input + button actions). |
| `coop.rs` | 360 | No | Mixes resource type declarations, a run-condition, `start_coop_host` helper, `init_coop_host`, lifecycle systems, and `receive_coop_lifecycle`. Split into `coop/resources.rs`, `coop/host_init.rs`, `coop/lifecycle.rs`. |
| `spawning.rs` | 557 | Yes | Per-unit-type spawn helpers; each is a small focused function. Long but entirely cohesive spawn boilerplate. Exempt. |

---

### Looks bad but is actually fine

- **`attach_crdt_health` runs on ALL `Health` entities without a `GhostEntity` filter**: intentional — both host real units and guest ghost units need `CrdtHealth` for the merge pipeline. Ghost spell effects (`GhostSpellEffect`) never carry `Health`, so no spurious attachment occurs.
- **`accumulate_wizard_spell_stats` queries ALL `SpellDamageTally`/`SpellHealTally` with no wizard filter**: each tally is inserted on the target unit when the LOCAL wizard's spell lands a hit. On the guest, the local wizard's spells hit ghost units → tally lands on ghost → accumulator drains it. This correctly attributes the guest's spell output.
- **Multiple host receive systems each `drain` `incoming_messages` independently**: safe because all take `ResMut<NetworkConnection>`, which Bevy serializes. Each system sees the unhandled messages left by the previous system. No message type is handled by more than one receive function.
- **`theme_remote_excremage_ghosts` and `is_remote_excremage` run condition**: correctly gated on session state; no per-frame material allocation when the remote wizard is not Excremage.
- **`CoopPauseState::default()` pre-ticks the grace timer to 3600s to start finished**: unusual but correct; `Timer::from_seconds(GRACE_DURATION, TimerMode::Once)` followed by `grace.tick(Duration::from_secs(3600))` reliably marks it finished. The alternative `Timer::new_finished()` is not in Bevy 0.17's public API.
- **`plugin.rs` at 552 LOC**: all registration + doc comments, no system bodies. Correct per project convention.
- **`MpSpawnQueue` processes all remaining tasks in a single frame (until a deferred-state break)**: by design — MP loading needs to be fast. No visual progress bar is expected.

---

### Open questions

1. **WS9 co-op spell-stat forwarding** (M-07): Is there a planned milestone to fix the zeroed spell stats in `CoopLevelOver`? Any future achievement or co-op scoreboard feature that reads `host_spell_damage` / `host_spell_healing` will silently receive 0.
2. **Fragile Form polymorph gap** (M-09): Accepted limitation or slated for a protocol bump? Guest-cast Fragile Form polymorph gives the host a full-HP sheep — a subtle but exploitable balance gap.
3. **Unit entity lookup map** (M-06): `NetworkEntityMap` already maps remote→local for ghost entities. Could a complementary host-side `NetworkEntityId → Entity` hashmap be maintained cheaply to replace the four `find_map` O(n) scans in receive handlers?
4. **Drain-re-queue ordering (M-08)**: the sequential nature of all receive systems (each draining and re-queuing) means message order within a frame depends on system registration order in `plugin.rs`. This is currently correct but should be called out in a code comment so future registrations do not accidentally re-order message consumption.
