//! Canonical I2P Destination identity used by accepted-stream server tunnels.
//!
//! The trusted remote destination reported by SAM is structurally validated
//! against the repository's `Destination` parser. The canonical cryptographic
//! 32-byte Destination hash derived from the parsed Destination is the only
//! identity used by the shared admission/POST accounting layers. The original
//! textual representation remains available to protocol handlers that need
//! the validated full Destination for forwarding, but never to security
//! accounting that would otherwise be vulnerable to long textual representations
//! or unspecified general-purpose hashes.
//!
//! Malformed/non-canonical remote Destination text is rejected at this
//! boundary so it cannot reach admission state, POST accounting, or any other
//! attacker-influenced accounting structure.

use std::{fmt, sync::Arc};

use emissary_core::{crypto::base64_decode, primitives::Destination};
use yosemite::Stream;

/// Hard upper bound on the textual base64 representation that may be accepted
/// at the trusted boundary. The repository's `Destination::parse` accepts both
/// 387-byte null-certificate and 391-byte key-certificate forms, plus larger
/// forms that the parser will reject as `InvalidLength`/certificate errors.
/// The textual bound below is documented in `peer_identity` test fixtures and
/// is the maximum decoded payload the parser can be expected to consider
/// without consuming bounded memory.
pub const MAX_TRUSTED_DESTINATION_B64_TEXT: usize = 1024;

/// Result of structurally validating a remote I2P Destination reported by SAM.
///
/// The textual representation is the exact `Stream::remote_destination`
/// string SAM returned; the canonical 32-byte hash is the SHA-256 of the
/// serialized Destination bytes and is the only key used by security
/// accounting.
#[derive(Clone, PartialEq, Eq)]
pub struct TrustedPeerIdentity {
    destination: Arc<str>,
    canonical_id: [u8; 32],
}

impl TrustedPeerIdentity {
    /// Structurally validate `stream.remote_destination()` and return the
    /// canonical peer identity, or `None` if the text is missing, oversized,
    /// contains control characters, is not valid base64, or does not parse as
    /// an I2P `Destination`.
    ///
    /// This is the sole ingress for remote identity into the accepted-server
    /// runtime. Callers must not insert, store, or key any accounting
    /// structure on identity text that has not passed this check.
    pub fn from_stream(stream: &Stream) -> Option<Self> {
        let destination = stream.remote_destination();
        if destination.is_empty()
            || destination.len() > MAX_TRUSTED_DESTINATION_B64_TEXT
            || destination.chars().any(char::is_control)
            || destination.chars().any(|ch| ch.is_whitespace())
        {
            return None;
        }
        Self::from_destination_text(destination)
    }

    /// Parse a base64-encoded I2P `Destination` text and return the
    /// canonical peer identity, or `None` if the bytes do not parse as a
    /// structurally valid I2P `Destination`.
    ///
    /// This helper exists for trusted internal callers (fake SAM fixtures,
    /// restart restoration, test scaffolding). Production ingress remains
    /// [`Self::from_stream`].
    pub(crate) fn from_destination_text(destination: &str) -> Option<Self> {
        let decoded = base64_decode(destination)?;
        let parsed = Destination::parse(&decoded).ok()?;
        let id_bytes = parsed.id().to_vec();
        if id_bytes.len() != 32 {
            return None;
        }
        let mut canonical_id = [0u8; 32];
        canonical_id.copy_from_slice(&id_bytes);
        Some(Self {
            destination: Arc::from(destination),
            canonical_id,
        })
    }

    /// Return the validated base64-encoded remote Destination text.
    pub fn destination(&self) -> &str {
        &self.destination
    }

    /// Return the canonical 32-byte cryptographic Destination hash derived
    /// from the parsed Destination.
    pub fn canonical_id(&self) -> &[u8; 32] {
        &self.canonical_id
    }

    /// Construct a trusted peer identity from a precomputed structurally valid
    /// serialized Destination byte slice. The slice is required to be
    /// `Destination::parse`-able; the call panics if the fixture is
    /// malformed so test scaffolding cannot accidentally use a placeholder
    /// string in place of a real identity.
    #[cfg(test)]
    pub(crate) fn from_bytes_for_test(bytes: &[u8]) -> Self {
        use emissary_core::crypto::base64_encode;

        let encoded = base64_encode(bytes);
        Self::from_destination_text(&encoded)
            .expect("test Destination fixture must be structurally valid")
    }
}

impl fmt::Debug for TrustedPeerIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrustedPeerIdentity")
            .field("destination", &"<redacted>")
            .field("canonical_id", &"<redacted>")
            .finish()
    }
}

#[cfg(test)]
pub(crate) mod test_fixtures {
    use super::TrustedPeerIdentity;

    /// 387-byte null-certificate Destination fixture used as a seed for
    /// structurally valid test peer identities.
    pub const NULL_CERT_DESTINATION_BYTES: [u8; 387] = [
        89, 215, 97, 216, 78, 133, 203, 37, 193, 23, 180, 175, 81, 129, 202, 116, 223, 175, 141,
        253, 255, 55, 171, 170, 65, 99, 94, 4, 52, 204, 208, 253, 247, 98, 56, 144, 8, 235, 50,
        121, 218, 227, 152, 54, 102, 88, 90, 215, 80, 151, 201, 45, 105, 194, 111, 150, 231, 41,
        236, 223, 147, 139, 131, 104, 204, 163, 254, 235, 195, 27, 252, 175, 45, 87, 5, 129, 195,
        214, 73, 71, 123, 5, 241, 160, 202, 111, 179, 169, 193, 181, 171, 80, 220, 51, 203, 223,
        186, 127, 148, 75, 182, 26, 152, 25, 102, 180, 46, 140, 103, 104, 254, 252, 136, 42, 206,
        104, 44, 134, 43, 90, 241, 162, 207, 9, 243, 64, 3, 164, 186, 123, 101, 12, 142, 59, 70,
        237, 2, 23, 151, 26, 76, 121, 206, 249, 118, 65, 221, 38, 85, 86, 111, 58, 228, 247, 63,
        16, 130, 187, 183, 96, 137, 52, 83, 59, 88, 128, 76, 3, 52, 22, 230, 247, 2, 39, 177, 177,
        225, 175, 113, 237, 1, 246, 180, 217, 7, 32, 69, 90, 145, 55, 99, 231, 65, 123, 170, 80,
        155, 59, 71, 191, 244, 244, 86, 79, 18, 248, 162, 33, 197, 41, 145, 141, 197, 123, 34, 229,
        95, 91, 32, 64, 80, 94, 25, 224, 61, 233, 185, 90, 62, 246, 77, 25, 222, 138, 156, 215, 96,
        124, 184, 12, 121, 188, 121, 73, 44, 66, 248, 222, 10, 100, 196, 140, 7, 62, 92, 130, 137,
        208, 23, 127, 230, 216, 113, 197, 69, 34, 60, 231, 58, 153, 52, 110, 87, 245, 178, 77, 243,
        155, 124, 210, 91, 98, 191, 85, 181, 122, 207, 25, 157, 5, 184, 122, 205, 117, 175, 179,
        43, 188, 147, 87, 207, 150, 230, 72, 126, 184, 215, 34, 72, 189, 46, 170, 35, 195, 137, 36,
        218, 69, 84, 18, 16, 73, 114, 195, 251, 222, 147, 107, 42, 203, 64, 246, 152, 195, 251,
        141, 103, 231, 151, 104, 78, 134, 229, 214, 33, 138, 227, 124, 196, 123, 5, 35, 84, 36,
        117, 165, 26, 85, 153, 239, 12, 103, 236, 69, 186, 82, 228, 107, 105, 81, 176, 67, 111, 34,
        228, 116, 251, 171, 27, 72, 187, 116, 221, 112, 0, 0, 0,
    ];

    /// Build a distinct, structurally valid peer identity from the null-cert
    /// fixture by varying the all-zero public key prefix. The substituted
    /// bytes do not participate in `Destination::parse` validation, so each
    /// variant remains a structurally valid Destination with a unique
    /// canonical 32-byte hash.
    pub(crate) fn distinct_peer(seed: u8) -> TrustedPeerIdentity {
        let mut bytes = NULL_CERT_DESTINATION_BYTES.to_vec();
        bytes[0] = seed;
        bytes[1] = seed.wrapping_add(1);
        bytes[2] = seed.wrapping_add(2);
        TrustedPeerIdentity::from_bytes_for_test(&bytes)
    }
}
