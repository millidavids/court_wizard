//! BroadcastChannel-based LAN signaling for same-machine auto-discovery.
//!
//! Uses the browser's `BroadcastChannel` API to automatically exchange WebRTC
//! signaling codes between tabs on the same machine. For different machines on
//! a LAN, the manual copy-paste fallback still works.
//!
//! Protocol: JSON messages with `{ "type": "offer"|"answer", "code": "..." }`.

use std::cell::RefCell;

use bevy::prelude::*;
use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;
use web_sys::BroadcastChannel;

use super::constants::LAN_BROADCAST_CHANNEL;

/// Thread-local state for the BroadcastChannel signaling.
struct LanSignalingState {
    /// The BroadcastChannel instance.
    channel: Option<BroadcastChannel>,

    /// Offer code received from a LAN host (guest reads this).
    received_offer: Option<String>,

    /// Answer code received from a LAN guest (host reads this).
    received_answer: Option<String>,

    /// Closure kept alive for the channel's onmessage handler.
    _closure: Option<Closure<dyn FnMut(JsValue)>>,
}

impl Default for LanSignalingState {
    fn default() -> Self {
        Self {
            channel: None,
            received_offer: None,
            received_answer: None,
            _closure: None,
        }
    }
}

thread_local! {
    static LAN_STATE: RefCell<LanSignalingState> = RefCell::new(LanSignalingState::default());
}

/// Opens a BroadcastChannel and sets up a message handler.
///
/// The handler parses incoming JSON messages and stores offer/answer codes
/// in the thread-local state for polling by the ECS system.
fn open_channel() -> (BroadcastChannel, Closure<dyn FnMut(JsValue)>) {
    let channel = BroadcastChannel::new(LAN_BROADCAST_CHANNEL)
        .expect("BroadcastChannel should be available");

    let on_message = Closure::wrap(Box::new(move |event: JsValue| {
        let event: web_sys::MessageEvent = event.unchecked_into();
        let data = event.data();

        // Parse the message type and code from the JSON object
        let msg_type = Reflect::get(&data, &JsValue::from_str("type"))
            .ok()
            .and_then(|v| v.as_string());
        let code = Reflect::get(&data, &JsValue::from_str("code"))
            .ok()
            .and_then(|v| v.as_string());

        if let (Some(msg_type), Some(code)) = (msg_type, code) {
            LAN_STATE.with(|s| {
                let mut state = s.borrow_mut();
                match msg_type.as_str() {
                    "offer" => {
                        info!("[LAN] Received offer via BroadcastChannel");
                        state.received_offer = Some(code);
                    }
                    "answer" => {
                        info!("[LAN] Received answer via BroadcastChannel");
                        state.received_answer = Some(code);
                    }
                    _ => {}
                }
            });
        }
    }) as Box<dyn FnMut(JsValue)>);

    channel.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

    (channel, on_message)
}

/// Posts a JSON message to the BroadcastChannel.
fn post_message(msg_type: &str, code: &str) {
    LAN_STATE.with(|s| {
        let state = s.borrow();
        if let Some(channel) = &state.channel {
            let obj = Object::new();
            Reflect::set(&obj, &JsValue::from_str("type"), &JsValue::from_str(msg_type)).ok();
            Reflect::set(&obj, &JsValue::from_str("code"), &JsValue::from_str(code)).ok();
            let _ = channel.post_message(&obj);
        }
    });
}

/// Start hosting on LAN: opens BroadcastChannel, listens for answers.
pub fn start_lan_host() {
    stop_lan_signaling();
    let (channel, closure) = open_channel();
    LAN_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.channel = Some(channel);
        state._closure = Some(closure);
    });
    info!("[LAN] Host signaling started");
}

/// Start joining on LAN: opens BroadcastChannel, listens for offers.
pub fn start_lan_guest() {
    stop_lan_signaling();
    let (channel, closure) = open_channel();
    LAN_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.channel = Some(channel);
        state._closure = Some(closure);
    });
    info!("[LAN] Guest signaling started");
}

/// Broadcast the host's offer code to the channel.
///
/// Called repeatedly by the polling system so that guests opening their tab
/// after the host already broadcast will still receive the offer.
pub fn broadcast_offer(code: &str) {
    post_message("offer", code);
}

/// Broadcast the guest's answer code to the channel.
pub fn broadcast_answer(code: &str) {
    post_message("answer", code);
}

/// Take the received offer code, if any (guest polls this).
pub fn take_received_offer() -> Option<String> {
    LAN_STATE.with(|s| s.borrow_mut().received_offer.take())
}

/// Take the received answer code, if any (host polls this).
pub fn take_received_answer() -> Option<String> {
    LAN_STATE.with(|s| s.borrow_mut().received_answer.take())
}

/// Stop LAN signaling and close the BroadcastChannel.
pub fn stop_lan_signaling() {
    LAN_STATE.with(|s| {
        let mut state = s.borrow_mut();
        if let Some(channel) = state.channel.take() {
            channel.close();
        }
        *state = LanSignalingState::default();
    });
}
