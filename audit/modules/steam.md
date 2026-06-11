## steam

**Scope:** `src/steam/` — Steam SDK integration (achievements, cloud saves, leaderboards, multiplayer lobby + P2P sockets).

---

### Mental model

The module is a thin integration layer over `bevy_steamworks`. It splits cleanly into four concerns: achievement forwarding (`achievements.rs`), cloud-save sync (`cloud_save.rs`), leaderboards (`leaderboards/`), and Steam Matchmaking + SteamNetworkingSockets multiplayer (`multiplayer/`). `SteamPlugin::build` wraps the entire surface in a single `match SteamworksPlugin::init_app(...)` — Steam absent ⇒ nothing is registered, so every downstream resource and system is inherently optional, which explains the pervasive `Option<Res<Client>>` parameters in lobby systems. The `.unwrap()` calls flagged in the scope note live exclusively in test code (`leaderboards/systems.rs:283,287`, `join_requests.rs:111`), not in production paths — this is the graceful-degradation boundary working correctly. The main structural concern is that several `Update` systems in `SteamMultiplayerPlugin` have no state-based `run_if` guards, plus two files exceed 300 LOC with mixed concerns.

---

### Findings

| ID | Category | File:Line | Severity | Effort | Description | Recommendation |
|----|----------|-----------|----------|--------|-------------|----------------|
| S1 | Performance | `multiplayer/plugin.rs:34–49` | High | S | Nine `Update` systems (`process_create_lobby_result`, `process_join_lobby_result`, `process_lobby_chat_updates`, `process_game_lobby_join_requested`, `process_game_rich_presence_join_requested`, `consume_pending_join_in_main_menu`, `drive_steam_listen_socket`, `poll_steam_guest_connection_state`, `steam_transport_bridge_system`) run unconditionally every frame with no state guard. During a solo run, all nine cross the FFI boundary or drain channels on every tick for no reason. | Chain the group with `.run_if(in_state(ConnectionMode::Steam).or(resource_changed::<SteamLobbyState>))`, or at minimum `.run_if(resource_exists::<SteamLobbyBridge>)` so they no-op during solo play. |
| S2 | Performance | `plugin.rs:33` | Medium | S | `sync_achievements_to_steam` runs unconditionally in `Update` with no `run_if` guard. The comment says "runs in all states because some achievements fire in menus" — that justifies the state coverage but not the frame-every-tick cost. Even with an empty `MessageReader` the system still schedules, acquires the `Client` borrow, and calls `client.user_stats()` every frame. | Add `.run_if(on_event::<AchievementUnlockedMessage>())` or the message-based equivalent so the system only runs when there is actually a message to process. |
| S3 | ErrorObservability | `cloud_save.rs:82–84` | Medium | S | `sync_save_to_steam_cloud` silently swallows `std::fs::read` failures with `Err(_) => return`. If the local save path exists (returned by `cloud_save_path`) but the file fails to open (permissions, lock, corruption), the cloud is silently left stale with no log entry. | Replace `Err(_) => return` with `Err(e) => { warn!("Steam Cloud: failed to read local save for upload: {e}"); return; }`. |
| S4 | ArchitecturalDecay | `multiplayer/lobby_systems.rs:1–404` | Medium | M | 404 LOC mixing three distinct concerns: create-lobby flow (`process_create_lobby_result`, `discard_stale_lobby`), join-lobby flow (`process_join_lobby_result`, `accept_incoming_join`), and chat/presence updates (`process_lobby_chat_updates`, `process_game_lobby_join_requested`, `process_game_rich_presence_join_requested`, `sync_coop_peer_name`). The 300-LOC rule applies; this is not a single match-on-enum or asset registry. | Split into `create_lobby.rs` (host-side create + discard), `join_lobby.rs` (guest-side join + accept), and keep `lobby_systems.rs` for the chat/presence drains. |
| S5 | ErrorObservability | `multiplayer/sockets.rs:36–38` | Low | S | In `tear_down_socket`, a poisoned-Mutex branch (`if let Ok(...) = socket.listener.lock()`) silently skips taking the listener. A Mutex poison only occurs if a previous lock-holder panicked — unlikely but not impossible in tests or future refactors. | Add an `else` arm: `} else { warn!("[Steam MP] ListenSocket Mutex poisoned — listener not dropped"); }`. Same pattern at line 57. |
| S6 | TypeContract | `multiplayer/lobby_state.rs:30–34` | Low | S | The `peer` field in `SteamLobbyState::Joined` is suppressed with `#[allow(dead_code)]`. The doc comment says "future code can render 'Connected to \<friend name\>'" but `sync_coop_peer_name` in `lobby_systems.rs` already reads `peer` via `client.friends().get_friend(peer).name()`. The `allow(dead_code)` is now stale. | Remove `#[allow(dead_code)]`. The field is live; the attribute is doc drift. |
| S7 | TypeContract | `leaderboards/systems.rs:159` | Low | S | `stats.total_time as i32` is cast without a guard when submitting to the clear-time leaderboard. `total_time` is `f32` (sum of per-level elapsed seconds). A very long run (>2.1 × 10⁹ seconds) would overflow, but more practically a run of >2,147,483 seconds (~25 days) would give a negative rank. The base-score path at line 130 correctly applies `.min(9_999.0)` for the penalty term but that different bound doesn't apply here. | Cap with `.min(i32::MAX as f32) as i32` or document the implicit upper bound. |
| S8 | DocDrift | `multiplayer/sockets.rs:238–239` | Low | S | The comment on `steam_transport_bridge_system` says `NetworkingMessage::send_flags()` "doesn't reliably distinguish unreliable variants on the receive side (see `networking_types.rs:1879`)". That line reference points into the upstream `bevy_steamworks` crate source which may change with any crate version bump, making the reference immediately stale. | Replace the line-number reference with a prose explanation of the invariant ("the Steam API merges all send-flag variants into a single flag word on receive, so we prefix our own tag byte instead"). |

---

### Oversized files

| File | LOC | Exempt | Reason / Split proposal |
|------|-----|--------|--------------------------|
| `multiplayer/lobby_systems.rs` | 404 | No | Split into: `create_lobby.rs` (host-side create + discard_stale), `join_lobby.rs` (guest accept + accept_incoming_join), `lobby_systems.rs` (chat updates + rich presence + sync_coop_peer_name) |
| `multiplayer/sockets.rs` | 332 | Yes | Single concern: all functions are tightly coupled to `SteamP2pSocket` lifecycle (open, close, send, receive). Extracting send/recv into a separate file would add indirection with no cohesion gain. |

---

### Looks bad but is actually fine

- **`.unwrap()` at `leaderboards/systems.rs:283,287` and `join_requests.rs:111`** — both are inside `#[cfg(test)]` test bodies. The scope note flags these as the "only `.unwrap()`s in the codebase" but they are test assertions, not production paths. This is the correct pattern.
- **`Option<Res<Client>>` everywhere in `lobby_systems.rs`** — looks like defensive paranoia but is load-bearing: `SteamMultiplayerPlugin` is registered inside the `Ok` arm of `SteamPlugin`, yet Bevy's scheduler evaluates system parameters at run time, not plugin-build time. If Steam drops mid-session the resource can disappear. `Option<Res<>>` is the correct Bevy idiom here.
- **`crossbeam_channel::unbounded()` in `LeaderboardHandles` and `SteamLobbyBridge`** — unbounded channels could theoretically grow without bound, but the consumers drain on every frame and the producers fire only on user-initiated Steam callbacks (not per-frame), so this is fine in practice.
- **`Mutex<Option<ListenSocket>>` in `SteamP2pSocket`** — wrapping a `!Sync` type in `Mutex` to satisfy Bevy's `Resource: Send + Sync` bound is the correct pattern when `unsafe impl Sync` is not warranted. The all-single-threaded access pattern means the Mutex is never contended at runtime.
- **`discard_stale_lobby` at `lobby_systems.rs:383`** — calling `leave_lobby` on a lobby we just created (after Cancel) and also `clear_rich_presence` looks aggressive, but both are idempotent and necessary to avoid ghost lobbies visible in the Steam friends list.
- **`sync_coop_peer_name` is not guarded by in-game state** — it runs whenever `SteamLobbyState` changes, which only happens from Bevy's change detection. Since `SteamLobbyState` is only mutated by lobby systems (not every frame), this is a correct and cheap use of `resource_changed`.

---

### Open questions

1. `VIRTUAL_PORT = 0` in `multiplayer/constants.rs` — Steam allows any non-negative i32, but port 0 is the documented default. If the game ever needs two simultaneous P2P sessions (e.g., spectator mode), both would clash on port 0. Is a reserved second port documented anywhere?
2. `prewarm_leaderboard_handles` is called `OnEnter(AppState::MainMenu)` but `handles_incomplete` guards the drain system until all 5 handles are resolved. If the player launches directly into a match (via `+connect_lobby`), the prewarming never fires and `submit_run_scores_to_steam` will log "handle not ready" silently. Is that acceptable?
3. `build_roguelite_details` returns a `Vec<i32>` with 9 entries. Steam leaderboard details are capped at 64 `i32`s, so there's headroom, but there's no assertion or comment documenting the cap. Worth adding a `const` assertion before the vector grows.
4. The `#[allow(dead_code)]` on `SteamLobbyState::Joined::peer` (finding S6) — was this field read by an old system that got replaced by `sync_coop_peer_name`? If so, the attribute is a leftover; if the field is truly unused, remove it entirely rather than suppressing the lint.
