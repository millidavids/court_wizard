## networking

**Scope:** `src/networking/` — P2P transport, wire protocol, snapshots, CRDT, session state.

---

### Mental model

The networking module is a clean, well-layered P2P stack. `NetworkConnection` is a shared resource acting as a mailbox (outgoing/incoming queues). `TransportBridgePlugin` bridges it to an async QUIC transport (`iroh`) running on a background tokio thread via crossbeam channels. `protocol.rs` defines a versioned `NetworkMessage` enum over a reliable channel; `snapshot.rs` defines compact binary structs (CRDT health, unit/spell/VFX snapshots) for the unreliable channel. `crdt.rs` implements a minimal 2-peer grow-only counter CRDT for health convergence. `session.rs` holds `MultiplayerSession` and run-condition helpers. The module is generally clean, well-documented, and correctly structured. Debt is mostly doc drift, a handful of dead fields, and one real contract gap in the codec.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| N-01 | DocDrift | `src/networking/mod.rs:1-4` | Medium | S | Module-level doc claims "P2P WebRTC communication" with "copy-paste SDP signaling". The transport has been rewritten to iroh/QUIC — WebRTC, SDP, and ICE no longer exist in the codebase. | Update the module doc to reflect iroh/QUIC + base64url ticket-code flow. |
| N-02 | DocDrift | `src/networking/resources.rs:29-34,62-63` | Medium | S | `ConnectionState::WaitingForSignaling` comment says "Waiting for user to exchange signaling codes", `ConnectionState::Connecting` says "SDP exchange complete, waiting for data channel to open", and `local_code` says "Base64-encoded SDP+ICE code". None of these are accurate for iroh. `WaitingForSignaling` is genuinely repurposed (it's now set before a ticket exists), and `local_code` is now a base64url-encoded `EndpointAddr`. | Rewrite all three doc comments to describe the iroh ticket flow. |
| N-03 | DocDrift | `src/networking/snapshot.rs:4,481` | Low | S | `snapshot.rs` header calls the unreliable channel a "WebRTC data channel". Line 481's `SpellArcSnapshot.kind` doc lists "6=Disintegrate" but kind 6 is no longer emitted (replaced by `BeamSnapshot`; confirmed at `spell_sync.rs:598-602`). | Fix header to say "QUIC datagrams". Remove "6=Disintegrate" from the kind list or annotate it as deprecated/unused. |
| N-04 | DocDrift | `src/networking/protocol.rs:3,70` | Low | S | `protocol.rs` header and the `NetworkMessage` doc both say "WebRTC data channel". | Change to "QUIC stream". |
| N-05 | ArchitecturalDecay | `src/networking/session.rs:67-71` | Low | S | `MultiplayerSession.host_spells` and `guest_spells` are both `#[allow(dead_code)]`. They are populated at session creation but never read outside of construction sites (confirmed: only written to in `lobby_messages.rs` and `init.rs`). | Remove both fields from `MultiplayerSession`. The only active consumer of guest spells is `coop.rs`, which carries `guest_spells` on its own intermediate struct, not on `MultiplayerSession`. |
| N-06 | ArchitecturalDecay | `src/networking/resources.rs:33,80-81` | Low | S | `ConnectionState::Connecting` is `#[allow(dead_code)]` in the enum definition. `NetworkConnection.ping_timer` is a public field (f32) that is never written to or read outside of `reset()` — the iroh transport does not implement application-level ping ticks. | Remove `ping_timer` from `NetworkConnection` (ping latency comes in via `TransportEvent::PingUpdate`). If `Connecting` should stay as a UI-visible state, remove `#[allow(dead_code)]`. |
| N-07 | ArchitecturalDecay | `src/networking/transport/runtime.rs:44-46` | Low | S | `TransportEvent::PingUpdate(f32)` is `#[allow(dead_code)]`. The transport layer never emits it (no ping measurement logic exists in `connection.rs`). The game currently only receives ping from Steam's lobby transport. | Either implement ping measurement on the iroh path and emit `PingUpdate`, or remove the variant and the corresponding bridge handler at `bridge.rs:162-164`. |
| N-08 | TypeContract | `src/networking/transport/codec.rs:72-73` | Medium | S | `fragment_datagram` silently truncates payloads that produce more than 255 chunks (`chunks.len().min(255) as u8`). Extra chunks are iterated but the receiver expects exactly `fragment_count` fragments — it will never complete a reassembly that was truncated, silently dropping the message. No caller enforces a payload size limit to prevent triggering this path. | Add an assertion or early-return `Vec::new()` when `chunks.len() > 255`, and log a warning. The actual payloads should be well under 255 × ~1196 bytes, but a hard contract is better than silent truncation. |
| N-09 | ConsistencyRot | `src/networking/protocol.rs:362-403` | Low | S | `StatusEffectKind` in `protocol.rs` uses manual `to_u8()` / `from_u8()` methods while all other wire enums in the same file (`SpellEffectKind`, `CastEventKind`, `SpellSoundId`, `AuraBubbleVariant`, etc.) use `TryFrom<u8>`. The pattern is inconsistent within the module. | Replace `to_u8()` / `from_u8()` with `impl TryFrom<u8>` and `Self as u8` inline, matching the rest of the file. |
| N-10 | DocDrift | `src/networking/snapshot.rs:499-507` | Low | S | `SpellSnapshotData` doc says it "Avoids exceeding Bevy's 16-parameter limit on a single system." Bevy 0.18 does not have a 16-parameter limit (the limit was lifted earlier). | Update the comment to describe the actual reason (splitting collection and transmission into two systems to keep each focused). |

---

### Oversized files

| File | LOC | Exempt | Reason / Proposed split |
|------|-----|--------|------------------------|
| `src/networking/snapshot.rs` | 833 | Yes | Every line is a wire-type definition or its serialization helper. The file is a single registry of snapshot structs, enums, `TryFrom<u8>` impls, and two small free functions — no logic. Splitting by type group would just create `unit_snapshot.rs`, `spell_snapshot.rs`, `cast_event_snapshot.rs` with identical boilerplate; the cohesion of having the whole wire format in one place outweighs the LOC concern here. |
| `src/networking/transport/connection.rs` | 558 | Yes | Pure async transport flow: host path, guest path, 4 I/O loops, encode/decode helpers. Splitting would produce `host.rs`, `guest.rs`, `io_loops.rs`, `encoding.rs` — but each function is used only once and the current file reads naturally top-to-bottom as a coherent narrative. No logic is shared across concerns that would benefit from extraction. |
| `src/networking/protocol.rs` | 480 | Yes | A single large match-on-enum registry: one `NetworkMessage` enum with exhaustive per-variant documentation, plus `StatusEffectKind`, `status_flags`, `GameOverResult`, `HostMatchSummary`, `HostMode`. All wire-type definitions. Exempt under the match/registry monolith rule. |

---

### Looks bad but is actually fine

- **`transport_bridge_system` in `PreUpdate` with no `run_if`** — This system runs every frame unconditionally, which looks like a project-convention violation. However, the system guards itself with an early return when `connection.mode == ConnectionMode::Steam` (drains stale events) and when both `has_outgoing` and `has_incoming` are false. The fast path is essentially two `is_empty()` checks and a `try_recv()` that immediately returns `Err(Empty)`. Making it `run_if(not_in_menu)` etc. would add complexity without meaningful gain since the system is already near-zero cost when idle.
- **`encode_endpoint_addr` uses `.expect()`** — `src/networking/transport/connection.rs:525`. This is on `bincode::serialize(&addr)` where `addr: &EndpointAddr`, a known-serializable library type with no fallible fields. `.expect("EndpointAddr serialization should not fail")` is correct invariant use.
- **`recently_completed.contains(&sequence)` is O(n) on VecDeque** — `codec.rs:183`. The deque is capped at 16 entries (`COMPLETED_SENTINEL_LEN`), so this is O(16) max — functionally O(1).
- **`build_unit_snapshot` has 21 boolean parameters** — `snapshot.rs:146`. The `#[allow(clippy::too_many_arguments)]` is justified: each bool maps 1-to-1 to a `UnitFlags` bit and the call sites derive each bool from a distinct ECS query component. Collapsing them into a flags bitmask at the call site would obscure which components are being checked.
- **Dual `StatusEffectKind` enums** — One at `networking/protocol.rs:362` (wire discriminator for `ApplyStatusEffect`) and one at `game/units/wizard/spell_status_effects.rs:26` (gameplay effects). They are intentionally separate: the network enum is a stable wire ordinal list; the gameplay enum drives SP systems. The name collision is a mild readability bump but the type system keeps them separate.
- **`fragment_count` mismatch detection** — `codec.rs:228`. Treating a mismatched `fragment_count` on a same-sequence packet as corrupt-and-drop is correct: fragment_count should be constant for all fragments of a given sequence, so a mismatch is either a bug or a spoofed packet.

---

### Open questions

1. **`PingUpdate` is emitted by no current code path** — should application-level RTT measurement be added to the iroh transport, or should `ping_ms` on `NetworkConnection` be populated solely by the Steam transport and left as `None` for iroh connections? If iroh-only sessions never show a ping, the UI should handle `None` gracefully (it currently shows nothing, which is fine).
2. **`WaitingForSignaling` semantics drift** — the state name was designed for a copy-paste SDP handshake but is now used to mean "the iroh endpoint is building and generating a ticket." Is it worth introducing a more accurate state name (e.g. `CreatingTicket`) to prevent future confusion?
3. **`SpellArcSnapshot.kind` gap at 2 and 3** — values 2 and 3 exist in `SpellProjectileSnapshot.kind` but not in `SpellArcSnapshot.kind` (which jumps 0→1→4→5). Is this intentional (reserved for future arc types) or do the doc comment and the match arm in `spell_sync.rs:923` need to account for an unhandled case?
