//! Message framing and datagram fragmentation for the transport layer.

use bytes::{Buf, BufMut, Bytes, BytesMut};

/// Maximum size for a single reliable message (64 KB safety limit).
const MAX_RELIABLE_MESSAGE_SIZE: u32 = 65_536;

/// Header size for datagram fragments: sequence(2) + fragment_index(1) + fragment_count(1).
pub(super) const DATAGRAM_HEADER_SIZE: usize = 4;

// ── Reliable stream framing ──────────────────────────────────────────────────

/// Encode a reliable message with a 4-byte big-endian length prefix.
pub(super) fn encode_reliable(payload: &[u8]) -> Bytes {
    let len = payload.len() as u32;
    let mut buf = BytesMut::with_capacity(4 + payload.len());
    buf.put_u32(len);
    buf.put_slice(payload);
    buf.freeze()
}

/// Read exactly one length-prefixed message from a QUIC recv stream.
///
/// Returns `None` if the stream was cleanly closed (EOF on the length prefix).
pub(super) async fn decode_reliable(
    recv: &mut iroh::endpoint::RecvStream,
) -> Result<Option<Vec<u8>>, String> {
    // Read 4-byte length prefix.
    let mut len_buf = [0u8; 4];
    match recv.read_exact(&mut len_buf).await {
        Ok(()) => {}
        Err(e) => {
            let msg = format!("{e}");
            if msg.contains("closed") || msg.contains("finished") || msg.contains("reset") {
                return Ok(None); // clean EOF
            }
            return Err(format!("Failed to read message length: {e}"));
        }
    }

    let len = u32::from_be_bytes(len_buf);
    if len > MAX_RELIABLE_MESSAGE_SIZE {
        return Err(format!(
            "Message size {len} exceeds maximum {MAX_RELIABLE_MESSAGE_SIZE}"
        ));
    }

    let mut payload = vec![0u8; len as usize];
    recv.read_exact(&mut payload)
        .await
        .map_err(|e| format!("Failed to read message payload: {e}"))?;

    Ok(Some(payload))
}

// ── Unreliable datagram fragmentation ────────────────────────────────────────

/// Fragment a payload into datagram-sized pieces.
///
/// Each fragment has a 4-byte header: `[sequence_hi][sequence_lo][fragment_index][fragment_count]`.
/// The receiver reassembles complete frames and drops stale partial ones.
pub(super) fn fragment_datagram(
    payload: &[u8],
    sequence: u16,
    max_datagram_size: usize,
) -> Vec<Bytes> {
    let max_payload = max_datagram_size.saturating_sub(DATAGRAM_HEADER_SIZE);
    if max_payload == 0 {
        return Vec::new();
    }

    let chunks: Vec<&[u8]> = payload.chunks(max_payload).collect();
    let fragment_count = chunks.len().min(255) as u8;

    chunks
        .into_iter()
        .enumerate()
        .map(|(i, chunk)| {
            let mut buf = BytesMut::with_capacity(DATAGRAM_HEADER_SIZE + chunk.len());
            buf.put_u16(sequence);
            buf.put_u8(i as u8);
            buf.put_u8(fragment_count);
            buf.put_slice(chunk);
            buf.freeze()
        })
        .collect()
}

/// Reassembly buffer for incoming datagram fragments.
pub(super) struct DatagramReassembler {
    /// Current sequence number being assembled.
    current_sequence: u16,
    /// Expected total fragment count.
    expected_count: u8,
    /// Received fragments indexed by fragment_index.
    fragments: Vec<Option<Vec<u8>>>,
    /// How many fragments have been received for the current sequence.
    received_count: u8,
}

impl DatagramReassembler {
    pub(super) fn new() -> Self {
        Self {
            current_sequence: 0,
            expected_count: 0,
            fragments: Vec::new(),
            received_count: 0,
        }
    }

    /// Feed a raw datagram fragment. Returns the reassembled payload if complete.
    pub(super) fn feed(&mut self, mut data: Bytes) -> Option<Vec<u8>> {
        if data.len() < DATAGRAM_HEADER_SIZE {
            return None;
        }

        let sequence = data.get_u16();
        let fragment_index = data.get_u8();
        let fragment_count = data.get_u8();
        let payload = data.to_vec();

        if fragment_count == 0 {
            return None;
        }

        if sequence != self.current_sequence {
            // Different sequence — check if newer (wrapping comparison).
            let diff = sequence.wrapping_sub(self.current_sequence);
            if diff > 0 && diff <= 32768 {
                // Newer sequence: reset and start fresh.
                self.current_sequence = sequence;
                self.expected_count = fragment_count;
                self.fragments = vec![None; fragment_count as usize];
                self.received_count = 0;
            } else {
                // Older or way-future sequence — drop.
                return None;
            }
        } else if fragment_count != self.expected_count {
            // Same sequence but different fragment_count — corrupted, ignore.
            return None;
        }

        let idx = fragment_index as usize;
        if idx >= self.fragments.len() {
            return None;
        }

        if self.fragments[idx].is_none() {
            self.received_count += 1;
        }
        self.fragments[idx] = Some(payload);

        // All fragments received — reassemble.
        if self.received_count == self.expected_count {
            let total_len: usize = self
                .fragments
                .iter()
                .filter_map(|f| f.as_ref())
                .map(|f| f.len())
                .sum();
            let mut result = Vec::with_capacity(total_len);
            for data in self.fragments.iter().flatten() {
                result.extend_from_slice(data);
            }
            // Advance sequence so duplicates are dropped.
            self.current_sequence = self.current_sequence.wrapping_add(1);
            self.received_count = 0;
            self.fragments.clear();
            Some(result)
        } else {
            None
        }
    }
}
