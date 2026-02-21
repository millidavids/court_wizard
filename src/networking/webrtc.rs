//! WebRTC implementation for WASM targets.
//!
//! Uses thread-local storage to bridge between async JS callbacks and
//! synchronous Bevy ECS systems. JS callbacks write to thread-local state,
//! and `sync_webrtc_state` copies it into the `NetworkConnection` resource each frame.

use std::cell::RefCell;

use bevy::prelude::*;
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    MessageEvent, RtcDataChannel, RtcDataChannelEvent, RtcPeerConnection,
    RtcPeerConnectionIceEvent, RtcSdpType, RtcSessionDescriptionInit,
};

use super::constants::{DATA_CHANNEL_NAME, STUN_URL, UNRELIABLE_CHANNEL_NAME};
use super::protocol::NetworkMessage;
use super::resources::{ConnectionState, NetworkConnection};

use js_sys::Uint8Array;

/// Callback state shared between JS callbacks and the Bevy sync system.
struct WebRtcCallbackState {
    /// The peer connection (kept alive for the duration of the session).
    peer_connection: Option<RtcPeerConnection>,

    /// The reliable data channel for sending/receiving messages.
    data_channel: Option<RtcDataChannel>,

    /// The unreliable data channel for state snapshots (UDP-like).
    unreliable_data_channel: Option<RtcDataChannel>,

    /// Base64-encoded local SDP description ready for the user to copy.
    local_code: Option<String>,

    /// Messages received from the remote peer on the reliable channel.
    incoming_messages: Vec<NetworkMessage>,

    /// Binary data received on the unreliable channel.
    incoming_unreliable: Vec<Vec<u8>>,

    /// Current connection state as determined by callbacks.
    state: ConnectionState,

    /// Error message if something went wrong.
    error: Option<String>,

    /// User-provided local IP address for LAN mode (injected as host candidate).
    local_ip: Option<String>,

    /// Closures that must be kept alive for the duration of the connection.
    _closures: Vec<Closure<dyn FnMut(JsValue)>>,
}

impl Default for WebRtcCallbackState {
    fn default() -> Self {
        Self {
            peer_connection: None,
            data_channel: None,
            unreliable_data_channel: None,
            local_code: None,
            incoming_messages: Vec::new(),
            incoming_unreliable: Vec::new(),
            state: ConnectionState::Disconnected,
            error: None,
            local_ip: None,
            _closures: Vec::new(),
        }
    }
}

thread_local! {
    static WEBRTC_STATE: RefCell<WebRtcCallbackState> = RefCell::new(WebRtcCallbackState::default());
}

/// Creates an `RtcPeerConnection` with STUN server configuration.
///
/// Always uses Google's public STUN server. This is needed even for LAN connections
/// because modern browsers hide local IPs behind mDNS hostnames — STUN forces the
/// browser to also generate candidates with real IP addresses.
fn create_peer_connection() -> Result<RtcPeerConnection, JsValue> {
    info!("[WebRTC] Creating peer connection");
    let ice_server = Object::new();
    let urls = Array::new();
    urls.push(&JsValue::from_str(STUN_URL));
    Reflect::set(&ice_server, &JsValue::from_str("urls"), &urls)?;

    let ice_servers = Array::new();
    ice_servers.push(&ice_server);

    let config = Object::new();
    Reflect::set(
        &config,
        &JsValue::from_str("iceServers"),
        &ice_servers,
    )?;

    let rtc_config: web_sys::RtcConfiguration = config.unchecked_into();
    let pc = RtcPeerConnection::new_with_configuration(&rtc_config)?;

    // Monitor ICE connection state changes — transition to Failed on ICE failure
    let pc_for_ice = pc.clone();
    let on_ice_state = Closure::wrap(Box::new(move |_: JsValue| {
        let state = Reflect::get(&pc_for_ice, &JsValue::from_str("iceConnectionState"))
            .unwrap_or_default()
            .as_string()
            .unwrap_or_default();
        let gathering = Reflect::get(&pc_for_ice, &JsValue::from_str("iceGatheringState"))
            .unwrap_or_default()
            .as_string()
            .unwrap_or_default();
        info!(
            "[WebRTC] ICE connection state: '{}', gathering: '{}'",
            state, gathering
        );
        match state.as_str() {
            "failed" => {
                WEBRTC_STATE.with(|s| {
                    let mut ws = s.borrow_mut();
                    ws.state = ConnectionState::Failed;
                    ws.error = Some("ICE connection failed — could not reach peer".to_string());
                });
            }
            "disconnected" => {
                // "disconnected" is transient during initial connection — the browser
                // will keep trying candidates and eventually reach "failed" if it can't connect.
                // Only treat as failure if we had an established connection that was lost.
                WEBRTC_STATE.with(|s| {
                    let ws = s.borrow();
                    let current = ws.state;
                    drop(ws);
                    if current == ConnectionState::Connected {
                        let mut ws = s.borrow_mut();
                        ws.state = ConnectionState::Failed;
                        ws.error = Some("Connection lost".to_string());
                    }
                });
            }
            "checking" => {
                // ICE is actively checking candidates — mark as Connecting,
                // but only if the local code is already set (user had a chance to copy it).
                // The guest generates its answer code *after* ICE checking starts,
                // so we must not hide the signaling UI prematurely.
                WEBRTC_STATE.with(|s| {
                    let mut ws = s.borrow_mut();
                    if ws.state == ConnectionState::WaitingForSignaling
                        && ws.local_code.is_some()
                    {
                        ws.state = ConnectionState::Connecting;
                    }
                });
            }
            _ => {}
        }
    }) as Box<dyn FnMut(JsValue)>);
    pc.set_oniceconnectionstatechange(Some(on_ice_state.as_ref().unchecked_ref()));
    // Leak the closure to keep it alive (it's tied to the peer connection lifetime)
    on_ice_state.forget();

    Ok(pc)
}

/// Sets up the `onicecandidate` callback that detects when ICE gathering is complete.
///
/// When gathering completes, the local description is encoded into a compact
/// connection code and stored as `local_code` for the user to copy.
/// If a `local_ip` was provided (LAN mode), it is injected as an additional
/// host candidate line in the SDP before encoding.
fn setup_ice_candidate_handler(pc: &RtcPeerConnection) -> Closure<dyn FnMut(JsValue)> {
    let pc_clone = pc.clone();
    let closure = Closure::wrap(Box::new(move |event: JsValue| {
        let event: RtcPeerConnectionIceEvent = event.unchecked_into();
        // Log each candidate
        if let Some(candidate) = event.candidate() {
            let candidate_str = candidate.candidate();
            info!("[WebRTC] ICE candidate: {}", candidate_str);
        }
        // When candidate is null, ICE gathering is complete
        if event.candidate().is_none() {
            let desc = Reflect::get(&pc_clone, &JsValue::from_str("localDescription"))
                .unwrap_or_default();
            if !desc.is_null() && !desc.is_undefined() {
                let mut sdp = Reflect::get(&desc, &JsValue::from_str("sdp"))
                    .unwrap_or_default()
                    .as_string()
                    .unwrap_or_default();
                let sdp_type = Reflect::get(&desc, &JsValue::from_str("type"))
                    .unwrap_or_default()
                    .as_string()
                    .unwrap_or_default();

                // If a local IP was provided, inject it as a host candidate.
                // Reuse the port from the first existing host candidate so ICE
                // can reach the same local DTLS/SCTP listener.
                WEBRTC_STATE.with(|s| {
                    let state = s.borrow();
                    if let Some(local_ip) = &state.local_ip {
                        let port = sdp
                            .lines()
                            .filter(|l| l.contains("typ host"))
                            .find_map(|l| {
                                let parts: Vec<&str> = l.split_whitespace().collect();
                                if parts.len() >= 6 {
                                    parts[5].parse::<u16>().ok()
                                } else {
                                    None
                                }
                            });
                        if let Some(port) = port {
                            let candidate_line = format!(
                                "a=candidate:local 1 udp 2130706431 {} {} typ host\r\n",
                                local_ip, port
                            );
                            info!(
                                "[WebRTC] Injecting local IP candidate: {}",
                                candidate_line.trim()
                            );
                            sdp.push_str(&candidate_line);
                        } else {
                            warn!(
                                "[WebRTC] No host candidate port found to reuse for local IP injection"
                            );
                        }
                    }
                });

                let encoded = match super::connection_code::encode(&sdp, &sdp_type) {
                    Ok(code) => code,
                    Err(e) => {
                        warn!("Failed to encode connection code: {}", e);
                        return;
                    }
                };
                info!(
                    "[WebRTC] ICE gathering complete, code length: {} chars",
                    encoded.len()
                );
                WEBRTC_STATE.with(|s| {
                    let mut state = s.borrow_mut();
                    state.local_code = Some(encoded);
                    // Only set WaitingForSignaling if we haven't already progressed
                    // past it (e.g. ICE checking may have already set Connecting)
                    if state.state != ConnectionState::Connecting
                        && state.state != ConnectionState::Connected
                        && state.state != ConnectionState::Failed
                    {
                        state.state = ConnectionState::WaitingForSignaling;
                    }
                });
            }
        }
    }) as Box<dyn FnMut(JsValue)>);

    pc.set_onicecandidate(Some(closure.as_ref().unchecked_ref()));
    closure
}

/// Sets up the data channel event handlers (onopen, onmessage, onclose, onerror).
fn setup_data_channel_handlers(dc: &RtcDataChannel) -> Vec<Closure<dyn FnMut(JsValue)>> {
    let mut closures = Vec::new();

    // onopen
    let on_open = Closure::wrap(Box::new(move |_: JsValue| {
        info!("[WebRTC] Data channel opened — connected!");
        WEBRTC_STATE.with(|s| {
            s.borrow_mut().state = ConnectionState::Connected;
        });
    }) as Box<dyn FnMut(JsValue)>);
    dc.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    closures.push(on_open);

    // onmessage
    let on_message = Closure::wrap(Box::new(move |event: JsValue| {
        let event: MessageEvent = event.unchecked_into();
        if let Some(text) = event.data().as_string() {
            match serde_json::from_str::<NetworkMessage>(&text) {
                Ok(msg) => {
                    WEBRTC_STATE.with(|s| {
                        s.borrow_mut().incoming_messages.push(msg);
                    });
                }
                Err(e) => {
                    warn!("Failed to parse network message: {}", e);
                }
            }
        } else {
            warn!("Received non-string data on reliable channel");
        }
    }) as Box<dyn FnMut(JsValue)>);
    dc.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    closures.push(on_message);

    // onclose
    let on_close = Closure::wrap(Box::new(move |_: JsValue| {
        WEBRTC_STATE.with(|s| {
            s.borrow_mut().state = ConnectionState::Disconnected;
        });
    }) as Box<dyn FnMut(JsValue)>);
    dc.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    closures.push(on_close);

    // onerror
    let on_error = Closure::wrap(Box::new(move |_event: JsValue| {
        WEBRTC_STATE.with(|s| {
            let mut state = s.borrow_mut();
            state.state = ConnectionState::Failed;
            state.error = Some("Data channel error".to_string());
        });
    }) as Box<dyn FnMut(JsValue)>);
    dc.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    closures.push(on_error);

    closures
}

/// Sets up event handlers for the unreliable data channel (binary, UDP-like).
fn setup_unreliable_channel_handlers(dc: &RtcDataChannel) -> Vec<Closure<dyn FnMut(JsValue)>> {
    let mut closures = Vec::new();

    // Set binary type to arraybuffer for efficient binary transfer
    Reflect::set(
        dc.as_ref(),
        &JsValue::from_str("binaryType"),
        &JsValue::from_str("arraybuffer"),
    )
    .ok();

    // onopen
    let on_open = Closure::wrap(Box::new(move |_: JsValue| {}) as Box<dyn FnMut(JsValue)>);
    dc.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    closures.push(on_open);

    // onmessage - receive binary data
    let on_message = Closure::wrap(Box::new(move |event: JsValue| {
        let event: MessageEvent = event.unchecked_into();
        let data = event.data();
        // Data arrives as ArrayBuffer
        if let Ok(buffer) = data.dyn_into::<js_sys::ArrayBuffer>() {
            let array = Uint8Array::new(&buffer);
            let bytes = array.to_vec();
            WEBRTC_STATE.with(|s| {
                s.borrow_mut().incoming_unreliable.push(bytes);
            });
        }
    }) as Box<dyn FnMut(JsValue)>);
    dc.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    closures.push(on_message);

    // onclose
    let on_close = Closure::wrap(Box::new(move |_: JsValue| {}) as Box<dyn FnMut(JsValue)>);
    dc.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    closures.push(on_close);

    // onerror
    let on_error = Closure::wrap(Box::new(move |_: JsValue| {}) as Box<dyn FnMut(JsValue)>);
    dc.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    closures.push(on_error);

    closures
}

/// Initiates the host flow: creates an offer and waits for ICE gathering.
///
/// Called from the UI when the user clicks "Host Game" or "LAN Host".
/// For LAN mode, `local_ip` should be the user's local network IP (e.g., "192.168.1.5")
/// which will be injected as a host candidate alongside the browser's own candidates.
pub fn create_host_offer(local_ip: Option<&str>) {
    let local_ip = local_ip.map(|s| s.to_string());

    WEBRTC_STATE.with(|s| {
        let mut state = s.borrow_mut();
        *state = WebRtcCallbackState::default();
        state.state = ConnectionState::WaitingForSignaling;
        state.local_ip = local_ip;
    });

    wasm_bindgen_futures::spawn_local(async move {
        let result = async_create_host_offer().await;
        if let Err(e) = result {
            let msg = format!("{:?}", e);
            WEBRTC_STATE.with(|s| {
                let mut state = s.borrow_mut();
                state.state = ConnectionState::Failed;
                state.error = Some(msg);
            });
        }
    });
}

async fn async_create_host_offer() -> Result<(), JsValue> {
    info!("[WebRTC] Creating host offer...");
    let pc = create_peer_connection()?;

    // Create the reliable data channel before creating the offer
    let dc = pc.create_data_channel(DATA_CHANNEL_NAME);
    let dc_closures = setup_data_channel_handlers(&dc);

    // Create the unreliable data channel (UDP-like: unordered, no retransmits)
    let unreliable_init = web_sys::RtcDataChannelInit::new();
    unreliable_init.set_ordered(false);
    unreliable_init.set_max_retransmits(0);
    let unreliable_dc =
        pc.create_data_channel_with_data_channel_dict(UNRELIABLE_CHANNEL_NAME, &unreliable_init);
    let unreliable_closures = setup_unreliable_channel_handlers(&unreliable_dc);

    // Setup ICE candidate handler
    let ice_closure = setup_ice_candidate_handler(&pc);

    // Create offer
    let offer = JsFuture::from(pc.create_offer()).await?;
    let offer_sdp: RtcSessionDescriptionInit = offer.unchecked_into();

    // Set local description
    JsFuture::from(pc.set_local_description(&offer_sdp)).await?;

    // Store everything in thread-local state
    WEBRTC_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.peer_connection = Some(pc);
        state.data_channel = Some(dc);
        state.unreliable_data_channel = Some(unreliable_dc);
        state._closures.extend(dc_closures);
        state._closures.extend(unreliable_closures);
        state._closures.push(ice_closure);
    });

    Ok(())
}

/// Processes the host's offer code as a guest and creates an answer.
///
/// Called from the UI when the guest pastes the host's invite code.
/// For LAN mode, `local_ip` should be the user's local network IP (e.g., "192.168.1.5")
/// which will be injected as a host candidate alongside the browser's own candidates.
pub fn create_guest_answer(host_code: &str, local_ip: Option<&str>) {
    let host_code = host_code.to_string();
    let local_ip = local_ip.map(|s| s.to_string());

    WEBRTC_STATE.with(|s| {
        let mut state = s.borrow_mut();
        *state = WebRtcCallbackState::default();
        state.state = ConnectionState::WaitingForSignaling;
        state.local_ip = local_ip;
    });

    wasm_bindgen_futures::spawn_local(async move {
        let result = async_create_guest_answer(&host_code).await;
        if let Err(e) = result {
            let msg = format!("{:?}", e);
            WEBRTC_STATE.with(|s| {
                let mut state = s.borrow_mut();
                state.state = ConnectionState::Failed;
                state.error = Some(msg);
            });
        }
    });
}

async fn async_create_guest_answer(host_code: &str) -> Result<(), JsValue> {
    info!(
        "[WebRTC] Creating guest answer (code length: {})...",
        host_code.len()
    );
    // Decode and validate the host's offer
    let code = super::connection_code::decode(host_code)
        .map_err(|e| JsValue::from_str(&format!("Invalid code: {}", e)))?;

    if !code.is_offer {
        return Err(JsValue::from_str("Expected an offer code, got an answer"));
    }

    // Reconstruct SDP from validated fields (never pass raw remote SDP to browser)
    let (_, sdp) = code.to_sdp();

    // Log the candidates from the decoded offer
    for line in sdp.lines() {
        if line.starts_with("a=candidate:") {
            info!("[WebRTC] Host candidate from code: {}", line);
        }
    }

    let pc = create_peer_connection()?;

    // Set up handler for incoming data channels from host (reliable + unreliable)
    let on_datachannel = Closure::wrap(Box::new(move |event: JsValue| {
        let event: RtcDataChannelEvent = event.unchecked_into();
        let dc = event.channel();
        let label = dc.label();
        if label == UNRELIABLE_CHANNEL_NAME {
            let closures = setup_unreliable_channel_handlers(&dc);
            WEBRTC_STATE.with(|s| {
                let mut state = s.borrow_mut();
                state.unreliable_data_channel = Some(dc);
                state._closures.extend(closures);
            });
        } else {
            let closures = setup_data_channel_handlers(&dc);
            WEBRTC_STATE.with(|s| {
                let mut state = s.borrow_mut();
                state.data_channel = Some(dc);
                state._closures.extend(closures);
            });
        }
    }) as Box<dyn FnMut(JsValue)>);
    pc.set_ondatachannel(Some(on_datachannel.as_ref().unchecked_ref()));

    // Setup ICE candidate handler
    let ice_closure = setup_ice_candidate_handler(&pc);

    // Set remote description (reconstructed from template)
    let offer_init = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
    offer_init.set_sdp(&sdp);
    JsFuture::from(pc.set_remote_description(&offer_init)).await?;

    // Create answer
    let answer = JsFuture::from(pc.create_answer()).await?;
    let answer_sdp: RtcSessionDescriptionInit = answer.unchecked_into();

    // Set local description
    JsFuture::from(pc.set_local_description(&answer_sdp)).await?;

    WEBRTC_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.peer_connection = Some(pc);
        state._closures.push(on_datachannel);
        state._closures.push(ice_closure);
    });

    Ok(())
}

/// Processes the guest's answer code as the host.
///
/// Called from the UI when the host pastes the guest's response code.
pub fn process_answer(guest_code: &str) {
    let guest_code = guest_code.to_string();

    WEBRTC_STATE.with(|s| {
        s.borrow_mut().state = ConnectionState::Connecting;
    });

    wasm_bindgen_futures::spawn_local(async move {
        let result = async_process_answer(&guest_code).await;
        if let Err(e) = result {
            let msg = format!("{:?}", e);
            WEBRTC_STATE.with(|s| {
                let mut state = s.borrow_mut();
                state.state = ConnectionState::Failed;
                state.error = Some(msg);
            });
        }
    });
}

async fn async_process_answer(guest_code: &str) -> Result<(), JsValue> {
    info!(
        "[WebRTC] Processing guest answer (code length: {})...",
        guest_code.len()
    );
    // Decode and validate the guest's answer
    let code = super::connection_code::decode(guest_code)
        .map_err(|e| JsValue::from_str(&format!("Invalid code: {}", e)))?;

    if code.is_offer {
        return Err(JsValue::from_str("Expected an answer code, got an offer"));
    }

    // Reconstruct SDP from validated fields (never pass raw remote SDP to browser)
    let (_, sdp) = code.to_sdp();

    // Log the candidates from the decoded answer
    for line in sdp.lines() {
        if line.starts_with("a=candidate:") {
            info!("[WebRTC] Guest candidate from code: {}", line);
        }
    }

    WEBRTC_STATE.with(|s| {
        let state = s.borrow();
        if let Some(pc) = &state.peer_connection {
            let answer_init = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
            answer_init.set_sdp(&sdp);
            wasm_bindgen_futures::spawn_local({
                let pc = pc.clone();
                async move {
                    if let Err(e) = JsFuture::from(pc.set_remote_description(&answer_init)).await {
                        warn!("[WebRTC] Failed to set remote description: {:?}", e);
                        WEBRTC_STATE.with(|s| {
                            let mut state = s.borrow_mut();
                            state.state = ConnectionState::Failed;
                            state.error = Some(format!("{:?}", e));
                        });
                    } else {
                        info!("[WebRTC] Remote description set, now connecting...");
                        WEBRTC_STATE.with(|s| {
                            let mut state = s.borrow_mut();
                            if state.state != ConnectionState::Connected {
                                state.state = ConnectionState::Connecting;
                            }
                        });
                    }
                }
            });
        }
    });

    Ok(())
}

/// Sends a text message over the data channel.
///
/// Called by the plugin system to send serialized `NetworkMessage`s.
pub fn send_message(text: &str) {
    WEBRTC_STATE.with(|s| {
        let state = s.borrow();
        if let Some(dc) = &state.data_channel {
            let _ = dc.send_with_str(text);
        }
    });
}

/// Sends binary data over the unreliable data channel.
///
/// Used for state snapshots that benefit from UDP-like delivery.
pub fn send_unreliable(data: &[u8]) {
    WEBRTC_STATE.with(|s| {
        let state = s.borrow();
        if let Some(dc) = &state.unreliable_data_channel {
            let _ = dc.send_with_u8_array(data);
        }
    });
}

/// Resets the WebRTC state, closing any active connections.
///
/// Takes channels and peer connection out of state first, then resets state,
/// then clears handlers and closes — this avoids recursive `RefCell` borrows
/// when `close()` synchronously fires `onclose` callbacks that access state.
pub fn disconnect() {
    let (dc, udc, pc) = WEBRTC_STATE.with(|s| {
        let mut state = s.borrow_mut();
        let dc = state.data_channel.take();
        let udc = state.unreliable_data_channel.take();
        let pc = state.peer_connection.take();
        *state = WebRtcCallbackState::default();
        (dc, udc, pc)
    });

    // Clear handlers and close outside the borrow scope
    if let Some(dc) = dc {
        dc.set_onmessage(None);
        dc.set_onclose(None);
        dc.set_onerror(None);
        dc.set_onopen(None);
        dc.close();
    }
    if let Some(dc) = udc {
        dc.set_onmessage(None);
        dc.set_onclose(None);
        dc.set_onerror(None);
        dc.set_onopen(None);
        dc.close();
    }
    if let Some(pc) = pc {
        pc.set_onicecandidate(None);
        pc.set_ondatachannel(None);
        pc.set_oniceconnectionstatechange(None);
        pc.close();
    }
}

/// Bevy system that syncs thread-local WebRTC callback state into the `NetworkConnection` resource.
///
/// This runs every frame and bridges the async JS world with the synchronous ECS.
/// Only mutates the resource when values actually changed, to avoid triggering
/// `is_changed()` every frame (which would cause UI rebuilds that make buttons unclickable).
pub fn sync_webrtc_state(connection: ResMut<NetworkConnection>) {
    // First, check if anything actually needs to change (read-only peek at thread-local).
    let needs_update = WEBRTC_STATE.with(|s| {
        let state = s.borrow();
        let state_changed = connection.state != state.state;
        let code_ready = state.local_code.is_some() && connection.local_code.is_none();
        let has_messages = !state.incoming_messages.is_empty();
        let has_unreliable = !state.incoming_unreliable.is_empty();
        let has_error = state.error.is_some() && connection.error.is_none();
        state_changed || code_ready || has_messages || has_unreliable || has_error
    });

    if !needs_update {
        return;
    }

    // Now take mutable access - this marks is_changed(), but only when there's actual new data.
    let mut connection = connection;
    WEBRTC_STATE.with(|s| {
        let mut state = s.borrow_mut();

        if connection.state != state.state {
            connection.state = state.state;
        }

        if state.local_code.is_some() && connection.local_code.is_none() {
            connection.local_code = state.local_code.clone();
        }

        if !state.incoming_messages.is_empty() {
            connection
                .incoming_messages
                .append(&mut state.incoming_messages);
        }

        if !state.incoming_unreliable.is_empty() {
            connection
                .incoming_unreliable
                .append(&mut state.incoming_unreliable);
        }

        if state.error.is_some() && connection.error.is_none() {
            connection.error = state.error.clone();
        }
    });
}

