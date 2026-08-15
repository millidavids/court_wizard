//! Guest flow — parse connection code, connect to host, run I/O.

use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::Receiver;
use iroh::{Endpoint, endpoint::presets};
use tokio::sync::Notify;

use crate::networking::resources::ConnectionState;

use super::endpoint::{ALPN, build_transport_config, close_endpoint};
use super::helpers::{send_error_and_fail, send_event, wait_for_disconnect};
use super::io::run_connection_io;
use super::ticket::decode_endpoint_addr;
use crate::networking::transport::runtime::{EventSink, TransportCommand, TransportEvent};

/// Ceiling on a single dial attempt. Generous enough for relay selection and NAT
/// traversal on a slow link, but bounded so a host that has stopped listening
/// surfaces as an error instead of an indefinite "Connecting…".
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Guest flow: parse connection code, connect to host, run I/O.
#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_guest(
    ticket_code: &str,
    command_rx: &mut tokio::sync::mpsc::UnboundedReceiver<(u64, TransportCommand)>,
    events: &EventSink,
    reliable_rx: &Receiver<Vec<u8>>,
    unreliable_rx: &Receiver<Vec<u8>>,
    reliable_notify: &Arc<Notify>,
    unreliable_notify: &Arc<Notify>,
) {
    send_event(
        events,
        TransportEvent::StateChanged(ConnectionState::Connecting),
    );

    let endpoint_addr = match decode_endpoint_addr(ticket_code.trim()) {
        Ok(addr) => addr,
        Err(e) => {
            send_error_and_fail(events, format!("Invalid connection code: {e}"));
            return;
        }
    };

    let ep = match Endpoint::builder(presets::N0)
        .transport_config(build_transport_config())
        .bind()
        .await
    {
        Ok(ep) => ep,
        Err(e) => {
            send_error_and_fail(events, format!("Failed to create endpoint: {e}"));
            return;
        }
    };

    // The dial must be cancellable and bounded. Previously this was a bare
    // `.await`, so between emitting `Connecting` and entering `run_connection_io`
    // the guest never looked at `command_rx` — unlike `handle_host`, which
    // `select!`s against `wait_for_disconnect` around `ep.accept()`.
    //
    // Two consequences, both of which stranded the player:
    //   - Cancel didn't cancel. The UI reset to the Connect screen while this task
    //     kept dialling; if the dial later SUCCEEDED it emitted `Connected` into a
    //     lobby that had moved on, leaving a permanently stale
    //     `ConnectionState::Connected` that made every later Steam invite get
    //     refused by `accept_incoming_join`.
    //   - A host that vanished mid-dial left this pending with no deadline at all.
    let conn = tokio::select! {
        result = ep.connect(endpoint_addr, ALPN) => match result {
            Ok(conn) => conn,
            Err(e) => {
                send_error_and_fail(events, format!("Failed to connect to host: {e}"));
                close_endpoint(&ep).await;
                return;
            }
        },
        _ = wait_for_disconnect(command_rx, events) => {
            // Cancelled locally. Report Disconnected (not Failed) — the player did
            // this on purpose and shouldn't be shown an error panel.
            close_endpoint(&ep).await;
            send_event(
                events,
                TransportEvent::StateChanged(ConnectionState::Disconnected),
            );
            return;
        }
        _ = tokio::time::sleep(CONNECT_TIMEOUT) => {
            send_error_and_fail(
                events,
                "Couldn't reach the host — they may have stopped hosting.".to_string(),
            );
            close_endpoint(&ep).await;
            return;
        }
    };

    send_event(
        events,
        TransportEvent::StateChanged(ConnectionState::Connected),
    );

    // Guest accepts the bidirectional stream opened by host. The guest does not
    // re-listen on its own — reconnection is driven by the host re-issuing the
    // invite / the user re-joining from the lobby — so the exit reason is moot.
    let _ = run_connection_io(
        conn,
        &ep,
        false,
        command_rx,
        events,
        reliable_rx,
        unreliable_rx,
        reliable_notify,
        unreliable_notify,
    )
    .await;

    close_endpoint(&ep).await;
}
