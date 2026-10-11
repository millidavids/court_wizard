//! Connection code encoding/decoding — base64url of a version byte + the wire-encoded EndpointAddr.

use iroh::EndpointAddr;

use crate::networking::wire;

/// Leading byte of every connection code, so a code from another build gets
/// "update the game" instead of a generic decode failure. Codes from builds
/// before this byte start with a random key byte: 1 in 256 slip past and fail
/// later as an unreachable peer.
const JOIN_CODE_VERSION: u8 = 2;

pub(super) fn encode_endpoint_addr(addr: &EndpointAddr) -> String {
    let mut bytes = vec![JOIN_CODE_VERSION];
    bytes.extend(wire::encode(addr).expect("EndpointAddr serialization should not fail"));
    data_encoding::BASE64URL_NOPAD.encode(&bytes)
}

pub(super) fn decode_endpoint_addr(code: &str) -> Result<EndpointAddr, String> {
    let bytes = data_encoding::BASE64URL_NOPAD
        .decode(code.as_bytes())
        .map_err(|e| format!("Invalid base64: {e}"))?;
    let Some((&version, payload)) = bytes.split_first() else {
        return Err("Empty code".to_string());
    };
    if version != JOIN_CODE_VERSION {
        return Err(
            "code is from a different game version — both players need the latest update"
                .to_string(),
        );
    }
    wire::decode(payload).map_err(|e| format!("Invalid address data: {e}"))
}

#[cfg(test)]
mod tests {
    use iroh::SecretKey;

    use super::*;

    fn sample_addr() -> EndpointAddr {
        EndpointAddr::new(SecretKey::from_bytes(&[7; 32]).public())
    }

    #[test]
    fn code_round_trips() {
        let addr = sample_addr();
        let decoded = decode_endpoint_addr(&encode_endpoint_addr(&addr)).expect("decodes");
        assert_eq!(decoded, addr);
    }

    // Both mark the transport generation; bump them together.
    #[test]
    fn alpn_matches_join_code_version() {
        let expected = format!("court-wizard/{JOIN_CODE_VERSION}");
        assert_eq!(super::super::endpoint::ALPN, expected.as_bytes());
    }

    #[test]
    fn other_version_is_reported_as_version_mismatch() {
        let mut bytes = vec![JOIN_CODE_VERSION + 1];
        bytes.extend(wire::encode(&sample_addr()).expect("serializes"));
        let code = data_encoding::BASE64URL_NOPAD.encode(&bytes);
        let err = decode_endpoint_addr(&code).expect_err("rejected");
        assert!(err.contains("different game version"), "{err}");
    }
}
