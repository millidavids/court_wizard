//! Compact connection code encoding/decoding for WebRTC signaling.
//!
//! Replaces the raw SDP base64 exchange (~1000 chars) with a compact binary
//! format (~95 chars) that only contains the 5 session-unique fields.
//! On decode, a hardcoded SDP template is reconstructed, preventing SDP
//! injection attacks.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use thiserror::Error;

use super::constants::CONNECTION_CODE_VERSION;

// ===== Error Type =====

#[derive(Error, Debug)]
pub(super) enum ConnectionCodeError {
    #[error("unsupported connection code version: {0}")]
    UnsupportedVersion(u8),

    #[error("invalid base64url encoding")]
    InvalidEncoding,

    #[error("connection code too short")]
    TooShort,

    #[error("invalid ufrag: {0}")]
    InvalidUfrag(String),

    #[error("invalid pwd: {0}")]
    InvalidPwd(String),

    #[error("invalid fingerprint")]
    InvalidFingerprint,

    #[error("invalid candidate: {0}")]
    InvalidCandidate(String),

    #[error("SDP parsing failed: {0}")]
    SdpParseError(String),

    #[error("trailing data in connection code")]
    TrailingData,
}

// ===== Types =====

/// ICE candidate transport type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CandidateType {
    Host = 0,
    Srflx = 1,
    Relay = 2,
}

/// A single ICE candidate with only the fields needed for connection.
#[derive(Debug, Clone)]
pub(super) struct IceCandidate {
    pub ip: IpAddr,
    pub port: u16,
    pub candidate_type: CandidateType,
}

/// Decoded connection code containing all session-unique fields.
#[derive(Debug, Clone)]
pub(super) struct ConnectionCode {
    /// True if this is an offer, false if answer.
    pub is_offer: bool,
    /// ICE username fragment.
    pub ufrag: String,
    /// ICE password.
    pub pwd: String,
    /// DTLS certificate fingerprint (SHA-256, raw 32 bytes).
    pub fingerprint: [u8; 32],
    /// ICE candidates (typically 1-3).
    pub candidates: Vec<IceCandidate>,
}

// ===== Base64url Encoding (RFC 4648 §5, no padding) =====

const BASE64URL_CHARS: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn base64url_encode(input: &[u8]) -> String {
    let mut output = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        output.push(BASE64URL_CHARS[((triple >> 18) & 0x3F) as usize] as char);
        output.push(BASE64URL_CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            output.push(BASE64URL_CHARS[((triple >> 6) & 0x3F) as usize] as char);
        }
        if chunk.len() > 2 {
            output.push(BASE64URL_CHARS[(triple & 0x3F) as usize] as char);
        }
    }
    output
}

fn base64url_decode(input: &str) -> Result<Vec<u8>, ConnectionCodeError> {
    // Build reverse lookup
    let mut table = [255u8; 128];
    for (i, &c) in BASE64URL_CHARS.iter().enumerate() {
        table[c as usize] = i as u8;
    }

    let bytes = input.as_bytes();
    let len = bytes.len();

    // Length mod 4 determines trailing bytes (0=aligned, 2=1 byte, 3=2 bytes, 1=invalid)
    if len % 4 == 1 {
        return Err(ConnectionCodeError::InvalidEncoding);
    }

    let mut output = Vec::with_capacity(len * 3 / 4);
    let full_chunks = len / 4;

    for i in 0..full_chunks {
        let idx = i * 4;
        let a = lookup_b64(bytes[idx], &table)?;
        let b = lookup_b64(bytes[idx + 1], &table)?;
        let c = lookup_b64(bytes[idx + 2], &table)?;
        let d = lookup_b64(bytes[idx + 3], &table)?;
        let triple = (a << 18) | (b << 12) | (c << 6) | d;
        output.push((triple >> 16) as u8);
        output.push((triple >> 8) as u8);
        output.push(triple as u8);
    }

    // Handle remaining 2 or 3 chars (no padding)
    let remainder = len % 4;
    if remainder == 2 {
        let idx = full_chunks * 4;
        let a = lookup_b64(bytes[idx], &table)?;
        let b = lookup_b64(bytes[idx + 1], &table)?;
        let triple = (a << 18) | (b << 12);
        output.push((triple >> 16) as u8);
    } else if remainder == 3 {
        let idx = full_chunks * 4;
        let a = lookup_b64(bytes[idx], &table)?;
        let b = lookup_b64(bytes[idx + 1], &table)?;
        let c = lookup_b64(bytes[idx + 2], &table)?;
        let triple = (a << 18) | (b << 12) | (c << 6);
        output.push((triple >> 16) as u8);
        output.push((triple >> 8) as u8);
    }

    Ok(output)
}

fn lookup_b64(byte: u8, table: &[u8; 128]) -> Result<u32, ConnectionCodeError> {
    if byte >= 128 || table[byte as usize] == 255 {
        return Err(ConnectionCodeError::InvalidEncoding);
    }
    Ok(table[byte as usize] as u32)
}

// ===== SDP Parsing (encode path) =====

/// Extracts the value from the first SDP line matching the given prefix.
fn parse_sdp_field<'a>(sdp: &'a str, prefix: &str) -> Option<&'a str> {
    sdp.lines()
        .find(|line| line.starts_with(prefix))
        .map(|line| line[prefix.len()..].trim_end_matches('\r'))
}

/// Parses the DTLS fingerprint from an SDP string.
fn parse_fingerprint(sdp: &str) -> Result<[u8; 32], ConnectionCodeError> {
    let hex_str = parse_sdp_field(sdp, "a=fingerprint:sha-256 ")
        .ok_or_else(|| ConnectionCodeError::SdpParseError("no sha-256 fingerprint".into()))?;

    let bytes: Result<Vec<u8>, _> = hex_str
        .split(':')
        .map(|h| u8::from_str_radix(h, 16))
        .collect();

    let bytes = bytes.map_err(|_| ConnectionCodeError::InvalidFingerprint)?;

    bytes
        .try_into()
        .map_err(|_| ConnectionCodeError::InvalidFingerprint)
}

/// Parses all ICE candidates from an SDP string.
///
/// Skips candidates that can't be parsed (e.g., mDNS `.local` hostnames,
/// prflx candidates) since they aren't useful for copy-paste signaling.
fn parse_candidates(sdp: &str) -> Vec<IceCandidate> {
    sdp.lines()
        .filter(|l| l.starts_with("a=candidate:"))
        .filter_map(|l| parse_single_candidate(l).ok())
        .collect()
}

/// Parses a single `a=candidate:` SDP line.
///
/// Format: `a=candidate:foundation component transport priority ip port typ type [...]`
/// Indices:    [0]            [1]       [2]       [3]    [4] [5]  [6] [7]
fn parse_single_candidate(line: &str) -> Result<IceCandidate, ConnectionCodeError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 8 {
        return Err(ConnectionCodeError::InvalidCandidate(
            "too few fields".into(),
        ));
    }

    // Find "typ" keyword — it's usually at index 6, but search to be safe
    let typ_idx = parts
        .iter()
        .position(|&p| p == "typ")
        .ok_or_else(|| ConnectionCodeError::InvalidCandidate("missing typ keyword".into()))?;

    if typ_idx + 1 >= parts.len() {
        return Err(ConnectionCodeError::InvalidCandidate(
            "missing type after typ".into(),
        ));
    }

    let ip: IpAddr = parts[4]
        .parse()
        .map_err(|_| ConnectionCodeError::InvalidCandidate(format!("bad IP: {}", parts[4])))?;

    let port: u16 = parts[5]
        .parse()
        .map_err(|_| ConnectionCodeError::InvalidCandidate(format!("bad port: {}", parts[5])))?;

    let candidate_type = match parts[typ_idx + 1] {
        "host" => CandidateType::Host,
        "srflx" => CandidateType::Srflx,
        "relay" => CandidateType::Relay,
        // Skip unknown candidate types (e.g., prflx) rather than failing
        other => {
            return Err(ConnectionCodeError::InvalidCandidate(format!(
                "unknown type: {}",
                other
            )));
        }
    };

    Ok(IceCandidate {
        ip,
        port,
        candidate_type,
    })
}

// ===== Credential Validation =====

/// Valid characters for ICE ufrag/pwd: alphanumeric, +, /
fn is_valid_ice_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '+' || c == '/'
}

fn validate_ufrag(ufrag: &str) -> Result<(), ConnectionCodeError> {
    if ufrag.len() < 4 || ufrag.len() > 256 {
        return Err(ConnectionCodeError::InvalidUfrag(format!(
            "length {} (must be 4-256)",
            ufrag.len()
        )));
    }
    if !ufrag.chars().all(is_valid_ice_char) {
        return Err(ConnectionCodeError::InvalidUfrag(
            "invalid characters".into(),
        ));
    }
    Ok(())
}

fn validate_pwd(pwd: &str) -> Result<(), ConnectionCodeError> {
    if pwd.len() < 22 || pwd.len() > 256 {
        return Err(ConnectionCodeError::InvalidPwd(format!(
            "length {} (must be 22-256)",
            pwd.len()
        )));
    }
    if !pwd.chars().all(is_valid_ice_char) {
        return Err(ConnectionCodeError::InvalidPwd("invalid characters".into()));
    }
    Ok(())
}

// ===== Encoding (SDP → Connection Code) =====

/// Encodes an SDP string and type into a compact connection code.
pub(super) fn encode(sdp: &str, sdp_type: &str) -> Result<String, ConnectionCodeError> {
    let is_offer = match sdp_type {
        "offer" => true,
        "answer" => false,
        other => {
            return Err(ConnectionCodeError::SdpParseError(format!(
                "unknown SDP type: {}",
                other
            )));
        }
    };

    let ufrag = parse_sdp_field(sdp, "a=ice-ufrag:")
        .ok_or_else(|| ConnectionCodeError::SdpParseError("no ice-ufrag".into()))?
        .to_string();

    let pwd = parse_sdp_field(sdp, "a=ice-pwd:")
        .ok_or_else(|| ConnectionCodeError::SdpParseError("no ice-pwd".into()))?
        .to_string();

    let fingerprint = parse_fingerprint(sdp)?;

    // Skip unparseable candidates (mDNS .local hostnames, prflx, etc.)
    let candidates: Vec<IceCandidate> = parse_candidates(sdp);

    validate_ufrag(&ufrag)?;
    validate_pwd(&pwd)?;

    if candidates.len() > 7 {
        return Err(ConnectionCodeError::InvalidCandidate(format!(
            "too many candidates: {} (max 7)",
            candidates.len()
        )));
    }

    // Pack binary format
    let mut buf = Vec::with_capacity(128);

    // Header byte: version (4 bits) | is_offer (1 bit) | candidate_count (3 bits)
    let header = (CONNECTION_CODE_VERSION & 0x0F)
        | (u8::from(is_offer) << 4)
        | ((candidates.len() as u8) << 5);
    buf.push(header);

    // Ufrag: length prefix + bytes
    buf.push(ufrag.len() as u8);
    buf.extend_from_slice(ufrag.as_bytes());

    // Pwd: length prefix + bytes
    buf.push(pwd.len() as u8);
    buf.extend_from_slice(pwd.as_bytes());

    // Fingerprint: raw 32 bytes
    buf.extend_from_slice(&fingerprint);

    // Candidates
    for c in &candidates {
        let is_ipv6 = matches!(c.ip, IpAddr::V6(_));
        let type_bits = (c.candidate_type as u8) << 1;
        let flags = u8::from(is_ipv6) | type_bits;
        buf.push(flags);

        match c.ip {
            IpAddr::V4(v4) => buf.extend_from_slice(&v4.octets()),
            IpAddr::V6(v6) => buf.extend_from_slice(&v6.octets()),
        }

        buf.extend_from_slice(&c.port.to_be_bytes());
    }

    Ok(base64url_encode(&buf))
}

// ===== Decoding (Connection Code → ConnectionCode) =====

/// Decodes a compact connection code string into structured fields.
pub(super) fn decode(code: &str) -> Result<ConnectionCode, ConnectionCodeError> {
    let bytes = base64url_decode(code)?;
    let mut pos = 0;

    let read_byte = |pos: &mut usize| -> Result<u8, ConnectionCodeError> {
        if *pos >= bytes.len() {
            return Err(ConnectionCodeError::TooShort);
        }
        let b = bytes[*pos];
        *pos += 1;
        Ok(b)
    };

    let read_bytes = |pos: &mut usize, n: usize| -> Result<&[u8], ConnectionCodeError> {
        if *pos + n > bytes.len() {
            return Err(ConnectionCodeError::TooShort);
        }
        let slice = &bytes[*pos..*pos + n];
        *pos += n;
        Ok(slice)
    };

    // Header
    let header = read_byte(&mut pos)?;
    let version = header & 0x0F;
    if version != CONNECTION_CODE_VERSION {
        return Err(ConnectionCodeError::UnsupportedVersion(version));
    }
    let is_offer = (header >> 4) & 1 == 1;
    let candidate_count = ((header >> 5) & 0x07) as usize;

    // Ufrag
    let ufrag_len = read_byte(&mut pos)? as usize;
    let ufrag_bytes = read_bytes(&mut pos, ufrag_len)?;
    let ufrag = std::str::from_utf8(ufrag_bytes)
        .map_err(|_| ConnectionCodeError::InvalidUfrag("not valid UTF-8".into()))?
        .to_string();
    validate_ufrag(&ufrag)?;

    // Pwd
    let pwd_len = read_byte(&mut pos)? as usize;
    let pwd_bytes = read_bytes(&mut pos, pwd_len)?;
    let pwd = std::str::from_utf8(pwd_bytes)
        .map_err(|_| ConnectionCodeError::InvalidPwd("not valid UTF-8".into()))?
        .to_string();
    validate_pwd(&pwd)?;

    // Fingerprint
    let fp_bytes = read_bytes(&mut pos, 32)?;
    let mut fingerprint = [0u8; 32];
    fingerprint.copy_from_slice(fp_bytes);

    // Candidates
    let mut candidates = Vec::with_capacity(candidate_count);
    for _ in 0..candidate_count {
        let flags = read_byte(&mut pos)?;
        let is_ipv6 = flags & 1 == 1;
        let type_val = (flags >> 1) & 0x03;

        let candidate_type = match type_val {
            0 => CandidateType::Host,
            1 => CandidateType::Srflx,
            2 => CandidateType::Relay,
            other => {
                return Err(ConnectionCodeError::InvalidCandidate(format!(
                    "unknown type enum: {}",
                    other
                )));
            }
        };

        let ip = if is_ipv6 {
            let octets = read_bytes(&mut pos, 16)?;
            let mut arr = [0u8; 16];
            arr.copy_from_slice(octets);
            IpAddr::V6(Ipv6Addr::from(arr))
        } else {
            let octets = read_bytes(&mut pos, 4)?;
            let mut arr = [0u8; 4];
            arr.copy_from_slice(octets);
            IpAddr::V4(Ipv4Addr::from(arr))
        };

        let port_bytes = read_bytes(&mut pos, 2)?;
        let port = u16::from_be_bytes([port_bytes[0], port_bytes[1]]);
        if port == 0 {
            return Err(ConnectionCodeError::InvalidCandidate(
                "port must be 1-65535".into(),
            ));
        }

        candidates.push(IceCandidate {
            ip,
            port,
            candidate_type,
        });
    }

    // Reject trailing data
    if pos != bytes.len() {
        return Err(ConnectionCodeError::TrailingData);
    }

    Ok(ConnectionCode {
        is_offer,
        ufrag,
        pwd,
        fingerprint,
        candidates,
    })
}

// ===== SDP Reconstruction =====

impl ConnectionCode {
    /// Reconstructs a full SDP string from the decoded fields.
    ///
    /// Returns `(sdp_type, sdp_string)` — e.g., `("offer", "v=0\r\n...")`.
    /// The SDP is built from a hardcoded template; only validated fields are inserted.
    pub(super) fn to_sdp(&self) -> (String, String) {
        let sdp_type = if self.is_offer { "offer" } else { "answer" };
        let setup_role = if self.is_offer { "actpass" } else { "active" };

        // Fingerprint bytes → colon-separated uppercase hex
        let fingerprint_hex = self
            .fingerprint
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(":");

        // Build candidate lines
        let candidate_lines: String = self
            .candidates
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let type_str = match c.candidate_type {
                    CandidateType::Host => "host",
                    CandidateType::Srflx => "srflx",
                    CandidateType::Relay => "relay",
                };
                let priority = match c.candidate_type {
                    CandidateType::Host => 2130706431u32,
                    CandidateType::Srflx => 1694498815u32,
                    CandidateType::Relay => 16777215u32,
                };
                format!(
                    "a=candidate:{} 1 udp {} {} {} typ {}\r\n",
                    i + 1,
                    priority,
                    c.ip,
                    c.port,
                    type_str,
                )
            })
            .collect();

        let sdp = format!(
            "v=0\r\n\
             o=- 0 0 IN IP4 0.0.0.0\r\n\
             s=-\r\n\
             t=0 0\r\n\
             a=group:BUNDLE 0\r\n\
             a=msid-semantic: WMS\r\n\
             m=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\n\
             c=IN IP4 0.0.0.0\r\n\
             a=mid:0\r\n\
             a=ice-ufrag:{ufrag}\r\n\
             a=ice-pwd:{pwd}\r\n\
             a=ice-options:trickle\r\n\
             a=fingerprint:sha-256 {fingerprint}\r\n\
             a=setup:{setup}\r\n\
             a=sctp-port:5000\r\n\
             a=max-message-size:262144\r\n\
             {candidates}",
            ufrag = self.ufrag,
            pwd = self.pwd,
            fingerprint = fingerprint_hex,
            setup = setup_role,
            candidates = candidate_lines,
        );

        (sdp_type.to_string(), sdp)
    }
}

// ===== Unit Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64url_roundtrip_empty() {
        let encoded = base64url_encode(&[]);
        let decoded = base64url_decode(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn base64url_roundtrip_short() {
        let data = b"hello";
        let encoded = base64url_encode(data);
        let decoded = base64url_decode(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn base64url_roundtrip_all_byte_values() {
        let data: Vec<u8> = (0..=255).collect();
        let encoded = base64url_encode(&data);
        let decoded = base64url_decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn base64url_invalid_char() {
        assert!(base64url_decode("abc!").is_err());
    }

    #[test]
    fn base64url_invalid_length() {
        // Length mod 4 == 1 is invalid
        assert!(base64url_decode("a").is_err());
    }

    fn sample_sdp_offer() -> String {
        "v=0\r\n\
         o=- 4567890123456789012 2 IN IP4 127.0.0.1\r\n\
         s=-\r\n\
         t=0 0\r\n\
         a=group:BUNDLE 0\r\n\
         a=msid-semantic: WMS\r\n\
         m=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\n\
         c=IN IP4 0.0.0.0\r\n\
         a=mid:0\r\n\
         a=ice-ufrag:abcd\r\n\
         a=ice-pwd:aabbccddee11223344556677\r\n\
         a=ice-options:trickle\r\n\
         a=fingerprint:sha-256 A0:B1:C2:D3:E4:F5:06:17:28:39:4A:5B:6C:7D:8E:9F:A0:B1:C2:D3:E4:F5:06:17:28:39:4A:5B:6C:7D:8E:9F\r\n\
         a=setup:actpass\r\n\
         a=sctp-port:5000\r\n\
         a=max-message-size:262144\r\n\
         a=candidate:842163049 1 udp 2130706431 192.168.1.100 54321 typ host\r\n\
         a=candidate:2513270441 1 udp 1694498815 203.0.113.141 8998 typ srflx raddr 192.168.1.100 rport 54321\r\n"
            .to_string()
    }

    fn sample_sdp_answer() -> String {
        "v=0\r\n\
         o=- 9876543210987654321 2 IN IP4 127.0.0.1\r\n\
         s=-\r\n\
         t=0 0\r\n\
         a=group:BUNDLE 0\r\n\
         a=msid-semantic: WMS\r\n\
         m=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\n\
         c=IN IP4 0.0.0.0\r\n\
         a=mid:0\r\n\
         a=ice-ufrag:wxyz\r\n\
         a=ice-pwd:zzyyxxwwvvuuttssrrqqpp\r\n\
         a=ice-options:trickle\r\n\
         a=fingerprint:sha-256 FF:EE:DD:CC:BB:AA:99:88:77:66:55:44:33:22:11:00:FF:EE:DD:CC:BB:AA:99:88:77:66:55:44:33:22:11:00\r\n\
         a=setup:active\r\n\
         a=sctp-port:5000\r\n\
         a=max-message-size:262144\r\n\
         a=candidate:1 1 udp 2130706431 10.0.0.5 12345 typ host\r\n"
            .to_string()
    }

    #[test]
    fn encode_decode_roundtrip_offer() {
        let sdp = sample_sdp_offer();
        let code = encode(&sdp, "offer").unwrap();

        // Verify the code is much shorter than the raw SDP
        assert!(code.len() < 200, "code too long: {} chars", code.len());

        let decoded = decode(&code).unwrap();
        assert!(decoded.is_offer);
        assert_eq!(decoded.ufrag, "abcd");
        assert_eq!(decoded.pwd, "aabbccddee11223344556677");
        assert_eq!(decoded.candidates.len(), 2);

        // First candidate: host
        assert_eq!(
            decoded.candidates[0].ip,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))
        );
        assert_eq!(decoded.candidates[0].port, 54321);
        assert_eq!(decoded.candidates[0].candidate_type, CandidateType::Host);

        // Second candidate: srflx
        assert_eq!(
            decoded.candidates[1].ip,
            IpAddr::V4(Ipv4Addr::new(203, 0, 113, 141))
        );
        assert_eq!(decoded.candidates[1].port, 8998);
        assert_eq!(decoded.candidates[1].candidate_type, CandidateType::Srflx);
    }

    #[test]
    fn encode_decode_roundtrip_answer() {
        let sdp = sample_sdp_answer();
        let code = encode(&sdp, "answer").unwrap();
        let decoded = decode(&code).unwrap();
        assert!(!decoded.is_offer);
        assert_eq!(decoded.ufrag, "wxyz");
        assert_eq!(decoded.pwd, "zzyyxxwwvvuuttssrrqqpp");
        assert_eq!(decoded.candidates.len(), 1);
        assert_eq!(decoded.candidates[0].candidate_type, CandidateType::Host);
    }

    #[test]
    fn fingerprint_roundtrip() {
        let sdp = sample_sdp_offer();
        let fp = parse_fingerprint(&sdp).unwrap();
        assert_eq!(
            fp,
            [
                0xA0, 0xB1, 0xC2, 0xD3, 0xE4, 0xF5, 0x06, 0x17, 0x28, 0x39, 0x4A, 0x5B, 0x6C,
                0x7D, 0x8E, 0x9F, 0xA0, 0xB1, 0xC2, 0xD3, 0xE4, 0xF5, 0x06, 0x17, 0x28, 0x39,
                0x4A, 0x5B, 0x6C, 0x7D, 0x8E, 0x9F,
            ]
        );
    }

    #[test]
    fn sdp_reconstruction_offer_has_actpass() {
        let sdp = sample_sdp_offer();
        let code = encode(&sdp, "offer").unwrap();
        let decoded = decode(&code).unwrap();
        let (sdp_type, reconstructed) = decoded.to_sdp();
        assert_eq!(sdp_type, "offer");
        assert!(reconstructed.contains("a=setup:actpass\r\n"));
        assert!(reconstructed.contains("a=ice-ufrag:abcd\r\n"));
        assert!(reconstructed.contains("a=ice-pwd:aabbccddee11223344556677\r\n"));
        assert!(reconstructed.contains("typ host\r\n"));
        assert!(reconstructed.contains("typ srflx\r\n"));
    }

    #[test]
    fn sdp_reconstruction_answer_has_active() {
        let sdp = sample_sdp_answer();
        let code = encode(&sdp, "answer").unwrap();
        let decoded = decode(&code).unwrap();
        let (sdp_type, reconstructed) = decoded.to_sdp();
        assert_eq!(sdp_type, "answer");
        assert!(reconstructed.contains("a=setup:active\r\n"));
    }

    #[test]
    fn sdp_reconstruction_has_crlf() {
        let sdp = sample_sdp_offer();
        let code = encode(&sdp, "offer").unwrap();
        let decoded = decode(&code).unwrap();
        let (_, reconstructed) = decoded.to_sdp();
        // Every line should end with \r\n
        for line in reconstructed.split("\r\n") {
            assert!(
                !line.contains('\n'),
                "line contains bare LF: {:?}",
                line
            );
        }
    }

    #[test]
    fn decode_invalid_version() {
        // Manually build a binary with version 2
        let mut buf = vec![0x02u8]; // version 2, no offer, 0 candidates
        buf.push(4); // ufrag len
        buf.extend_from_slice(b"abcd");
        buf.push(22); // pwd len
        buf.extend_from_slice(b"aabbccddee1122334455xx");
        buf.extend_from_slice(&[0u8; 32]); // fingerprint
        let code = base64url_encode(&buf);
        let result = decode(&code);
        assert!(matches!(result, Err(ConnectionCodeError::UnsupportedVersion(2))));
    }

    #[test]
    fn decode_truncated() {
        let code = base64url_encode(&[0x11]); // Just a header byte, nothing else
        let result = decode(&code);
        assert!(matches!(result, Err(ConnectionCodeError::TooShort)));
    }

    #[test]
    fn decode_trailing_data_rejected() {
        let sdp = sample_sdp_offer();
        let code = encode(&sdp, "offer").unwrap();
        let mut bytes = base64url_decode(&code).unwrap();
        bytes.push(0xFF); // Extra byte
        let bad_code = base64url_encode(&bytes);
        let result = decode(&bad_code);
        assert!(matches!(result, Err(ConnectionCodeError::TrailingData)));
    }

    #[test]
    fn ipv6_candidate_roundtrip() {
        let candidate = IceCandidate {
            ip: IpAddr::V6(Ipv6Addr::new(
                0x2001, 0xdb8, 0, 0, 0, 0, 0, 1,
            )),
            port: 8080,
            candidate_type: CandidateType::Host,
        };

        let code = ConnectionCode {
            is_offer: true,
            ufrag: "abcd".to_string(),
            pwd: "aabbccddee11223344556677".to_string(),
            fingerprint: [0xAA; 32],
            candidates: vec![candidate],
        };

        // Manually encode and decode to verify IPv6 roundtrip
        let (_, sdp) = code.to_sdp();
        assert!(sdp.contains("2001:db8::1"));

        // Also test binary roundtrip
        let encoded = encode(&sdp, "offer").unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded.candidates.len(), 1);
        assert_eq!(
            decoded.candidates[0].ip,
            IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1))
        );
        assert_eq!(decoded.candidates[0].port, 8080);
    }

    #[test]
    fn code_length_is_short() {
        let sdp = sample_sdp_offer();
        let code = encode(&sdp, "offer").unwrap();
        // With 2 IPv4 candidates, should be well under 150 chars
        assert!(
            code.len() < 150,
            "code should be < 150 chars, got {}",
            code.len()
        );
        // And dramatically shorter than the raw SDP
        assert!(code.len() < sdp.len() / 3);
    }
}
