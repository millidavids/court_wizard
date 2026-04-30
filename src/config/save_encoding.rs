//! Save-file encoding helpers (XOR obfuscation, base64, timestamps, ID generation).

use super::progress::keyed_hash;

// ---------------------------------------------------------------------------
// Obfuscation helpers
// ---------------------------------------------------------------------------

/// Simple XOR cipher for obfuscating save data.
pub(super) fn obfuscate(data: &[u8]) -> Vec<u8> {
    let seed = b"unified_save_v2";
    let key_hash = keyed_hash(seed);
    let key_bytes = key_hash.to_le_bytes();

    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key_bytes[i % key_bytes.len()])
        .collect()
}

/// Deobfuscate is the same as obfuscate (XOR is symmetric).
pub(super) fn deobfuscate(data: &[u8]) -> Vec<u8> {
    obfuscate(data)
}

/// Convert bytes to base64 for storage.
pub(crate) fn to_base64(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    // Base64 emits 4 output chars per 3 input bytes, rounded up with padding.
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);

    for chunk in data.chunks(3) {
        let b1 = chunk[0];
        let b2 = chunk.get(1).copied().unwrap_or(0);
        let b3 = chunk.get(2).copied().unwrap_or(0);

        result.push(CHARS[(b1 >> 2) as usize] as char);
        result.push(CHARS[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
        result.push(if chunk.len() > 1 {
            CHARS[(((b2 & 0x0f) << 2) | (b3 >> 6)) as usize] as char
        } else {
            '='
        });
        result.push(if chunk.len() > 2 {
            CHARS[(b3 & 0x3f) as usize] as char
        } else {
            '='
        });
    }

    result
}

/// Convert base64 back to bytes.
pub(crate) fn from_base64(s: &str) -> Option<Vec<u8>> {
    let chars: Vec<u8> = s.bytes().collect();
    let mut result = Vec::new();

    for chunk in chars.chunks(4) {
        if chunk.len() < 4 {
            break;
        }

        let decode = |c: u8| -> Option<u8> {
            match c {
                b'A'..=b'Z' => Some(c - b'A'),
                b'a'..=b'z' => Some(c - b'a' + 26),
                b'0'..=b'9' => Some(c - b'0' + 52),
                b'+' => Some(62),
                b'/' => Some(63),
                b'=' => Some(0),
                _ => None,
            }
        };

        let b1 = decode(chunk[0])?;
        let b2 = decode(chunk[1])?;
        let b3 = decode(chunk[2])?;
        let b4 = decode(chunk[3])?;

        result.push((b1 << 2) | (b2 >> 4));
        if chunk[2] != b'=' {
            result.push((b2 << 4) | (b3 >> 2));
        }
        if chunk[3] != b'=' {
            result.push((b3 << 6) | b4);
        }
    }

    Some(result)
}

// ---------------------------------------------------------------------------
// UUID / timestamp helpers
// ---------------------------------------------------------------------------

/// Generate a simple unique identifier.
/// Format: "{timestamp}-{random_hex}" (e.g., "1704067200-a3f9c2")
pub(super) fn generate_id() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let timestamp = current_timestamp();
    let random: u32 = rng.random();
    format!("{}-{:06x}", timestamp, random & 0xFFFFFF)
}

/// Get current Unix timestamp in seconds.
pub(crate) fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
