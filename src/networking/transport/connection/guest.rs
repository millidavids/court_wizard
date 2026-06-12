//! Guest flow — parse connection code, connect to host, run I/O.

use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use iroh::{Endpoint, endpoint::presets};
use tokio::sync::Notify;

use crate::networking::resources::ConnectionState;

use super::endpoint::{ALPN, build_transport_config, close_endpoint};
use super::helpers::{send_error_and_fail, send_event};
use super::io::run_connection_io;
use super::ticket::decode_endpoint_addr;
use crate::networking::transport::runtime::{TransportCommand, TransportEvent};

/// Guest flow: parse connection code, connect to host, run I/O.
#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_guest(
    ticket_code: &str,
    command_rx: &mut tokio::sync::mpsc::UnboundedReceiver<TransportCommand>,
    event_tx: &Sender<TransportEvent>,
    reliable_rx: &Receiver<Vec<u8>>,
    unreliable_rx: &Receiver<Vec<u8>>,
    reliable_notify: &Arc<Notify>,
    unreliable_notify: &Arc<Notify>,
) {
    send_event(
        event_tx,
        TransportEvent::StateChanged(ConnectionState::Connecting),
    );

    let endpoint_addr = match decode_endpoint_addr(ticket_code.trim()) {
        Ok(addr) => addr,
        Err(e) => {
            send_error_and_fail(event_tx, format!("Invalid connection code: {e}"));
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
            send_error_and_fail(event_tx, format!("Failed to create endpoint: {e}"));
            return;
        }
    };

    let conn = match ep.connect(endpoint_addr, ALPN).await {
        Ok(conn) => conn,
        Err(e) => {
            send_error_and_fail(event_tx, format!("Failed to connect to host: {e}"));
            close_endpoint(&ep).await;
            return;
        }
    };

    send_event(
        event_tx,
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
        event_tx,
        reliable_rx,
        unreliable_rx,
        reliable_notify,
        unreliable_notify,
    )
    .await;

    close_endpoint(&ep).await;
}
