//! Per-frame Steam Input pump.

use bevy::prelude::*;
use bevy_steamworks::Client;

/// Synchronizes Steam Input action data at the start of the frame, before the
/// poll producer reads it (lowest latency). `init` was called with
/// `explicitly_call_run_frame = true`, so we own this call.
pub(crate) fn steam_input_run_frame(client: Res<Client>) {
    client.input().run_frame();
}
