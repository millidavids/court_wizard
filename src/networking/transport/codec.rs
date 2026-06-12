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

    let mut chunks: Vec<&[u8]> = payload.chunks(max_payload).collect();
    // The header carries a u8 fragment index/count, so at most 255 fragments can
    // be reassembled. Cap the emitted set to 255 (rather than letting the index
    // wrap past 255 and corrupt reassembly). An oversized unreliable payload is
    // pathological — drop the tail and warn instead of shipping inconsistent
    // headers.
    if chunks.len() > 255 {
        bevy::log::warn!(
            "fragment_datagram: payload of {} bytes needs {} fragments (>255); truncating",
            payload.len(),
            chunks.len()
        );
        chunks.truncate(255);
    }
    let fragment_count = chunks.len() as u8;

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

/// One in-flight fragmented payload.
struct PendingPayload {
    sequence: u16,
    expected_count: u8,
    received_count: u8,
    fragments: Vec<Option<Vec<u8>>>,
}

/// Max number of concurrent in-progress sequences we keep around.
///
/// The host typically pushes 3 distinct payload kinds per frame (game
/// snapshot, spell snapshot, CRDT snapshot), each with up to ~10 fragments.
/// Under normal UDP reordering, fragments from successive payloads
/// interleave — a single-slot reassembler would discard whichever payload
/// hadn't finished yet. 8 slots leaves headroom for ~2 frames' worth of
/// in-flight payloads.
const MAX_PENDING_SEQUENCES: usize = 8;
/// A "completed" sentinel kept in the recent-completions ring so duplicates
/// or late stragglers from a finished sequence don't restart it.
const COMPLETED_SENTINEL_LEN: usize = 16;

/// Reassembly buffer for incoming datagram fragments.
///
/// Supports multiple concurrent in-progress sequences so a small/fast payload
/// arriving during the middle of a fragmented larger payload doesn't cause
/// the larger one to be dropped (which was the previous single-slot
/// behaviour).
pub(super) struct DatagramReassembler {
    /// In-progress payloads. Bounded by `MAX_PENDING_SEQUENCES`; when full,
    /// the oldest entry is evicted to make room for a new sequence.
    pending: Vec<PendingPayload>,
    /// Recently completed sequence numbers, to drop late duplicates.
    recently_completed: std::collections::VecDeque<u16>,
}

impl DatagramReassembler {
    pub(super) fn new() -> Self {
        Self {
            pending: Vec::with_capacity(MAX_PENDING_SEQUENCES),
            recently_completed: std::collections::VecDeque::with_capacity(COMPLETED_SENTINEL_LEN),
        }
    }

    fn mark_completed(&mut self, sequence: u16) {
        if self.recently_completed.len() >= COMPLETED_SENTINEL_LEN {
            self.recently_completed.pop_front();
        }
        self.recently_completed.push_back(sequence);
    }

    /// Returns the highest sequence number currently being tracked across
    /// both `pending` and `recently_completed`, using forward-progression
    /// (small wrapping_sub deltas) as the ordering metric. `None` if both
    /// are empty (cold start — no historical context to compare against).
    fn newest_pending_or_completed_sequence(&self) -> Option<u16> {
        let mut newest: Option<u16> = None;
        for seq in self
            .pending
            .iter()
            .map(|p| p.sequence)
            .chain(self.recently_completed.iter().copied())
        {
            newest = Some(match newest {
                None => seq,
                Some(cur) => {
                    // `seq` is "newer" than `cur` iff seq - cur is small
                    // (forward progression in wrap-aware terms).
                    if seq.wrapping_sub(cur) < 32768 {
                        seq
                    } else {
                        cur
                    }
                }
            });
        }
        newest
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

        // Drop late stragglers from already-completed sequences.
        if self.recently_completed.contains(&sequence) {
            return None;
        }

        // Drop fragments whose sequence is "much older" than anything we are
        // currently tracking, using wrap-aware comparison. This prevents a
        // post-wrap sequence number from being treated as a brand-new
        // payload when an orphan from the old wrap cycle could still arrive.
        if let Some(newest) = self.newest_pending_or_completed_sequence() {
            let diff_back = newest.wrapping_sub(sequence);
            // diff_back small (< 32768) means `sequence` is "older" than newest.
            // If it's older AND not actively pending, it's a wrap-around stale
            // packet — drop it.
            if diff_back > 0
                && diff_back < 32768
                && !self.pending.iter().any(|p| p.sequence == sequence)
            {
                return None;
            }
        }

        // Find or create the pending entry for this sequence.
        let slot = if let Some(idx) = self.pending.iter().position(|p| p.sequence == sequence) {
            idx
        } else {
            if self.pending.len() >= MAX_PENDING_SEQUENCES {
                // Evict the oldest (front) entry AND record its sequence
                // in `recently_completed` so any late-arriving fragments
                // for that sequence are dropped rather than resurrecting
                // it (which would evict another in-progress sequence and
                // could cascade under sustained UDP reordering).
                let evicted = self.pending.remove(0);
                self.mark_completed(evicted.sequence);
            }
            self.pending.push(PendingPayload {
                sequence,
                expected_count: fragment_count,
                received_count: 0,
                fragments: vec![None; fragment_count as usize],
            });
            self.pending.len() - 1
        };

        let entry = &mut self.pending[slot];

        if fragment_count != entry.expected_count {
            // Same sequence but mismatched fragment_count — treat as corrupt and drop.
            return None;
        }

        let idx = fragment_index as usize;
        if idx >= entry.fragments.len() {
            return None;
        }

        if entry.fragments[idx].is_none() {
            entry.received_count += 1;
        }
        entry.fragments[idx] = Some(payload);

        if entry.received_count == entry.expected_count {
            // Complete — reassemble and remove from pending.
            let completed = self.pending.remove(slot);
            let total_len: usize = completed
                .fragments
                .iter()
                .filter_map(|f| f.as_ref())
                .map(|f| f.len())
                .sum();
            self.mark_completed(sequence);
            // Reject zero-length reassemblies — no legitimate sender ships an
            // empty payload, and an empty `Vec<u8>` would surface to the
            // game-layer deserializers as a "Failed to deserialize (0 bytes)"
            // warning on every such datagram.
            if total_len == 0 {
                return None;
            }
            let mut result = Vec::with_capacity(total_len);
            for chunk in completed.fragments.iter().flatten() {
                result.extend_from_slice(chunk);
            }
            Some(result)
        } else {
            None
        }
    }
}
