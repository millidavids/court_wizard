//! QUIC endpoint creation helpers — transport config, keep-alive, idle timeout, close.

use std::time::Duration;

use iroh::Endpoint;
use iroh::endpoint::{IdleTimeout, QuicTransportConfig};
use tokio::time::timeout;

/// QUIC keep-alive interval. Sent automatically while idle so a peer that
/// has stopped responding can be detected quickly.
pub(super) const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(2);

/// QUIC idle timeout. The connection is declared dead after this many seconds
/// of no traffic. With `KEEP_ALIVE_INTERVAL = 2s`, a live peer never hits this
/// — but a peer that hard-closes (window close / Alt+F4 / process kill) is
/// detected within ~5s. Default iroh value is 30s, which leaves the host
/// stranded in a dead match for far too long.
pub(super) const QUIC_IDLE_TIMEOUT: Duration = Duration::from_secs(5);

/// Application-Layer Protocol Negotiation identifier for Court Wizard P2P.
pub(super) const ALPN: &[u8] = b"court-wizard/1";

/// Default max datagram payload size if the connection doesn't report one.
pub(super) const DEFAULT_MAX_DATAGRAM: usize = 1200;

/// Maximum time we will block on `Endpoint::close()` during shutdown.
/// iroh's close awaits `wait_idle()` which can stall for the full QUIC idle
/// timeout (tens of seconds) when the relay/STUN infrastructure is
/// unreachable — that freezes the process at app exit because the tokio
/// thread won't return and the OS process won't terminate until all
/// non-daemon threads complete. Capping the wait keeps shutdown snappy.
const CLOSE_TIMEOUT: Duration = Duration::from_secs(2);

/// Builds the QUIC transport config we apply to every endpoint we create.
/// Both host and guest use the same values so the negotiated idle timeout
/// is short on both sides.
pub(super) fn build_transport_config() -> QuicTransportConfig {
    QuicTransportConfig::builder()
        .keep_alive_interval(KEEP_ALIVE_INTERVAL)
        .max_idle_timeout(Some(
            IdleTimeout::try_from(QUIC_IDLE_TIMEOUT).expect("valid idle timeout"),
        ))
        .build()
}

/// Close an iroh endpoint with a bounded timeout. See `CLOSE_TIMEOUT`.
pub(super) async fn close_endpoint(ep: &Endpoint) {
    let _ = timeout(CLOSE_TIMEOUT, ep.close()).await;
}
