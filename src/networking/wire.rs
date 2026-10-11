//! The one encoding for every network payload: join codes, reliable messages
//! and snapshots. Changing it means a `PROTOCOL_VERSION` bump.

use serde::{Deserialize, Serialize};

pub(crate) fn encode<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_allocvec(value)
}

pub(crate) fn decode<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, postcard::Error> {
    postcard::from_bytes(bytes)
}
