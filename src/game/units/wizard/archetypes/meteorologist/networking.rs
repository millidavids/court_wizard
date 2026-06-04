//! Multiplayer replication for the Meteorologist's weather.
//!
//! Weather is a GLOBAL effect: the host simulates it (applying Wet/Cold/Dry and
//! storm-lightning to its authoritative units) and BOTH peers render it. Only
//! the discrete weather CHOICE crosses the wire — intensity is ramped
//! independently on each peer (`tick_weather_timers` runs on both) from the
//! same formula, so they stay in sync without per-frame intensity packets.

use bevy::prelude::*;

use crate::config::{GameConfig, WizardType};
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::NetworkConnection;
use crate::networking::session::MultiplayerSession;

use super::constants::THUNDERSTORM_LIGHTNING_INTERVAL;
use super::messages::WeatherChangedMessage;
use super::resources::{WeatherState, WeatherType};

/// True when EITHER peer is the Meteorologist. The host runs the weather sim and
/// both peers render the weather whenever either player chose the archetype. In
/// single-player this is just the local check.
pub(crate) fn is_meteorologist_participating(
    config: Res<GameConfig>,
    session: Option<Res<MultiplayerSession>>,
) -> bool {
    if config.wizard_type == WizardType::Meteorologist {
        return true;
    }
    session.is_some_and(|s| {
        s.host_wizard == WizardType::Meteorologist || s.guest_wizard == WizardType::Meteorologist
    })
}

/// True when the OPPONENT is the Meteorologist (the receiving side of the
/// weather-choice message).
pub(crate) fn is_remote_meteorologist(session: Option<Res<MultiplayerSession>>) -> bool {
    session.is_some_and(|s| s.remote_wizard() == WizardType::Meteorologist)
}

/// Sends the local Meteorologist's weather choice to the peer whenever it
/// changes. No-op in single-player.
pub(crate) fn send_weather_state(
    connection: Option<ResMut<NetworkConnection>>,
    weather: Res<WeatherState>,
    mut last_sent: Local<Option<Option<WeatherType>>>,
) {
    let Some(mut connection) = connection else {
        return;
    };
    // We replicate only this peer's OWN weather (the local slot); the opponent
    // writes it into their `remote` slot on receipt.
    let local_active = weather.local.active;
    // First run: record the (cleared) starting state without sending a spurious
    // WeatherChanged{None} packet.
    if last_sent.is_none() {
        *last_sent = Some(local_active);
        return;
    }
    if *last_sent == Some(local_active) {
        return;
    }
    *last_sent = Some(local_active);
    connection
        .outgoing_messages
        .push(NetworkMessage::WeatherChanged {
            weather: local_active.map(WeatherType::to_u8),
            intensity: weather.local.intensity,
        });
}

/// Applies a peer's weather choice to the local `WeatherState` (restarting the
/// intensity ramp) and fires `WeatherChangedMessage` so the local SFX reacts.
pub(crate) fn receive_weather_message(
    connection: Option<ResMut<NetworkConnection>>,
    mut weather: ResMut<WeatherState>,
    mut writer: MessageWriter<WeatherChangedMessage>,
) {
    let Some(mut connection) = connection else {
        return;
    };
    if connection.incoming_messages.is_empty() {
        return;
    }
    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();
    for msg in messages {
        match msg {
            NetworkMessage::WeatherChanged {
                weather: w,
                intensity,
            } => {
                let new_active = w.and_then(WeatherType::from_u8);
                // The opponent's choice lands in OUR `remote` slot — it never
                // touches the local slot, so both weathers coexist. Intensity
                // then ramps locally from `time_active` (same formula on both
                // peers), keeping them in sync without per-frame packets.
                if weather.remote.active != new_active {
                    weather.remote.active = new_active;
                    weather.remote.time_active = 0.0;
                    weather.remote.intensity = intensity;
                    weather.remote.lightning_timer = THUNDERSTORM_LIGHTNING_INTERVAL;
                    writer.write(WeatherChangedMessage);
                }
            }
            other => unhandled.push(other),
        }
    }
    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}
