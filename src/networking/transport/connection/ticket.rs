//! Connection code encoding/decoding — base64url serialization of EndpointAddr.

use iroh::EndpointAddr;

pub(super) fn encode_endpoint_addr(addr: &EndpointAddr) -> String {
    let bytes = bincode::serialize(addr).expect("EndpointAddr serialization should not fail");
    data_encoding::BASE64URL_NOPAD.encode(&bytes)
}

pub(super) fn decode_endpoint_addr(code: &str) -> Result<EndpointAddr, String> {
    let bytes = data_encoding::BASE64URL_NOPAD
        .decode(code.as_bytes())
        .map_err(|e| format!("Invalid base64: {e}"))?;
    bincode::deserialize(&bytes).map_err(|e| format!("Invalid address data: {e}"))
}
