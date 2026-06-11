## steam

**Scope:** `src/steam/` — Steam integration: achievements, cloud saves, leaderboards, and Steam-backed P2P multiplayer (SteamNetworkingSockets + SteamMatchmaking lobby).

---

### Mental Model

The `steam` module is a graceful-degradation wrapper: `SteamPlugin::build` branches on `SteamworksPlugin::init_app` success and only registers sub-plugins/systems inside the `Ok` arm. This means the `Client` resource is guaranteed present for every system registered here — the `resource_exists::<Client>` guards in `LeaderboardsPlugin` are therefore redundant defense-in-depth, not load-bearing.

The module splits cleanly into four sub-concerns:

1. **Achievements** (`achievements.rs`) — message-driven, one system, very clean.
2. **Cloud save** (`cloud_save.rs`) — startup restore + checkpoint sync; uses `bevy_steamworks` remote storage API.
3. **Leaderboards** (`leaderboards/`) — async handle pre-warming via `crossbeam_channel` + score submission with a well-commented scoring formula.
4. **Multiplayer** (`multiplayer/`) — lobby signalling (SteamMatchmaking) feeds into SteamNetworkingSockets P2P; bridges the async Steam callback model onto the Bevy main thread via per-channel `crossbeam_channel` pairs. Plugs into the existing `NetworkConnection` resource so the rest of the game is transport-agnostic.

The only `.unwrap()` calls in the entire module are in `#[cfg(test)]` blocks — all production paths use `match`/`if let`/`unwrap_or`. This is the codebase's clean graceful-degradation boundary: Steam failures are warnings, never panics.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| S-01 | Performance | `src/steam/plugin.rs:33` | Medium | S | `sync_achievements_to_steam` is registered to `Update` with no `run_if` guard. Project convention requires every `Update` system to have one. The system calls `client.user_stats()` — a non-trivial FFI call — every frame even when no messages are pending. | Add `.run_if(on_event::<AchievementUnlockedMessage>())` or an equivalent message-based condition to avoid the per-frame FFI call when idle. |
| S-02 | Performance | `src/steam/multiplayer/plugin.rs:33-48` | Medium | S | Eight `Update` systems (`process_create_lobby_result`, `process_join_lobby_result`, `process_lobby_chat_updates`, `process_game_lobby_join_requested`, `process_game_rich_presence_join_requested`, `consume_pending_join_in_main_menu`, `drive_steam_listen_socket`, `poll_steam_guest_connection_state`) run every frame with no `run_if` guard, violating project convention. Each has an inline early-return so they are cheap-when-idle, but function-call + borrow overhead adds up. | Wrap the entire block with `.run_if(resource_exists::<SteamLobbyBridge>)` as a proxy for "Steam MP initialized". `steam_transport_bridge_system` self-gates on `ConnectionMode::Steam` but still needs an outer `run_if`. |
| S-03 | ErrorObservability | `src/steam/cloud_save.rs:83-85` | Low | S | `sync_save_to_steam_cloud` silently discards `std::fs::read` failure with bare `Err(_) => return`. When no local save exists this is correct (nothing to sync), but any other error (permissions, path issue) is swallowed with no log line. | Use `Err(e) if e.kind() == std::io::ErrorKind::NotFound => return` for the expected case, and add a `warn!` for all other errors before returning. |
| S-04 | TypeContract | `src/steam/multiplayer/lobby_state.rs:41` | Low | S | `join_lobby_rx: Receiver<Result<LobbyId, ()>>` uses `()` as the error type because the upstream `bevy_steamworks` `join_lobby` callback uses `Result<LobbyId, ()>`. The loss of the concrete error means `process_join_lobby_result` can only log a generic "join_lobby failed" message. | Add a comment explaining that `()` mirrors the upstream API shape so future readers don't try to enrich the type and discover the constraint. If `bevy_steamworks` exposes an error type in a future version, update the channel at that point. |
| S-05 | ArchitecturalDecay | `src/steam/multiplayer/lobby_systems.rs:25-35` | Low | S | Two Startup-phase system bodies (`init_steam_lobby_bridge`, `init_relay_network_access`) live in a file named `lobby_systems.rs`, which readers expect to contain Update-phase systems. This breaks the implicit naming contract. | Move the two Startup system bodies to a new `startup.rs` sibling file, consistent with the project's feature-sliced convention. |

---

### Oversized Files

| File | LOC | Exempt | Reason / Proposed Split |
|------|-----|--------|-------------------------|
| `src/steam/multiplayer/lobby_systems.rs` | 418 | No | Contains two concerns: Startup-phase init (`init_steam_lobby_bridge`, `init_relay_network_access`) and Update-phase lobby state machine. Proposed split: `startup.rs` (init bodies), `lobby_systems.rs` (Update state machine + helpers). |
| `src/steam/multiplayer/sockets.rs` | 332 | Yes | All lines are one tightly-coupled concern: SteamNetworkingSockets lifecycle (`start_listening`, `start_connecting`, `drive_*`, `poll_*`, `tear_down_socket`) plus the transport bridge that depends on those primitives. Splitting would require passing the socket reference across a module boundary for no benefit. |

---

### Looks Bad But Is Actually Fine

- **`unwrap_or(false)` in `achievements.rs:18`** — `user_stats.achievement(api_name).get()` returns `Result<bool, ()>`. The `unwrap_or(false)` default means "assume not yet unlocked if Steam can't confirm" — causes a redundant `.set()` call at worst, never a panic. Correct behavior.
- **All three `.unwrap()` calls in the module are inside `#[cfg(test)]` blocks** — `join_requests.rs:111`, `leaderboards/systems.rs:283,287`. None are production code. The scope note's concern is fully resolved.
- **`resource_exists::<Client>` guards in `LeaderboardsPlugin`** — redundant given the `Ok`-arm registration guarantee, but cheap and make the invariant self-documenting. Not a violation.
- **`SteamLobbyState::Joined { peer, .. }` field `peer` not read by any active system** — the comment at `lobby_state.rs:29-30` explicitly marks it as reserved for "Connected to \<name\>" UI without an extra Steam round-trip. Not dead code.
- **`Mutex<Option<ListenSocket>>` on `SteamP2pSocket`** — `ListenSocket` is `!Sync` due to an internal `mpsc::Receiver`. `Mutex` is the correct and documented solution. The code comment explains this.
- **`let _ = tx.send(...)` throughout** — sending on a crossbeam unbounded channel fails only if all receivers are dropped. Since the `Receiver` lives inside the `SteamLobbyBridge` resource for the session lifetime, this is unreachable in practice. Correct to ignore.
- **`discard_stale_lobby` helper called in exactly one place** — the two-step operation (leave lobby + clear rich presence) plus its "why" comment justifies a named function over inlining.

---

### Open Questions

1. **`drive_steam_listen_socket` acquires the `Mutex` lock every frame** even when no listener is open. The early-return after `slot.as_ref()` is `None` keeps this cheap, but a `run_if` gated on `SteamLobbyState::Hosting | Joined` would eliminate the lock acquisition entirely on non-host frames. Is that worth adding alongside S-02?
2. **`total_time as i32` for clear-time leaderboard submission (`systems.rs:159`)** — `total_time` is an `f32` truncated to integer seconds. `DisplayType::TimeSeconds` shows whole seconds anyway, but truncation (e.g., 1h59m59.9s submits as 7199 rather than 7200) differs from rounding. Should this be `total_time.round() as i32`?
3. **Protocol version comparison is string-based (`lobby_systems.rs:147`)** — `u32 PROTOCOL_VERSION` is stringified and compared against a lobby metadata string. Parsing the remote value back to `u32` would enable "host is newer" vs "host is older" distinction in the error message. Worth considering for UX, but the current approach is functionally correct.
