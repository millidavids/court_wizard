use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{GameSnapshot, UNRELIABLE_GAME_SNAPSHOT};

/// Drains `connection.incoming_unreliable`, separates game-snapshot packets
/// (prefix byte `UNRELIABLE_GAME_SNAPSHOT`) from everything else, re-queues the
/// non-game packets, and returns the raw payload bytes of the latest game
/// snapshot (without the prefix byte) if one was received.
///
/// Returning `None` means no game snapshot arrived this frame; the caller
/// should return early.
pub(super) fn filter_latest_game_snapshot(connection: &mut NetworkConnection) -> Option<Vec<u8>> {
    let raw_data: Vec<Vec<u8>> = std::mem::take(&mut connection.incoming_unreliable);
    let mut other_data: Vec<Vec<u8>> = Vec::new();
    let mut latest_game_data: Option<Vec<u8>> = None;

    for data in raw_data {
        if data.is_empty() {
            continue;
        }
        match data[0] {
            UNRELIABLE_GAME_SNAPSHOT => {
                latest_game_data = Some(data[1..].to_vec());
            }
            _ => {
                other_data.push(data);
            }
        }
    }

    // Re-queue non-game data for other systems (spell snapshots)
    connection.incoming_unreliable = other_data;

    latest_game_data
}

/// Deserializes raw bytes into a [`GameSnapshot`], logging a warning and
/// returning `None` on failure.
pub(super) fn parse_game_snapshot(game_bytes: &[u8]) -> Option<GameSnapshot> {
    match bincode::deserialize::<GameSnapshot>(game_bytes) {
        Ok(snapshot) => Some(snapshot),
        Err(_) => {
            bevy::log::warn!(
                "Failed to deserialize game snapshot ({} bytes)",
                game_bytes.len()
            );
            None
        }
    }
}
