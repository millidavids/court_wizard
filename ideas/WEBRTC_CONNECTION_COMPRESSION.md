# WebRTC Connection Code Compression & Reconstruction Plan

## Context

This is a Rust/Bevy game compiled to WebAssembly, hosted as a static site in the browser. Two players establish a WebRTC data channel connection by manually exchanging connection codes (copy/paste). There is no signaling server. The current connection codes are ~1000 characters because they contain the full raw SDP offer/answer with bundled ICE candidates. Most of that is boilerplate.

The goal is to:
1. Shorten the connection codes dramatically by extracting only the unique fields
2. Harden security by reconstructing SDP from a fixed template, eliminating SDP injection/munging attacks

## Architecture Overview

The system has two sides:

- **Encode**: After the browser generates an SDP offer or answer (with ICE candidates gathered), extract only the unique fields, validate them, pack them into a compact binary format, and present a base64url-encoded string to the player.
- **Decode + Reconstruct**: When a player pastes the remote peer's code, decode it, validate every field against strict type and length constraints, then slot the values into a hardcoded SDP template. Only the reconstructed SDP is passed to `setRemoteDescription()`.

The remote peer's raw SDP text must **never** be passed to the browser's WebRTC API. The browser only ever sees SDP that our code generated from the template.

---

## Part 1: Identifying Unique vs. Constant Fields

### Fields That Are Unique Per Session (must be in the connection code)

These are the only values that change between sessions and must be exchanged:

- **ICE ufrag** — short alphanumeric string (typically 4-8 chars). Parsed from `a=ice-ufrag:` in the SDP.
- **ICE pwd** — alphanumeric string (typically 22-24 chars). Parsed from `a=ice-pwd:` in the SDP.
- **DTLS fingerprint** — SHA-256 hash of the peer's ephemeral DTLS certificate. Appears as `a=fingerprint:sha-256 AA:BB:CC:...` in the SDP. This is 32 bytes of raw data represented as colon-separated hex. Store as raw 32 bytes in the binary format.
- **ICE candidates** — each candidate contains an IP address (4 bytes for IPv4, 16 for IPv6), a port (2 bytes), and a candidate type (host/srflx/relay — representable as a 2-bit enum). There may be 1-3 candidates. Parsed from `a=candidate:` lines in the SDP or from the gathered ICE candidate events.
- **DTLS setup role** — whether this is an offer (`actpass`) or answer (`active`). This is 1 bit of information — encode it as an offer/answer flag byte in the connection code, and use it to select the correct role in the template.

### Fields That Are Constant (hardcode in the reconstruction template)

These do not vary for a data-channel-only WebRTC connection and must NOT come from the remote peer's connection code:

- `v=0` — SDP version, always 0
- `o=` — origin line (session ID can be generated locally, not security-relevant)
- `s=-` — session name
- `t=0 0` — timing
- `a=group:BUNDLE` — bundle policy
- `m=application 9 UDP/DTLS/SCTP webrtc-datachannel` — the media line declaring a data channel
- `c=IN IP4 0.0.0.0` — connection line (placeholder, ICE handles actual connectivity)
- `a=mid:` — media ID for bundling
- `a=ice-options:trickle` — ICE options
- `a=sctp-port:` — the SCTP port number for data channels
- `a=max-message-size:` — maximum SCTP message size

### Why This Matters for Security

If the remote peer controlled the full SDP text, they could:

- **Inject additional `m=` media sections** (e.g., `m=audio`, `m=video`) to try to negotiate media streams the game never intended
- **Modify SCTP parameters** (`max-message-size`) to extreme values to cause crashes or memory issues
- **Add unknown `a=` attributes** that might trigger browser implementation bugs (fuzzed SDP has historically caused browser vulnerabilities)
- **Alter the DTLS setup role** in unexpected ways
- **Add extra ICE candidates** pointing to malicious relay servers

By reconstructing from a template, none of these attacks are possible. The only peer-controlled values are typed, length-constrained, and validated before insertion.

---

## Part 2: Binary Format Specification

Design a compact binary format for the connection code. Suggested layout:

```
[1 byte]  Version + flags
            - bits 0-3: format version (start at 1)
            - bit 4: is_offer (1) or is_answer (0)
            - bits 5-7: candidate count (0-7)

[variable] ICE ufrag
            - 1 byte length prefix
            - followed by UTF-8 bytes
            - validate: alphanumeric + '+' and '/', 4-256 chars

[variable] ICE pwd  
            - 1 byte length prefix
            - followed by UTF-8 bytes
            - validate: alphanumeric + '+' and '/', 22-256 chars

[32 bytes] DTLS fingerprint
            - raw SHA-256 bytes (NOT the colon-hex representation)

[per candidate, repeated by candidate count]:
    [1 byte]  flags
                - bit 0: 0 = IPv4, 1 = IPv6
                - bits 1-2: candidate type (0 = host, 1 = srflx, 2 = relay)
    [4 or 16 bytes] IP address (depending on IPv4/IPv6 flag)
    [2 bytes] port (big-endian u16)
```

After packing, compress with deflate (optional — at this size the overhead of deflate headers may not help, so benchmark with and without). Then base64url encode (RFC 4648 §5 — URL-safe alphabet, no padding).

### Expected Size

For a typical case with 1 IPv4 host candidate:
- 1 (header) + 1+4 (ufrag) + 1+24 (pwd) + 32 (fingerprint) + 1+4+2 (candidate) = ~70 bytes
- Base64url: ~95 characters

For 2 candidates (host + srflx):
- Add 7 bytes → ~77 bytes → ~103 characters

This is dramatically shorter than ~1000 characters.

---

## Part 3: Encoding (SDP → Connection Code)

Steps to implement:

1. Create the `RTCPeerConnection` and data channel as usual.
2. Create the offer or answer.
3. Set the local description.
4. **Wait for ICE gathering to complete** (vanilla ICE, not trickle). Listen for `icegatheringstatechange` and wait until state is `complete`, OR listen for the null `onicecandidate` event. This is critical — the connection code must be self-contained since there's no signaling channel to trickle candidates through.
5. Read `pc.localDescription.sdp` — this now contains the SDP with all candidates embedded.
6. Parse out the unique fields: ufrag, pwd, fingerprint, candidates.
7. Validate each field against the type constraints listed above.
8. Pack into the binary format.
9. Base64url encode.
10. Present the string to the player for copying.

### Parsing Notes

- The fingerprint line looks like `a=fingerprint:sha-256 AA:BB:CC:DD:...`. Split on spaces to get the hash algorithm and hex string. Verify the algorithm is `sha-256`. Convert the colon-separated hex to raw bytes.
- ICE candidates in the SDP look like `a=candidate:foundation component-id transport priority ip port typ type [raddr ip rport port]`. Parse by splitting on whitespace. Extract IP (field 4), port (field 5), and type (field 7).
- For the offer/answer flag, check whether the local description type is `"offer"` or `"answer"`.

---

## Part 4: Decoding + Reconstruction (Connection Code → SDP)

Steps to implement:

1. Player pastes the remote peer's connection code.
2. Base64url decode.
3. Unpack the binary format. **Validate strictly at every step**:
   - Version byte: reject if format version is unsupported
   - Ufrag/pwd: verify length bounds and character set (alphanumeric only)
   - Fingerprint: must be exactly 32 bytes
   - Candidate count: reject if > reasonable max (e.g., 5)
   - IP addresses: for IPv4, must be 4 bytes; for IPv6, must be 16 bytes
   - Ports: must be 1-65535
   - Candidate type: must be a known enum value
4. If any validation fails, show an error to the player ("Invalid connection code"). Do NOT attempt to use partially valid data.
5. **Reconstruct the SDP from the hardcoded template**, inserting only the validated fields:
   - Determine `a=setup:` role from the offer/answer flag (if remote is offer, local answer uses `active`; if remote is answer, use `passive` or as appropriate)
   - Convert fingerprint bytes back to colon-separated hex for the `a=fingerprint:sha-256` line
   - Build `a=candidate:` lines from the parsed candidate data, generating appropriate foundation and priority values
   - All other SDP fields come from constants in the template
6. Pass the reconstructed SDP string to `setRemoteDescription()` as a `RTCSessionDescription` with the appropriate type (`offer` or `answer`).
7. If this is an offer, create and set the local answer, then encode it as a connection code for the local player to share back.

### Template Construction Notes

- The SDP must end lines with `\r\n` (CRLF), not just `\n`.
- The `a=candidate:` line has a specific format: `candidate:foundation component transport priority ip port typ type`. For the foundation, a static value or hash is fine. Component should be `1` (RTCP-mux). Transport is `udp`. Priority should be calculated appropriately (host > srflx > relay) but exact values are flexible.
- Set `a=ice-options:trickle` even though we're using vanilla ICE — it's harmless and some browsers expect it.

---

## Part 5: Integration Points

This is a Rust/Bevy/WASM project. The WebRTC APIs are browser APIs, so the encode/decode logic will likely involve:

- Rust code for the binary packing/unpacking and validation (runs in WASM)
- `wasm-bindgen` or `web-sys` calls for the WebRTC API interactions (`RTCPeerConnection`, `setLocalDescription`, `setRemoteDescription`, etc.)
- The SDP string comes from JavaScript-land via the WebRTC API, gets passed to Rust for parsing/encoding, and the reconstructed SDP goes back through the WebRTC API

Alternatively, the encode/decode logic could live in a small JavaScript module called from Rust via `wasm-bindgen`, depending on what's easier for the existing codebase.

---

## Summary of Security Properties

By implementing this system:

- The browser's `setRemoteDescription()` only ever receives SDP generated by our template — never raw text from the remote peer
- No additional media sections can be injected (template only contains `m=application`)
- SCTP parameters are constants controlled by us, not the remote peer
- No unknown SDP attributes can be injected
- All peer-supplied values are typed, length-bounded, and validated before use
- The binary format has no room for extraneous data — the parser reads exactly the expected bytes and rejects anything else
- The only trust assumption remaining is the DTLS fingerprint (inherent to WebRTC without a CA) — consider adding a post-connection verification step where both players see a short hash derived from both fingerprints to confirm no MITM
