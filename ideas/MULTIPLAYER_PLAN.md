# Serverless P2P Multiplayer via WebRTC (Phase 1 -- Connection Only)

## Context

Adding multiplayer to Court Wizard using WebRTC with copy-paste SDP signaling codes. Zero server infrastructure -- the human is the signaling channel. Phase 1 establishes the P2P connection and verifies it with ping/pong messages. Game synchronization comes in a later phase.

**Flow:** Player A clicks "Host" -> gets a code (~500-1000 chars) -> sends it to Player B via Discord/text -> Player B clicks "Join", pastes it -> gets a response code -> sends it back -> Player A pastes it -> P2P connected.

The only external dependency is Google's free public STUN server (`stun:stun.l.google.com:19302`) for NAT traversal discovery. It's stateless, handles zero game data, and is used only during connection setup.

---

## Step 1: Dependencies (`Cargo.toml`)

Add `wasm-bindgen-futures` and expand `web-sys` features for WebRTC:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"                    # NEW - for spawn_local
web-sys = { version = "0.3", features = [
    "Window", "Storage",
    # WebRTC features (NEW)
    "MessageEvent",
    "RtcPeerConnection", "RtcConfiguration", "RtcIceServer",
    "RtcSignalingState", "RtcIceGatheringState", "RtcIceConnectionState",
    "RtcSdpType", "RtcSessionDescriptionInit", "RtcPeerConnectionIceEvent",
    "RtcIceCandidate", "RtcDataChannel", "RtcDataChannelEvent",
    "RtcDataChannelInit", "RtcDataChannelState",
] }
js-sys = "0.3"
getrandom = { version = "0.2", features = ["js"] }
```

---

## Step 2: Networking Module (`src/networking/`)

Create a new top-level module. Add `mod networking;` to `src/main.rs` and register `NetworkingPlugin` in the app builder.

### File structure:
```
src/networking/
    mod.rs              -- Module root, re-exports NetworkingPlugin
    plugin.rs           -- Bevy plugin registration
    resources.rs        -- NetworkConnection resource, PeerRole, ConnectionState
    messages.rs         -- Bevy Messages (ConnectionEstablished, etc.)
    protocol.rs         -- Serializable NetworkMessage enum (Ping/Pong/Text)
    constants.rs        -- STUN URL, channel name, timeouts
    webrtc.rs           -- #[cfg(wasm32)] WebRTC implementation
    clipboard.rs        -- #[cfg(wasm32)] JS interop for copy/prompt
```

### Key resource: `NetworkConnection`
```rust
#[derive(Resource, Default)]
pub struct NetworkConnection {
    pub state: ConnectionState,        // Disconnected/WaitingForSignaling/Connecting/Connected/Failed
    pub role: Option<PeerRole>,        // Host or Guest
    pub local_code: Option<String>,    // Base64 SDP+ICE code to display to user
    pub incoming_messages: Vec<NetworkMessage>,
    pub outgoing_messages: Vec<NetworkMessage>,
    pub ping_ms: Option<f32>,
}
```

### WebRTC bridge pattern

`RtcPeerConnection` and `RtcDataChannel` are JS objects (not `Send+Sync`). Use `thread_local!` storage:
- JS callbacks write to `thread_local! { static WEBRTC_STATE: RefCell<WebRtcCallbackState> }`
- A Bevy system (`sync_webrtc_state`) copies data into `NetworkConnection` each frame
- This cleanly separates async JS callback world from synchronous Bevy ECS

### WebRTC flow:

**Host:**
1. Create `RtcPeerConnection` with STUN config
2. Create data channel `"game"`
3. `create_offer()` -> `set_local_description(offer)`
4. Wait for ICE gathering complete (all candidates bundled into SDP)
5. Base64-encode local description -> store as `local_code`
6. User copies code, sends to guest, guest sends back response code
7. Base64-decode response -> `set_remote_description(answer)`
8. Data channel opens -> `Connected`

**Guest:**
1. Base64-decode host's offer code
2. Create `RtcPeerConnection` with STUN config
3. Set `ondatachannel` callback to capture incoming data channel
4. `set_remote_description(offer)` -> `create_answer()` -> `set_local_description(answer)`
5. Wait for ICE gathering complete
6. Base64-encode local description -> store as `local_code`
7. User copies response code, sends back to host
8. Data channel opens -> `Connected`

### Clipboard JS interop (`clipboard.rs`)

Use `wasm_bindgen(inline_js)` for clipboard copy (avoids `web_sys_unstable_apis`):
```rust
#[wasm_bindgen(inline_js = "export function copy_to_clipboard(text) { navigator.clipboard.writeText(text).catch(function() { ... fallback ... }); }")]
extern "C" { pub fn copy_to_clipboard(text: &str); }
```

For paste direction: use `window.prompt()` -- simple, handles paste natively, works everywhere. Good enough for Phase 1.

---

## Step 3: State Changes (`src/state/states.rs`)

Add `Multiplayer` variant to `MenuState`:
```rust
pub enum MenuState {
    // ... existing variants ...
    Multiplayer,    // NEW
}
```

---

## Step 4: Multiplayer UI (`src/ui/main_menu/multiplayer/`)

Follow the exact landing screen pattern (`mod.rs` + `plugin.rs` + `systems.rs` + `components.rs` + `constants.rs`).

### Plugin registration:
- `OnEnter(MenuState::Multiplayer)` -> `setup`
- `OnExit(MenuState::Multiplayer)` -> `cleanup`
- `Update` with `run_if(in_state(MenuState::Multiplayer))` -> `button_action`, `update_ui_state`

### UI flow (single screen, content reacts to `NetworkConnection.state`):

**Initial:** Title "Multiplayer", buttons "Host Game" / "Join Game" / "Back"

**Host flow:**
1. Click "Host Game" -> calls `create_host_offer()`, shows "Generating code..."
2. `local_code` populated -> shows code text + "Copy Code" button
3. "Paste Response" button -> calls `prompt_for_text()` -> calls `process_answer()`
4. "Connecting..." -> "Connected! Ping: Xms"

**Join flow:**
1. Click "Join Game" -> calls `prompt_for_text()` for invite code -> calls `create_guest_answer()`
2. Shows "Generating response..." 
3. `local_code` populated -> shows response code + "Copy Code" button
4. "Connecting..." -> "Connected! Ping: Xms"

---

## Step 5: Landing Screen Integration

**`src/ui/main_menu/landing/components.rs`:** Add `Multiplayer` to `MenuButtonAction`

**`src/ui/main_menu/landing/systems.rs`:**
- Add `spawn_button(parent, "Multiplayer", MenuButtonAction::Multiplayer, &BUTTON_STYLE)` in `setup()`
- Add `MenuButtonAction::Multiplayer => next_menu_state.set(MenuState::Multiplayer)` in `button_action()`

**`src/ui/main_menu/mod.rs`:** Add `mod multiplayer;`

**`src/ui/main_menu/plugin.rs`:** Add `MultiplayerPlugin` to the plugin tuple

---

## Step 6: Ping/Pong Verification

- `send_ping` system: sends `NetworkMessage::Ping { timestamp_ms }` every 2 seconds when `Connected`
- `handle_pong` system: calculates RTT from `Pong.timestamp_ms`, updates `NetworkConnection.ping_ms`
- Multiplayer UI displays ping in status area

---

## Files Summary

### Create:
| File | Purpose |
|------|---------|
| `src/networking/mod.rs` | Module root |
| `src/networking/plugin.rs` | `NetworkingPlugin` |
| `src/networking/resources.rs` | `NetworkConnection`, `ConnectionState`, `PeerRole` |
| `src/networking/messages.rs` | Bevy Messages |
| `src/networking/protocol.rs` | `NetworkMessage` enum (serde) |
| `src/networking/constants.rs` | STUN URL, timeouts |
| `src/networking/webrtc.rs` | `#[cfg(wasm32)]` WebRTC core |
| `src/networking/clipboard.rs` | `#[cfg(wasm32)]` JS clipboard/prompt interop |
| `src/ui/main_menu/multiplayer/mod.rs` | Module root |
| `src/ui/main_menu/multiplayer/plugin.rs` | `MultiplayerPlugin` |
| `src/ui/main_menu/multiplayer/systems.rs` | setup, cleanup, button_action, update_ui_state |
| `src/ui/main_menu/multiplayer/components.rs` | Screen markers, button actions |
| `src/ui/main_menu/multiplayer/constants.rs` | Styles (reuse landing constants) |

### Modify:
| File | Change |
|------|--------|
| `Cargo.toml` | Add web-sys WebRTC features + `wasm-bindgen-futures` |
| `src/main.rs` | Add `mod networking;`, register `NetworkingPlugin` |
| `src/state/states.rs` | Add `Multiplayer` to `MenuState` |
| `src/ui/main_menu/mod.rs` | Add `mod multiplayer;` |
| `src/ui/main_menu/plugin.rs` | Add `MultiplayerPlugin` |
| `src/ui/main_menu/landing/components.rs` | Add `Multiplayer` to `MenuButtonAction` |
| `src/ui/main_menu/landing/systems.rs` | Add "Multiplayer" button + handle action |

---

## Verification

1. `./build_wasm.sh` -- compiles successfully
2. Open game in browser -- "Multiplayer" button visible on landing screen
3. Click "Multiplayer" -> Host/Join screen appears, "Back" returns to landing
4. Open two browser tabs, one hosts, one joins
5. Host clicks "Host Game" -> code appears -> copy it
6. Guest clicks "Join Game" -> paste host's code -> response code appears -> copy it
7. Host clicks "Paste Response" -> paste guest's code -> both show "Connected!"
8. Ping display updates with round-trip time
9. `cargo test` -- no regressions (networking is WASM-only, native builds skip it)
