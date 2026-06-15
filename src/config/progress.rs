use std::collections::HashMap;
use std::fmt::Write;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::storage;

/// Keyed-hash constants used to sign local save/progress files.
/// NOTE: client-side, best-effort tamper-resistance only. These constants (and the
/// signing algorithm) ship inside the distributed binary and are public in this
/// open-source repo, so a determined user can forge a valid signature. Saves are
/// therefore not trusted input; any authoritative validation must be server-side.
const KEY_A: u64 = 0x9E37_79B9_7F4A_7C15;
const KEY_B: u64 = 0x6A09_E667_F3BC_C908;

/// Player progress data that gets signed and verified.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ProgressData {
    pub(crate) current_level: u32,
    pub(crate) highest_level_achieved: u32,
    pub(crate) efficiency_ratios: HashMap<String, f32>,
}

/// Signed progress container with data and its signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignedProgress {
    signature: String,
    data: ProgressData,
}

/// Computes a keyed hash of the input bytes using SipHash-style mixing.
/// Returns a 128-bit hash as two u64 values.
pub(crate) fn keyed_hash(data: &[u8]) -> u128 {
    let mut v0: u64 = KEY_A;
    let mut v1: u64 = KEY_B;
    let mut v2: u64 = KEY_A ^ 0xFF51_AFD7_ED55_8CCD;
    let mut v3: u64 = KEY_B ^ 0xC4CE_B9FE_1A85_EC53;

    // Process 8 bytes at a time
    let chunks = data.chunks_exact(8);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let m = u64::from_le_bytes(
            chunk
                .try_into()
                .expect("chunks_exact(8) guarantees 8-byte slices"),
        );
        v3 ^= m;
        for _ in 0..2 {
            v0 = v0.wrapping_add(v1);
            v1 = v1.rotate_left(13);
            v1 ^= v0;
            v0 = v0.rotate_left(32);
            v2 = v2.wrapping_add(v3);
            v3 = v3.rotate_left(16);
            v3 ^= v2;
            v0 = v0.wrapping_add(v3);
            v3 = v3.rotate_left(21);
            v3 ^= v0;
            v2 = v2.wrapping_add(v1);
            v1 = v1.rotate_left(17);
            v1 ^= v2;
            v2 = v2.rotate_left(32);
        }
        v0 ^= m;
    }

    // Process remaining bytes
    let mut last: u64 = (data.len() as u64) << 56;
    for (i, &byte) in remainder.iter().enumerate() {
        last |= (byte as u64) << (i * 8);
    }
    v3 ^= last;
    for _ in 0..2 {
        v0 = v0.wrapping_add(v1);
        v1 = v1.rotate_left(13);
        v1 ^= v0;
        v0 = v0.rotate_left(32);
        v2 = v2.wrapping_add(v3);
        v3 = v3.rotate_left(16);
        v3 ^= v2;
        v0 = v0.wrapping_add(v3);
        v3 = v3.rotate_left(21);
        v3 ^= v0;
        v2 = v2.wrapping_add(v1);
        v1 = v1.rotate_left(17);
        v1 ^= v2;
        v2 = v2.rotate_left(32);
    }
    v0 ^= last;

    // Finalization
    v2 ^= 0xFF;
    for _ in 0..4 {
        v0 = v0.wrapping_add(v1);
        v1 = v1.rotate_left(13);
        v1 ^= v0;
        v0 = v0.rotate_left(32);
        v2 = v2.wrapping_add(v3);
        v3 = v3.rotate_left(16);
        v3 ^= v2;
        v0 = v0.wrapping_add(v3);
        v3 = v3.rotate_left(21);
        v3 ^= v0;
        v2 = v2.wrapping_add(v1);
        v1 = v1.rotate_left(17);
        v1 ^= v2;
        v2 = v2.rotate_left(32);
    }

    let lo = v0 ^ v1 ^ v2 ^ v3;
    v1 ^= 0xDD;
    for _ in 0..4 {
        v0 = v0.wrapping_add(v1);
        v1 = v1.rotate_left(13);
        v1 ^= v0;
        v0 = v0.rotate_left(32);
        v2 = v2.wrapping_add(v3);
        v3 = v3.rotate_left(16);
        v3 ^= v2;
        v0 = v0.wrapping_add(v3);
        v3 = v3.rotate_left(21);
        v3 ^= v0;
        v2 = v2.wrapping_add(v1);
        v1 = v1.rotate_left(17);
        v1 ^= v2;
        v2 = v2.rotate_left(32);
    }
    let hi = v0 ^ v1 ^ v2 ^ v3;

    ((hi as u128) << 64) | (lo as u128)
}

/// Converts a u128 to a hex string.
pub(crate) fn to_hex(value: u128) -> String {
    let bytes = value.to_be_bytes();
    let mut hex = String::with_capacity(32);
    for byte in &bytes {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

/// Computes the signature for the given progress data.
fn compute_signature(data: &ProgressData) -> String {
    let canonical = toml::to_string(data).unwrap_or_default();
    let hash = keyed_hash(canonical.as_bytes());
    to_hex(hash)
}

/// Loads and verifies progress from disk.
/// Returns None if missing, tampered, or invalid.
pub(crate) fn load_verified_progress() -> Option<ProgressData> {
    let contents = storage::load_progress().ok()?;
    let signed: SignedProgress = toml::from_str(&contents).ok()?;

    let expected = compute_signature(&signed.data);
    if expected == signed.signature {
        Some(signed.data)
    } else {
        warn!("Progress signature mismatch — progress has been tampered with, resetting");
        None
    }
}
