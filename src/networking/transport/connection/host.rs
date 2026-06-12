//! Host flow — create endpoint, generate connection code, accept guest, run I/O.

use std::sync::Arc;

use bevy::log::warn;
use crossbeam_channel::{Receiver, Sender};
use iroh::{Endpoint, endpoint::presets};
use tokio::sync::Notify;

use crate::networking::resources::ConnectionState;

use super::endpoint::{ALPN, build_transport_config, close_endpoint};
use super::helpers::{send_error_and_fail, send_event, wait_for_disconnect};
use super::io::{ConnectionExitReason, run_connection_io};
use super::ticket::encode_endpoint_addr;
use crate::networking::transport::runtime::{TransportCommand, TransportEvent};

/// Host flow: create endpoint, generate connection code, wait for guest, run I/O.
#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_host(
    use_relay: bool,
    command_rx: &mut tokio::sync::mpsc::UnboundedReceiver<TransportCommand>,
    event_tx: &Sender<TransportEvent>,
    reliable_rx: &Receiver<Vec<u8>>,
    unreliable_rx: &Receiver<Vec<u8>>,
    reliable_notify: &Arc<Notify>,
    unreliable_notify: &Arc<Notify>,
) {
    let ep = if use_relay {
        Endpoint::builder(presets::N0)
            .alpns(vec![ALPN.to_vec()])
            .transport_config(build_transport_config())
            .bind()
            .await
    } else {
        Endpoint::builder(presets::N0DisableRelay)
            .alpns(vec![ALPN.to_vec()])
            .transport_config(build_transport_config())
            .bind()
            .await
    };

    let ep = match ep {
        Ok(ep) => ep,
        Err(e) => {
            send_error_and_fail(event_tx, format!("Failed to create endpoint: {e}"));
            return;
        }
    };

    // Wait for relay connectivity (online mode) with a timeout.
    if use_relay
        && tokio::time::timeout(std::time::Duration::from_secs(10), ep.online())
            .await
            .is_err()
    {
        warn!("Timed out waiting for relay, proceeding with local addresses only");
    }

    let addr = ep.addr();
    let ticket_str = encode_endpoint_addr(&addr);
    send_event(event_tx, TransportEvent::LocalCode(ticket_str));

    // Re-accept loop. The first iteration is the initial connect. If the guest
    // later drops without an explicit Disconnect (process kill, network loss),
    // `run_connection_io` returns `Lost` and we loop back to `ep.accept()` so
    // the same guest can reconnect — the endpoint stays bound the whole time
    // (it keeps the same ticket code). An explicit `Disconnect` (local leave)
    // closes the endpoint and ends the host flow.
    loop {
        // Wait for guest to connect, or a Disconnect command.
        let accept_handle = tokio::spawn({
            let ep = ep.clone();
            async move { ep.accept().await }
        });

        let conn = tokio::select! {
            result = accept_handle => {
                match result {
                    Ok(Some(incoming)) => match incoming.await {
                        Ok(conn) => conn,
                        Err(e) => {
                            send_error_and_fail(event_tx, format!("Guest connection failed: {e}"));
                            close_endpoint(&ep).await;
                            return;
                        }
                    },
                    Ok(None) => {
                        send_error_and_fail(event_tx, "Endpoint closed before guest connected".into());
                        return;
                    }
                    Err(e) => {
                        send_error_and_fail(event_tx, format!("Accept task panicked: {e}"));
                        return;
                    }
                }
            }
            _ = wait_for_disconnect(command_rx) => {
                close_endpoint(&ep).await;
                send_event(event_tx, TransportEvent::StateChanged(ConnectionState::Disconnected));
                return;
            }
        };

        send_event(
            event_tx,
            TransportEvent::StateChanged(ConnectionState::Connected),
        );

        // Host opens the bidirectional stream.
        let reason = run_connection_io(
            conn,
            &ep,
            true,
            command_rx,
            event_tx,
            reliable_rx,
            unreliable_rx,
            reliable_notify,
            unreliable_notify,
        )
        .await;

        match reason {
            // Local peer is leaving for good — shut the endpoint down.
            ConnectionExitReason::Disconnect => {
                close_endpoint(&ep).await;
                return;
            }
            // Guest vanished — keep the endpoint bound and re-listen so it can
            // reconnect at a level boundary (or sooner).
            ConnectionExitReason::Lost => continue,
        }
    }
}
