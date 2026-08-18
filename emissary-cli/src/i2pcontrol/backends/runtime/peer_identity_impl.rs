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

    /// 391-byte key-certificate EdDSA-SHA512-Ed25519 Destination fixture.
    /// This is the largest structurally valid serialized Destination
    /// currently produced by the repository parser. The trailing seven bytes
    /// carry the KEY_CERTIFICATE type (`0x05`), cert length (`0x0004`),
    /// signing key kind `EdDSA-SHA512-Ed25519` (`0x0007`), and a private
    /// key kind placeholder (`0x0000`).
    pub const KEY_CERT_DESTINATION_BYTES: [u8; 391] = [
        216, 194, 83, 151, 200, 24, 84, 242, 184, 222, 34, 89, 37, 175, 254, 228, 173, 78, 114, 24,
        15, 104, 241, 215, 166, 166, 102, 40, 136, 138, 22, 252, 114, 203, 192, 101, 32, 156, 212,
        74, 177, 120, 153, 172, 221, 181, 175, 190, 178, 17, 71, 33, 39, 211, 208, 241, 30, 35,
        222, 99, 215, 32, 242, 40, 202, 70, 197, 171, 84, 202, 52, 173, 221, 153, 242, 77, 240, 30,
        133, 4, 126, 31, 105, 24, 87, 209, 111, 140, 122, 58, 242, 224, 61, 95, 144, 26, 25, 129,
        202, 69, 130, 238, 88, 201, 34, 132, 197, 242, 129, 223, 50, 194, 130, 227, 102, 209, 79,
        209, 70, 202, 48, 248, 32, 124, 230, 90, 90, 104, 199, 23, 141, 60, 213, 122, 20, 94, 223,
        251, 48, 64, 30, 97, 36, 40, 194, 119, 98, 83, 29, 0, 31, 113, 241, 52, 72, 175, 208, 221,
        251, 220, 146, 82, 235, 83, 172, 33, 118, 249, 114, 218, 149, 42, 76, 8, 137, 125, 81, 209,
        156, 68, 75, 58, 79, 245, 124, 41, 243, 228, 244, 162, 239, 31, 176, 185, 44, 202, 6, 202,
        200, 127, 247, 43, 80, 178, 76, 120, 211, 75, 157, 84, 199, 229, 62, 10, 51, 143, 31, 218,
        237, 8, 232, 227, 1, 168, 159, 119, 35, 41, 43, 67, 241, 91, 87, 213, 118, 129, 172, 192,
        92, 176, 79, 63, 80, 251, 160, 212, 50, 194, 46, 229, 59, 15, 48, 93, 62, 80, 237, 86, 159,
        203, 194, 165, 80, 22, 108, 18, 64, 58, 210, 130, 124, 26, 198, 206, 159, 132, 252, 96,
        155, 124, 35, 108, 231, 22, 53, 246, 114, 232, 108, 192, 249, 122, 24, 236, 5, 210, 53, 149,
        124, 6, 12, 36, 59, 144, 19, 176, 11, 159, 46, 184, 45, 193, 58, 134, 179, 130, 176, 122,
        34, 177, 172, 147, 35, 19, 123, 22, 176, 182, 216, 78, 246, 104, 110, 62, 111, 117, 110,
        174, 49, 132, 214, 130, 96, 112, 30, 211, 159, 113, 131, 151, 166, 156, 206, 227, 20, 21,
        115, 66, 8, 218, 103, 153, 78, 46, 127, 199, 169, 197, 168, 124, 158, 232, 115, 71, 104,
        19, 165, 200, 234, 67, 168, 253, 137, 220, 5, 0, 4, 0, 7, 0, 0,
    ];

    /// Build a distinct, structurally valid peer identity from the null-cert
    /// fixture by varying the all-zero public key prefix. The substituted
    /// bytes do not participate in `Destination::parse` validation, so each
    /// variant remains a structurally valid Destination with a unique
    /// canonical 32-byte hash.
    pub(crate) fn distinct_peer(seed: u8) -> TrustedPeerIdentity {
        distinct_peer_u32(seed as u32)
    }

    /// Variant of [`distinct_peer`] that takes a `u32` seed, allowing tests
    /// to generate more than 256 distinct structurally-valid peer identities
    /// from the same null-certificate fixture.
    pub(crate) fn distinct_peer_u32(seed: u32) -> TrustedPeerIdentity {
        let mut bytes = NULL_CERT_DESTINATION_BYTES.to_vec();
        bytes[0] = seed as u8;
        bytes[1] = (seed >> 8) as u8;
        bytes[2] = (seed >> 16) as u8;
        bytes[3] = (seed >> 24) as u8;
        bytes[4] = (seed as u8).wrapping_add(1);
        bytes[5] = ((seed >> 8) as u8).wrapping_add(2);
        TrustedPeerIdentity::from_bytes_for_test(&bytes)
    }

    /// Build a distinct 391-byte key-certificate (EdDSA-SHA512-Ed25519)
    /// peer identity. The substituted bytes do not participate in
    /// `Destination::parse` validation, so each variant remains a
    /// structurally valid Destination with a unique canonical 32-byte hash.
    pub(crate) fn distinct_key_cert_peer(seed: u8) -> TrustedPeerIdentity {
        let mut bytes = KEY_CERT_DESTINATION_BYTES.to_vec();
        bytes[0] = seed;
        bytes[1] = seed.wrapping_add(1);
        bytes[2] = seed.wrapping_add(2);
        bytes[3] = seed.wrapping_add(3);
        bytes[4] = seed.wrapping_add(4);
        TrustedPeerIdentity::from_bytes_for_test(&bytes)
    }
}
