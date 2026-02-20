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
            _closures: Vec::new(),
        }
    }
}

thread_local! {
    static WEBRTC_STATE: RefCell<WebRtcCallbackState> = RefCell::new(WebRtcCallbackState::default());
}

/// Creates an `RtcPeerConnection` with STUN server configuration.
fn create_peer_connection() -> Result<RtcPeerConnection, JsValue> {
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

    // Monitor ICE connection state changes
    let pc_for_ice = pc.clone();
    let on_ice_state = Closure::wrap(Box::new(move |_: JsValue| {
        let state = Reflect::get(&pc_for_ice, &JsValue::from_str("iceConnectionState"))
            .unwrap_or_default()
            .as_string()
            .unwrap_or_default();
        info!("[WebRTC] ICE connection state: {}", state);
    }) as Box<dyn FnMut(JsValue)>);
    pc.set_oniceconnectionstatechange(Some(on_ice_state.as_ref().unchecked_ref()));
    // Leak the closure to keep it alive (it's tied to the peer connection lifetime)
    on_ice_state.forget();

    // Monitor overall connection state changes
    let pc_for_conn = pc.clone();
    let on_conn_state = Closure::wrap(Box::new(move |_: JsValue| {
        let state = Reflect::get(&pc_for_conn, &JsValue::from_str("connectionState"))
            .unwrap_or_default()
            .as_string()
            .unwrap_or_default();
        info!("[WebRTC] Connection state: {}", state);
    }) as Box<dyn FnMut(JsValue)>);
    Reflect::set(
        &pc,
        &JsValue::from_str("onconnectionstatechange"),
        on_conn_state.as_ref(),
    ).ok();
    on_conn_state.forget();

    Ok(pc)
}

/// Sets up the `onicecandidate` callback that detects when ICE gathering is complete.
///
/// When gathering completes, the full local description (with all ICE candidates bundled)
/// is base64-encoded and stored as `local_code` for the user to copy.
fn setup_ice_candidate_handler(pc: &RtcPeerConnection) -> Closure<dyn FnMut(JsValue)> {
    let pc_clone = pc.clone();
    let closure = Closure::wrap(Box::new(move |event: JsValue| {
        let event: RtcPeerConnectionIceEvent = event.unchecked_into();
        if event.candidate().is_some() {
            info!("[WebRTC] ICE candidate gathered");
        }
        // When candidate is null, ICE gathering is complete
        if event.candidate().is_none() {
            info!("[WebRTC] ICE gathering complete");
            // Use Reflect to get localDescription properties (avoids feature-gating issues)
            let desc = Reflect::get(&pc_clone, &JsValue::from_str("localDescription"))
                .unwrap_or_default();
            if !desc.is_null() && !desc.is_undefined() {
                let sdp = Reflect::get(&desc, &JsValue::from_str("sdp"))
                    .unwrap_or_default()
                    .as_string()
                    .unwrap_or_default();
                let sdp_type = Reflect::get(&desc, &JsValue::from_str("type"))
                    .unwrap_or_default()
                    .as_string()
                    .unwrap_or_default();
                let payload = format!("{}|{}", sdp_type, sdp);
                let encoded = base64_encode(&payload);
                info!("[WebRTC] Local SDP ready (type={}), encoding...", sdp_type);
                WEBRTC_STATE.with(|s| {
                    let mut state = s.borrow_mut();
                    state.local_code = Some(encoded);
                    state.state = ConnectionState::WaitingForSignaling;
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
        info!("[WebRTC] Reliable data channel OPENED");
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
                    info!("[WebRTC] Received reliable message: {:?}", msg);
                    WEBRTC_STATE.with(|s| {
                        s.borrow_mut().incoming_messages.push(msg);
                    });
                }
                Err(e) => {
                    warn!("[WebRTC] Failed to parse message: {} (raw: {})", e, &text[..text.len().min(100)]);
                }
            }
        } else {
            warn!("[WebRTC] Received non-string data on reliable channel");
        }
    }) as Box<dyn FnMut(JsValue)>);
    dc.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    closures.push(on_message);

    // onclose
    let on_close = Closure::wrap(Box::new(move |_: JsValue| {
        warn!("[WebRTC] Reliable data channel CLOSED");
        WEBRTC_STATE.with(|s| {
            s.borrow_mut().state = ConnectionState::Disconnected;
        });
    }) as Box<dyn FnMut(JsValue)>);
    dc.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    closures.push(on_close);

    // onerror
    let on_error = Closure::wrap(Box::new(move |event: JsValue| {
        let error_info = format!("{:?}", event);
        error!("[WebRTC] Reliable data channel ERROR: {}", &error_info[..error_info.len().min(200)]);
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

    // onopen - just log
    let on_open = Closure::wrap(Box::new(move |_: JsValue| {
        info!("[WebRTC] Unreliable data channel OPENED");
    }) as Box<dyn FnMut(JsValue)>);
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
    let on_close = Closure::wrap(Box::new(move |_: JsValue| {
        warn!("[WebRTC] Unreliable data channel CLOSED");
    }) as Box<dyn FnMut(JsValue)>);
    dc.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    closures.push(on_close);

    // onerror
    let on_error = Closure::wrap(Box::new(move |event: JsValue| {
        let error_info = format!("{:?}", event);
        error!("[WebRTC] Unreliable data channel ERROR: {}", &error_info[..error_info.len().min(200)]);
    }) as Box<dyn FnMut(JsValue)>);
    dc.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    closures.push(on_error);

    closures
}

/// Initiates the host flow: creates an offer and waits for ICE gathering.
///
/// Called from the UI when the user clicks "Host Game".
pub fn create_host_offer() {
    info!("[WebRTC] Creating host offer...");
    WEBRTC_STATE.with(|s| {
        let mut state = s.borrow_mut();
        *state = WebRtcCallbackState::default();
        state.state = ConnectionState::WaitingForSignaling;
    });

    wasm_bindgen_futures::spawn_local(async {
        let result = async_create_host_offer().await;
        if let Err(e) = result {
            let msg = format!("{:?}", e);
            error!("Failed to create host offer: {}", msg);
            WEBRTC_STATE.with(|s| {
                let mut state = s.borrow_mut();
                state.state = ConnectionState::Failed;
                state.error = Some(msg);
            });
        }
    });
}

async fn async_create_host_offer() -> Result<(), JsValue> {
    info!("[WebRTC] Creating peer connection for host...");
    let pc = create_peer_connection()?;
    info!("[WebRTC] Peer connection created, creating data channels...");

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
    info!("[WebRTC] Creating SDP offer...");
    let offer = JsFuture::from(pc.create_offer()).await?;
    let offer_sdp: RtcSessionDescriptionInit = offer.unchecked_into();

    // Set local description
    info!("[WebRTC] Setting local description...");
    JsFuture::from(pc.set_local_description(&offer_sdp)).await?;
    info!("[WebRTC] Local description set, waiting for ICE gathering...");

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
pub fn create_guest_answer(host_code: &str) {
    info!("[WebRTC] Creating guest answer (code length: {})...", host_code.len());
    let host_code = host_code.to_string();

    WEBRTC_STATE.with(|s| {
        let mut state = s.borrow_mut();
        *state = WebRtcCallbackState::default();
        state.state = ConnectionState::WaitingForSignaling;
    });

    wasm_bindgen_futures::spawn_local(async move {
        let result = async_create_guest_answer(&host_code).await;
        if let Err(e) = result {
            let msg = format!("{:?}", e);
            error!("Failed to create guest answer: {}", msg);
            WEBRTC_STATE.with(|s| {
                let mut state = s.borrow_mut();
                state.state = ConnectionState::Failed;
                state.error = Some(msg);
            });
        }
    });
}

async fn async_create_guest_answer(host_code: &str) -> Result<(), JsValue> {
    // Decode the host's offer
    info!("[WebRTC] Decoding host offer...");
    let decoded = base64_decode(host_code)
        .map_err(|e| JsValue::from_str(&format!("Invalid code: {}", e)))?;

    let (sdp_type, sdp) = decoded
        .split_once('|')
        .ok_or_else(|| JsValue::from_str("Invalid code format"))?;
    info!("[WebRTC] Decoded SDP type: {}", sdp_type);

    if sdp_type != "offer" {
        return Err(JsValue::from_str("Expected an offer code, got an answer"));
    }

    info!("[WebRTC] Creating peer connection for guest...");
    let pc = create_peer_connection()?;

    // Set up handler for incoming data channels from host (reliable + unreliable)
    let on_datachannel = Closure::wrap(Box::new(move |event: JsValue| {
        let event: RtcDataChannelEvent = event.unchecked_into();
        let dc = event.channel();
        let label = dc.label();
        info!("[WebRTC] Guest received data channel: '{}'", label);
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

    // Set remote description (host's offer)
    info!("[WebRTC] Setting remote description (host's offer)...");
    let offer_init = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
    offer_init.set_sdp(sdp);
    JsFuture::from(pc.set_remote_description(&offer_init)).await?;
    info!("[WebRTC] Remote description set, creating answer...");

    // Create answer
    let answer = JsFuture::from(pc.create_answer()).await?;
    let answer_sdp: RtcSessionDescriptionInit = answer.unchecked_into();

    // Set local description
    info!("[WebRTC] Setting local description (answer)...");
    JsFuture::from(pc.set_local_description(&answer_sdp)).await?;
    info!("[WebRTC] Local description set, waiting for ICE gathering...");

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
    info!("[WebRTC] Processing guest answer (code length: {})...", guest_code.len());
    let guest_code = guest_code.to_string();

    WEBRTC_STATE.with(|s| {
        s.borrow_mut().state = ConnectionState::Connecting;
    });

    wasm_bindgen_futures::spawn_local(async move {
        let result = async_process_answer(&guest_code).await;
        if let Err(e) = result {
            let msg = format!("{:?}", e);
            error!("Failed to process answer: {}", msg);
            WEBRTC_STATE.with(|s| {
                let mut state = s.borrow_mut();
                state.state = ConnectionState::Failed;
                state.error = Some(msg);
            });
        }
    });
}

async fn async_process_answer(guest_code: &str) -> Result<(), JsValue> {
    info!("[WebRTC] Decoding guest answer...");
    let decoded = base64_decode(guest_code)
        .map_err(|e| JsValue::from_str(&format!("Invalid code: {}", e)))?;

    let (sdp_type, sdp) = decoded
        .split_once('|')
        .ok_or_else(|| JsValue::from_str("Invalid code format"))?;
    info!("[WebRTC] Decoded guest SDP type: {}", sdp_type);

    if sdp_type != "answer" {
        return Err(JsValue::from_str("Expected an answer code, got an offer"));
    }

    WEBRTC_STATE.with(|s| {
        let state = s.borrow();
        if let Some(pc) = &state.peer_connection {
            info!("[WebRTC] Setting remote description (guest's answer)...");
            let answer_init = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
            answer_init.set_sdp(sdp);
            wasm_bindgen_futures::spawn_local({
                let pc = pc.clone();
                async move {
                    if let Err(e) = JsFuture::from(pc.set_remote_description(&answer_init)).await {
                        error!("[WebRTC] Failed to set remote description: {:?}", e);
                        WEBRTC_STATE.with(|s| {
                            let mut state = s.borrow_mut();
                            state.state = ConnectionState::Failed;
                            state.error = Some(format!("{:?}", e));
                        });
                    } else {
                        info!("[WebRTC] Remote description set successfully, waiting for data channel...");
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
            if let Err(e) = dc.send_with_str(text) {
                warn!("Failed to send message: {:?}", e);
            }
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
            if let Err(e) = dc.send_with_u8_array(data) {
                warn!("Failed to send unreliable data: {:?}", e);
            }
        }
    });
}

/// Resets the WebRTC state, closing any active connections.
pub fn disconnect() {
    WEBRTC_STATE.with(|s| {
        let mut state = s.borrow_mut();
        if let Some(dc) = state.data_channel.take() {
            dc.close();
        }
        if let Some(dc) = state.unreliable_data_channel.take() {
            dc.close();
        }
        if let Some(pc) = state.peer_connection.take() {
            pc.close();
        }
        *state = WebRtcCallbackState::default();
    });
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
            info!("[WebRTC Sync] State changed: {:?} -> {:?}", connection.state, state.state);
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

// Simple base64 encode/decode using js_sys (avoids adding another dependency)

fn base64_encode(input: &str) -> String {
    let window = web_sys::window().expect("no window");
    window
        .btoa(input)
        .unwrap_or_else(|_| String::from("encoding_error"))
}

fn base64_decode(input: &str) -> Result<String, String> {
    let window = web_sys::window().expect("no window");
    window
        .atob(input)
        .map_err(|_| "Failed to decode base64".to_string())
}
