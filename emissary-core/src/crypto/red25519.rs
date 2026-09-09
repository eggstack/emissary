// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

//! Neutral Ed25519-to-Red25519 blinding primitive.
//!
//! Implements the smallest standard key-blinding and signing operations needed
//! for type-5 encrypted LeaseSet2: daily blinding-scalar derivation, Edwards
//! blinding of public keys, matching blinded private-scalar derivation,
//! randomized Red25519 sign/verify, and blinded storage-key derivation.
//!
//! The API is intentionally narrow and free of administrative vocabulary:
//! callers pass explicit 32-byte keys, numeric signature-type codes, an
//! 8-byte ASCII `YYYYMMDD` UTC-day string, and optional UTF-8 secret bytes.
//! No persistent signature-type registry is created and no legacy
//! DSA/ECDSA generation is added.

use crate::error::Error;

use curve25519_dalek::{edwards::CompressedEdwardsY, traits::Identity, EdwardsPoint, Scalar};
use hmac::Mac;
use sha2::Digest;
use zeroize::{Zeroize, Zeroizing};

/// Unblinded Ed25519 signature-type code accepted as blinding input.
pub const ED25519_SIGTYPE: u16 = 7;

/// Blinded Red25519 signature-type code produced by this module.
pub const RED25519_SIGTYPE: u16 = 11;

/// Maximum Red25519 message length in bytes (`len_u16` reserves 65535).
pub const MAX_MESSAGE_LEN: usize = 65534;

/// HKDF info label for daily blinding-scalar derivation.
const BLINDING_INFO: &[u8] = b"i2pblinding1";

/// Personalization prefix for the alpha-derivation hash.
const GENERATE_ALPHA_DOMAIN: &[u8] = b"I2PGenerateAlpha";

/// Personalization prefix for the Red25519 hash (`HStar`).
const RED25519_HASH_DOMAIN: &[u8] = b"I2P_Red25519H(x)";

/// Length of the randomized signing transcript (`T`).
const SIGN_TRANSCRIPT_LEN: usize = 80;

/// Daily blinding scalar `alpha`.
///
/// Secret material: never `Debug`- or display-formattable and zeroized on
/// drop.
#[derive(Clone)]
pub struct Alpha(Zeroizing<[u8; 32]>);

impl Alpha {
    /// Return the canonical scalar bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Construct from canonical scalar bytes, failing closed on
    /// non-canonical encodings.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, Error> {
        if !bool::from(Scalar::from_canonical_bytes(*bytes).is_some()) {
            return Err(Error::InvalidData);
        }
        Ok(Self(Zeroizing::new(*bytes)))
    }
}

impl AsRef<[u8]> for Alpha {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/// Blinded private scalar (`a' = (a + alpha) mod L`).
///
/// Secret material: never `Debug`- or display-formattable and zeroized on
/// drop.
#[derive(Clone)]
pub struct BlindedPrivateKey(Zeroizing<[u8; 32]>);

impl BlindedPrivateKey {
    /// Return the canonical scalar bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Construct from canonical scalar bytes, failing closed on
    /// non-canonical encodings.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, Error> {
        if !bool::from(Scalar::from_canonical_bytes(*bytes).is_some()) {
            return Err(Error::InvalidData);
        }
        Ok(Self(Zeroizing::new(*bytes)))
    }
}

impl AsRef<[u8]> for BlindedPrivateKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/// Blinded public key (`A'`, 32-byte Edwards encoding).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BlindedPublicKey([u8; 32]);

impl BlindedPublicKey {
    /// Construct from 32-byte Edwards encoding, failing closed on invalid,
    /// non-canonical, small-order, or identity points.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, Error> {
        let point = decompress_torsion_free(bytes)?;
        if point == EdwardsPoint::identity() {
            return Err(Error::InvalidData);
        }
        Ok(Self(*bytes))
    }

    /// Return the 32-byte Edwards encoding.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert into the 32-byte Edwards encoding.
    pub fn to_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl AsRef<[u8]> for BlindedPublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Red25519 signature (`Rbytes || S`, 64 bytes).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RedSignature([u8; 64]);

impl RedSignature {
    /// Construct from 64 bytes, failing closed on invalid `R` encodings or
    /// non-canonical `S`.
    pub fn from_bytes(bytes: &[u8; 64]) -> Result<Self, Error> {
        let mut r_bytes = [0u8; 32];
        r_bytes.copy_from_slice(&bytes[..32]);
        let mut s_bytes = [0u8; 32];
        s_bytes.copy_from_slice(&bytes[32..]);
        CompressedEdwardsY(r_bytes).decompress().ok_or(Error::InvalidData)?;
        if !bool::from(Scalar::from_canonical_bytes(s_bytes).is_some()) {
            return Err(Error::InvalidData);
        }
        Ok(Self(*bytes))
    }

    /// Return the 64-byte signature.
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// Convert into the 64-byte signature.
    pub fn to_bytes(self) -> [u8; 64] {
        self.0
    }
}

impl AsRef<[u8]> for RedSignature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Format a `YYYYMMDD` UTC-day string from calendar components.
///
/// Fails closed on out-of-range month/day values. Leap-day validation is
/// intentionally minimal (day `1..=31` with February capped at 29) because
/// the string is a deterministic domain-separation input, not a calendar
/// library.
pub fn format_day_string(year: u16, month: u8, day: u8) -> Result<[u8; 8], Error> {
    if year > 9999 || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(Error::InvalidData);
    }
    if month == 2 && day > 29 {
        return Err(Error::InvalidData);
    }
    if matches!(month, 4 | 6 | 9 | 11) && day > 30 {
        return Err(Error::InvalidData);
    }
    let mut out = [b'0'; 8];
    out[0] = b'0' + (year / 1000) as u8;
    out[1] = b'0' + ((year / 100) % 10) as u8;
    out[2] = b'0' + ((year / 10) % 10) as u8;
    out[3] = b'0' + (year % 10) as u8;
    out[4] = b'0' + (month / 10);
    out[5] = b'0' + (month % 10);
    out[6] = b'0' + (day / 10);
    out[7] = b'0' + (day % 10);
    validate_day_string(&out)?;
    Ok(out)
}

/// Validate an 8-byte ASCII `YYYYMMDD` string.
fn validate_day_string(day: &[u8; 8]) -> Result<(), Error> {
    if !day.iter().all(u8::is_ascii_digit) {
        return Err(Error::InvalidData);
    }
    let month = (day[4] - b'0') * 10 + (day[5] - b'0');
    let date = (day[6] - b'0') * 10 + (day[7] - b'0');
    if !(1..=12).contains(&month) || !(1..=31).contains(&date) {
        return Err(Error::InvalidData);
    }
    Ok(())
}

/// Derive the daily blinding scalar `alpha`.
///
/// Inputs are the unblinded 32-byte signing public key, the unblinded
/// signature-type code (7 or 11), the blinded signature-type code (must be
/// 11), the 8-byte ASCII `YYYYMMDD` UTC-day string, and the optional UTF-8
/// secret (possibly empty).
///
/// Derivation follows the frozen construction:
/// `keydata = A || stA || stA'`,
/// `salt = SHA-256("I2PGenerateAlpha" || keydata)`,
/// `seed = HKDF-SHA256(salt, datestring || secret, "i2pblinding1", 64)`,
/// `alpha = seed mod L` with `seed` as 64-byte little-endian.
pub fn generate_alpha(
    unblinded_pubkey: &[u8; 32],
    unblinded_sigtype: u16,
    blinded_sigtype: u16,
    day: &[u8; 8],
    secret: &[u8],
) -> Result<Alpha, Error> {
    if !matches!(unblinded_sigtype, ED25519_SIGTYPE | RED25519_SIGTYPE) {
        return Err(Error::InvalidData);
    }
    if blinded_sigtype != RED25519_SIGTYPE {
        return Err(Error::InvalidData);
    }
    validate_day_string(day)?;
    core::str::from_utf8(secret).map_err(|_| Error::InvalidData)?;
    // Reject non-canonical or small-order unblinded points before deriving.
    let decoded = decompress_torsion_free(unblinded_pubkey)?;
    if decoded == EdwardsPoint::identity() {
        return Err(Error::InvalidData);
    }

    let mut keydata = [0u8; 36];
    keydata[..32].copy_from_slice(unblinded_pubkey);
    keydata[32..34].copy_from_slice(&unblinded_sigtype.to_be_bytes());
    keydata[34..36].copy_from_slice(&blinded_sigtype.to_be_bytes());

    let mut salt_input = [0u8; 52];
    salt_input[..GENERATE_ALPHA_DOMAIN.len()].copy_from_slice(GENERATE_ALPHA_DOMAIN);
    salt_input[GENERATE_ALPHA_DOMAIN.len()..].copy_from_slice(&keydata);
    let salt = sha256(&salt_input);

    let mut ikm = Zeroizing::new([0u8; 72]);
    let mut ikm_len = 8;
    ikm[..8].copy_from_slice(day);
    if !secret.is_empty() {
        if secret.len() > 64 {
            return Err(Error::InvalidData);
        }
        ikm_len = 8 + secret.len();
        ikm[8..ikm_len].copy_from_slice(secret);
    }
    let seed = Zeroizing::new(hkdf_sha256_64(&salt, &ikm[..ikm_len], BLINDING_INFO));
    let alpha = Scalar::from_bytes_mod_order_wide(&seed);
    Ok(Alpha(Zeroizing::new(alpha.to_bytes())))
}

/// Blind an unblinded Edwards public key: `A' = A + [alpha]B`.
///
/// Fails closed on invalid, non-canonical, small-order, or identity inputs.
pub fn blind_public_key(
    unblinded_pubkey: &[u8; 32],
    alpha: &Alpha,
) -> Result<BlindedPublicKey, Error> {
    let base = decompress_torsion_free(unblinded_pubkey)?;
    if base == EdwardsPoint::identity() {
        return Err(Error::InvalidData);
    }
    let alpha_scalar = scalar_from_canonical(alpha.as_bytes())?;
    let blinded = base + EdwardsPoint::mul_base(&alpha_scalar);
    if blinded == EdwardsPoint::identity() || blinded.is_small_order() {
        return Err(Error::InvalidData);
    }
    Ok(BlindedPublicKey(blinded.compress().to_bytes()))
}

/// Convert a 32-byte Ed25519 seed into its clamped Red25519 scalar.
pub fn convert_ed25519_seed(seed: &[u8; 32]) -> [u8; 32] {
    let mut hasher = sha2::Sha512::new();
    hasher.update(seed);
    let digest = hasher.finalize();
    let mut scalar = [0u8; 32];
    scalar.copy_from_slice(&digest[..32]);
    scalar[0] &= 248;
    scalar[31] = (scalar[31] & 63) | 64;
    scalar
}

/// Blind an Ed25519 seed: `a' = (clamp(SHA-512(seed)[..32]) + alpha) mod L`.
pub fn blind_private_key_ed25519(seed: &[u8; 32], alpha: &Alpha) -> BlindedPrivateKey {
    let converted = convert_ed25519_seed(seed);
    // Clamped Ed25519 output is a 256-bit integer that may exceed L, so reduce
    // mod L before adding (equivalent to `(a + alpha) mod L` on integers).
    let base = Scalar::from_bytes_mod_order(converted);
    let alpha_scalar = Scalar::from_canonical_bytes(*alpha.as_bytes()).expect("alpha is canonical");
    BlindedPrivateKey(Zeroizing::new((base + alpha_scalar).to_bytes()))
}

/// Blind a Red25519 scalar: `a' = (a + alpha) mod L`.
///
/// Fails closed on non-canonical input scalars.
pub fn blind_private_key_red25519(
    scalar_bytes: &[u8; 32],
    alpha: &Alpha,
) -> Result<BlindedPrivateKey, Error> {
    let base = scalar_from_canonical(scalar_bytes)?;
    let alpha_scalar = scalar_from_canonical(alpha.as_bytes())?;
    Ok(BlindedPrivateKey(Zeroizing::new(
        (base + alpha_scalar).to_bytes(),
    )))
}

/// Derive the public key for a blinded private scalar: `[sk]B`.
pub fn derive_public(private: &BlindedPrivateKey) -> BlindedPublicKey {
    let scalar =
        Scalar::from_canonical_bytes(*private.as_bytes()).expect("blinded scalar is canonical");
    BlindedPublicKey(EdwardsPoint::mul_base(&scalar).compress().to_bytes())
}

/// Sign `message` with a blinded private scalar using a secure RNG.
///
/// Construction: `T` is 80 fresh random bytes, `r = HStar(T, vkBytes, m)`,
/// `R = [r]B`, `c = HStar(Rbytes, vkBytes, m)`, `S = (r + c*sk) mod L`.
/// Every signature differs even for the same key/message. Fails closed when
/// `message` exceeds [`MAX_MESSAGE_LEN`].
pub fn sign(
    private: &BlindedPrivateKey,
    message: &[u8],
    mut rng: impl rand::rand_core::RngCore + rand::rand_core::CryptoRng,
) -> Result<RedSignature, Error> {
    let mut transcript = [0u8; SIGN_TRANSCRIPT_LEN];
    rng.fill_bytes(&mut transcript);
    sign_with_transcript(private, message, &transcript)
}

/// Test-only deterministic signing with an explicit 80-byte transcript.
///
/// Production code must use [`sign`] with a secure RNG.
pub fn sign_with_transcript(
    private: &BlindedPrivateKey,
    message: &[u8],
    transcript: &[u8; SIGN_TRANSCRIPT_LEN],
) -> Result<RedSignature, Error> {
    if message.len() > MAX_MESSAGE_LEN {
        return Err(Error::InvalidData);
    }
    let scalar =
        Scalar::from_canonical_bytes(*private.as_bytes()).expect("blinded scalar is canonical");
    let vk = EdwardsPoint::mul_base(&scalar);
    let vk_bytes = vk.compress().to_bytes();

    let r = h_star(transcript, &vk_bytes, message);
    let big_r = EdwardsPoint::mul_base(&r);
    let r_bytes = big_r.compress().to_bytes();
    let c = h_star(&r_bytes, &vk_bytes, message);
    let signature_scalar = r + c * scalar;

    let mut bytes = [0u8; 64];
    bytes[..32].copy_from_slice(&r_bytes);
    bytes[32..].copy_from_slice(&signature_scalar.to_bytes());
    Ok(RedSignature(bytes))
}

/// Verify a Red25519 signature.
///
/// Fails closed on invalid `R`, non-canonical `S`, invalid/small-order
/// public keys, overlong messages, or equation mismatch. Uses the cofactor
/// equation `((-[S]B) + R + ([c]vk)) * cofactor == identity`.
pub fn verify(
    public: &BlindedPublicKey,
    message: &[u8],
    signature: &RedSignature,
) -> Result<(), Error> {
    if message.len() > MAX_MESSAGE_LEN {
        return Err(Error::InvalidData);
    }
    let vk = decompress_torsion_free(public.as_bytes())?;
    if vk == EdwardsPoint::identity() {
        return Err(Error::InvalidData);
    }

    let bytes = signature.as_bytes();
    let mut r_bytes = [0u8; 32];
    r_bytes.copy_from_slice(&bytes[..32]);
    let mut s_bytes = [0u8; 32];
    s_bytes.copy_from_slice(&bytes[32..]);
    let big_r = CompressedEdwardsY(r_bytes).decompress().ok_or(Error::InvalidData)?;
    let scalar_s = scalar_from_canonical(&s_bytes)?;

    let vk_bytes = vk.compress().to_bytes();
    let c = h_star(&r_bytes, &vk_bytes, message);
    let minus_s_base = -EdwardsPoint::mul_base(&scalar_s);
    let c_vk = c * vk;
    let check = minus_s_base + big_r + c_vk;
    if check.mul_by_cofactor() == EdwardsPoint::identity() {
        Ok(())
    } else {
        Err(Error::InvalidData)
    }
}

/// Blinded storage-key preimage: `sigtype_BE || blinded_pubkey`.
///
/// The 2-byte big-endian blinded type prefix (always 11) is included so the
/// hash input is unambiguous for the publication owner.
pub fn blinded_storage_key_preimage(blinded_pubkey: &[u8; 32]) -> [u8; 34] {
    let mut preimage = [0u8; 34];
    preimage[..2].copy_from_slice(&RED25519_SIGTYPE.to_be_bytes());
    preimage[2..].copy_from_slice(blinded_pubkey);
    preimage
}

/// Blinded DHT storage key: `SHA-256(sigtype_BE || blinded_pubkey)`.
///
/// Rotates daily because the blinded key rotates daily.
pub fn blinded_storage_key(blinded: &BlindedPublicKey) -> [u8; 32] {
    sha256(&blinded_storage_key_preimage(blinded.as_bytes()))
}

/// Decode a compressed Edwards point, requiring prime-order subgroup
/// membership (torsion-free).
fn decompress_torsion_free(bytes: &[u8; 32]) -> Result<EdwardsPoint, Error> {
    let point = CompressedEdwardsY(*bytes).decompress().ok_or(Error::InvalidData)?;
    if !point.is_torsion_free() {
        return Err(Error::InvalidData);
    }
    // Reject non-canonical encodings: re-encoding must round-trip exactly so
    // malleated high-bit encodings fail closed.
    if point.compress().as_bytes() != bytes {
        return Err(Error::InvalidData);
    }
    Ok(point)
}

/// Decode a canonical scalar, failing closed on values `>= L`.
fn scalar_from_canonical(bytes: &[u8; 32]) -> Result<Scalar, Error> {
    if bool::from(Scalar::from_canonical_bytes(*bytes).is_some()) {
        Ok(Scalar::from_canonical_bytes(*bytes).expect("checked canonical"))
    } else {
        Err(Error::InvalidData)
    }
}

/// `HStar(prefix1, prefix2, m) = SHA-512(domain || prefix1 || prefix2 ||
/// len_u16(m) || m) mod L` with little-endian length.
fn h_star(prefix1: &[u8], prefix2: &[u8], message: &[u8]) -> Scalar {
    debug_assert!(message.len() <= MAX_MESSAGE_LEN);
    let mut hasher = sha2::Sha512::new();
    hasher.update(RED25519_HASH_DOMAIN);
    hasher.update(prefix1);
    hasher.update(prefix2);
    let len = message.len() as u16;
    hasher.update([len as u8, (len >> 8) as u8]);
    hasher.update(message);
    let digest = hasher.finalize();
    let mut wide = [0u8; 64];
    wide.copy_from_slice(digest.as_ref());
    Scalar::from_bytes_mod_order_wide(&wide)
}

/// Single-shot SHA-256.
fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// RFC-5869 HKDF-SHA256 with 64-byte output.
///
/// Composed from the existing `hmac`/`sha2` primitives; no new KDF crate is
/// introduced. Intermediate PRK/OKM material is zeroized.
fn hkdf_sha256_64(salt: &[u8; 32], ikm: &[u8], info: &[u8]) -> [u8; 64] {
    let mut prk = Zeroizing::new([0u8; 32]);
    {
        let mut mac =
            hmac::Hmac::<sha2::Sha256>::new_from_slice(salt).expect("HMAC accepts 32-byte salt");
        mac.update(ikm);
        prk.copy_from_slice(mac.finalize().into_bytes().as_ref());
    }

    let mut okm = [0u8; 64];
    let mut previous = Zeroizing::new([0u8; 32]);
    let mut previous_len = 0usize;
    for (index, chunk) in okm.chunks_mut(32).enumerate() {
        let mut mac =
            hmac::Hmac::<sha2::Sha256>::new_from_slice(prk.as_ref()).expect("HMAC accepts PRK");
        if previous_len > 0 {
            mac.update(&previous[..previous_len]);
        }
        mac.update(info);
        mac.update(&[(index + 1) as u8]);
        let output = mac.finalize().into_bytes();
        chunk.copy_from_slice(output.as_ref());
        previous.copy_from_slice(output.as_ref());
        previous_len = 32;
    }
    previous.zeroize();
    okm
}

#[cfg(test)]
mod tests {
    use super::*;

    // Red25519 specification test vector 1 (known-answer gate).
    const VECTOR1_EDSK: [u8; 32] = [0x01; 32];
    const VECTOR1_EDPK: [u8; 32] = [
        0x8a, 0x88, 0xe3, 0xdd, 0x74, 0x09, 0xf1, 0x95, 0xfd, 0x52, 0xdb, 0x2d, 0x3c, 0xba, 0x5d,
        0x72, 0xca, 0x67, 0x09, 0xbf, 0x1d, 0x94, 0x12, 0x1b, 0xf3, 0x74, 0x88, 0x01, 0xb4, 0x0f,
        0x6f, 0x5c,
    ];
    const VECTOR1_SK: [u8; 32] = [
        0x58, 0xe8, 0x6e, 0xfb, 0x75, 0xfa, 0x4e, 0x2c, 0x41, 0x0f, 0x46, 0xe1, 0x6d, 0xe9, 0xf6,
        0xac, 0xae, 0x1a, 0x17, 0x03, 0x52, 0x86, 0x51, 0xb6, 0x9b, 0xc1, 0x76, 0xc0, 0x88, 0xbe,
        0xf3, 0x6e,
    ];
    const VECTOR1_ALPHA: [u8; 32] = [
        0xae, 0x9b, 0xa9, 0xcb, 0xbc, 0x04, 0x7c, 0x44, 0x24, 0x48, 0xfc, 0xa7, 0xc9, 0xf4, 0xe2,
        0x88, 0xa2, 0x02, 0xed, 0x52, 0x0b, 0xfa, 0xd0, 0xc7, 0x84, 0xb7, 0x92, 0xb7, 0x77, 0x3c,
        0xee, 0x08,
    ];
    const VECTOR1_RSK: [u8; 32] = [
        0x8b, 0xb8, 0x5f, 0x3c, 0x7a, 0x49, 0x4a, 0x08, 0x89, 0x0d, 0x7d, 0x14, 0x21, 0x09, 0xc1,
        0xa3, 0x50, 0x1d, 0x04, 0x56, 0x5d, 0x80, 0x22, 0x7e, 0x20, 0x79, 0x09, 0x78, 0x00, 0xfb,
        0xe1, 0x07,
    ];
    const VECTOR1_RVK: [u8; 32] = [
        0x6f, 0xe1, 0x28, 0x73, 0x7b, 0x8e, 0x76, 0xfa, 0x66, 0x69, 0x8a, 0x74, 0x8b, 0x0d, 0xc0,
        0xa8, 0x91, 0x68, 0xdd, 0x8a, 0x06, 0x01, 0xc2, 0xb1, 0xc0, 0xb2, 0x68, 0x35, 0xd3, 0x23,
        0xe9, 0xb3,
    ];
    const VECTOR1_MSG: [u8; 32] = [0x02; 32];

    // Red25519 specification test vector 2 (second known-answer gate).
    const VECTOR2_EDSK: [u8; 32] = [0x02; 32];
    const VECTOR2_ALPHA: [u8; 32] = [
        0x98, 0xb6, 0x15, 0xd9, 0x02, 0x7e, 0x99, 0x6c, 0xc2, 0x79, 0x6c, 0x01, 0x9d, 0x9c, 0x8b,
        0xeb, 0x46, 0xaa, 0x7d, 0x2b, 0x6e, 0xea, 0x2e, 0x5d, 0x98, 0xeb, 0x29, 0xeb, 0x15, 0x84,
        0xc2, 0x03,
    ];
    const VECTOR2_RSK: [u8; 32] = [
        0x9f, 0xcf, 0xaa, 0x73, 0x48, 0x52, 0xca, 0x40, 0xb3, 0x81, 0x0e, 0xbe, 0xf5, 0x90, 0xe1,
        0x38, 0x51, 0x6e, 0x8c, 0xb4, 0xf4, 0xb1, 0xb6, 0xf0, 0x73, 0x09, 0x78, 0xde, 0x7f, 0x80,
        0x64, 0x02,
    ];
    const VECTOR2_RVK: [u8; 32] = [
        0x52, 0x7e, 0x12, 0x10, 0x90, 0x15, 0x84, 0x19, 0x60, 0x9e, 0x4a, 0x0d, 0x8d, 0xe6, 0xf7,
        0xd3, 0x27, 0x1b, 0x35, 0x3a, 0x8c, 0xd0, 0xb8, 0x17, 0x2f, 0xe4, 0x14, 0x68, 0xea, 0x1e,
        0x91, 0x77,
    ];

    fn alpha_fixture() -> Alpha {
        Alpha::from_bytes(&VECTOR1_ALPHA).expect("vector alpha is canonical")
    }

    #[test]
    fn ed25519_seed_conversion_matches_spec_vector1() {
        assert_eq!(convert_ed25519_seed(&VECTOR1_EDSK), VECTOR1_SK);
    }

    #[test]
    fn blinded_private_scalar_matches_spec_vector1() {
        let alpha = alpha_fixture();
        let blinded = blind_private_key_ed25519(&VECTOR1_EDSK, &alpha);
        assert_eq!(blinded.as_bytes(), &VECTOR1_RSK);
    }

    #[test]
    fn blinded_private_scalar_matches_spec_vector2() {
        let alpha = Alpha::from_bytes(&VECTOR2_ALPHA).expect("vector alpha is canonical");
        let blinded = blind_private_key_ed25519(&VECTOR2_EDSK, &alpha);
        assert_eq!(blinded.as_bytes(), &VECTOR2_RSK);
    }

    #[test]
    fn blinded_public_key_matches_spec_vectors() {
        let alpha = alpha_fixture();
        let blinded = blind_public_key(&VECTOR1_EDPK, &alpha).expect("valid blinding");
        assert_eq!(blinded.as_bytes(), &VECTOR1_RVK);

        let alpha2 = Alpha::from_bytes(&VECTOR2_ALPHA).expect("vector alpha is canonical");
        let edpk2 = [
            0x81, 0x39, 0x77, 0x0e, 0xa8, 0x7d, 0x17, 0x5f, 0x56, 0xa3, 0x54, 0x66, 0xc3, 0x4c,
            0x7e, 0xcc, 0xcb, 0x8d, 0x8a, 0x91, 0xb4, 0xee, 0x37, 0xa2, 0x5d, 0xf6, 0x0f, 0x5b,
            0x8f, 0xc9, 0xb3, 0x94,
        ];
        let blinded2 = blind_public_key(&edpk2, &alpha2).expect("valid blinding");
        assert_eq!(blinded2.as_bytes(), &VECTOR2_RVK);
    }

    #[test]
    fn blinded_private_public_agreement() {
        let alpha = alpha_fixture();
        let blinded_private = blind_private_key_ed25519(&VECTOR1_EDSK, &alpha);
        let derived = derive_public(&blinded_private);
        let blinded_public = blind_public_key(&VECTOR1_EDPK, &alpha).expect("valid blinding");
        assert_eq!(derived, blinded_public);
        assert_eq!(derived.as_bytes(), &VECTOR1_RVK);
    }

    #[test]
    fn red25519_sign_verify_roundtrip() {
        let private = BlindedPrivateKey::from_bytes(&VECTOR1_RSK).expect("canonical rsk");
        let public = BlindedPublicKey::from_bytes(&VECTOR1_RVK).expect("valid rvk");
        let transcript = [0x42u8; SIGN_TRANSCRIPT_LEN];
        let signature = sign_with_transcript(&private, &VECTOR1_MSG, &transcript)
            .expect("deterministic signing succeeds");
        verify(&public, &VECTOR1_MSG, &signature).expect("own signature verifies");
    }

    #[test]
    fn red25519_randomized_signatures_differ() {
        let private = BlindedPrivateKey::from_bytes(&VECTOR1_RSK).expect("canonical rsk");
        let first = sign_with_transcript(&private, &VECTOR1_MSG, &[0x11u8; 80])
            .expect("first signing succeeds");
        let second = sign_with_transcript(&private, &VECTOR1_MSG, &[0x22u8; 80])
            .expect("second signing succeeds");
        assert_ne!(first.as_bytes(), second.as_bytes());
        let public = BlindedPublicKey::from_bytes(&VECTOR1_RVK).expect("valid rvk");
        verify(&public, &VECTOR1_MSG, &first).expect("first verifies");
        verify(&public, &VECTOR1_MSG, &second).expect("second verifies");
    }

    #[test]
    fn red25519_wrong_key_and_message_rejected() {
        let private = BlindedPrivateKey::from_bytes(&VECTOR1_RSK).expect("canonical rsk");
        let public = BlindedPublicKey::from_bytes(&VECTOR1_RVK).expect("valid rvk");
        let signature =
            sign_with_transcript(&private, &VECTOR1_MSG, &[0x33u8; 80]).expect("signing succeeds");

        let mut wrong_message = VECTOR1_MSG;
        wrong_message[0] ^= 0xff;
        assert!(verify(&public, &wrong_message, &signature).is_err());

        let other_public =
            BlindedPublicKey::from_bytes(&VECTOR2_RVK).expect("second vector key valid");
        assert!(verify(&other_public, &VECTOR1_MSG, &signature).is_err());

        let mut tampered = *signature.as_bytes();
        tampered[63] ^= 0x01;
        // Tampering the top bit keeps S canonical in this case; otherwise the
        // tampered encoding may be non-canonical and also fail at parse time.
        // Either outcome is a closed rejection.
        let tampered_result = RedSignature::from_bytes(&tampered)
            .map(|candidate| verify(&public, &VECTOR1_MSG, &candidate));
        assert!(tampered_result.is_err() || tampered_result.expect("parsed").is_err());
    }

    #[test]
    fn alpha_derivation_is_deterministic_and_secret_sensitive() {
        let day = *b"20260909";
        let first = generate_alpha(&VECTOR1_EDPK, 7, 11, &day, b"").expect("alpha derives");
        let second = generate_alpha(&VECTOR1_EDPK, 7, 11, &day, b"").expect("alpha re-derives");
        assert_eq!(first.as_bytes(), second.as_bytes());

        let with_secret =
            generate_alpha(&VECTOR1_EDPK, 7, 11, &day, b"lookup-secret").expect("secret derives");
        assert_ne!(first.as_bytes(), with_secret.as_bytes());

        let other_day =
            generate_alpha(&VECTOR1_EDPK, 7, 11, b"20260910", b"").expect("next-day alpha derives");
        assert_ne!(first.as_bytes(), other_day.as_bytes());
    }

    #[test]
    fn alpha_derivation_day_rollover_changes_blinded_material() {
        let day = *b"20260909";
        let next = *b"20260910";
        let alpha = generate_alpha(&VECTOR1_EDPK, 7, 11, &day, b"").expect("alpha derives");
        let alpha_next =
            generate_alpha(&VECTOR1_EDPK, 7, 11, &next, b"").expect("next alpha derives");
        let blinded = blind_public_key(&VECTOR1_EDPK, &alpha).expect("blinding succeeds");
        let blinded_next =
            blind_public_key(&VECTOR1_EDPK, &alpha_next).expect("next blinding succeeds");
        assert_ne!(blinded, blinded_next);
        assert_ne!(
            blinded_storage_key(&blinded),
            blinded_storage_key(&blinded_next)
        );
    }

    #[test]
    fn alpha_derivation_rejects_bad_inputs() {
        let day = *b"20260909";
        assert!(generate_alpha(&VECTOR1_EDPK, 7, 12, &day, b"").is_err());
        assert!(generate_alpha(&VECTOR1_EDPK, 6, 11, &day, b"").is_err());
        assert!(generate_alpha(&VECTOR1_EDPK, 7, 11, b"2026-909", b"").is_err());
        assert!(generate_alpha(&VECTOR1_EDPK, 7, 11, b"20261301", b"").is_err());
        assert!(generate_alpha(&VECTOR1_EDPK, 7, 11, &day, &[0xff, 0xfe]).is_err());
        let mut too_long = [b'a'; 65];
        too_long[0] = b'b';
        assert!(generate_alpha(&VECTOR1_EDPK, 7, 11, &day, &too_long).is_err());
    }

    #[test]
    fn blinded_storage_key_is_sha256_of_typed_preimage() {
        let blinded = BlindedPublicKey::from_bytes(&VECTOR1_RVK).expect("valid rvk");
        let preimage = blinded_storage_key_preimage(blinded.as_bytes());
        assert_eq!(&preimage[..2], &[0x00, 0x0b]);
        assert_eq!(&preimage[2..], &VECTOR1_RVK);
        let expected = {
            use sha2::Digest as _;
            let mut hasher = sha2::Sha256::new();
            hasher.update(preimage);
            let digest = hasher.finalize();
            let mut out = [0u8; 32];
            out.copy_from_slice(&digest);
            out
        };
        assert_eq!(blinded_storage_key(&blinded), expected);
    }

    #[test]
    fn malformed_points_and_scalars_fail_closed() {
        let alpha = alpha_fixture();
        assert!(blind_public_key(&[0u8; 32], &alpha).is_err());
        assert!(blind_public_key(&[0xffu8; 32], &alpha).is_err());

        let low_order = CompressedEdwardsY([0u8; 32]).decompress();
        let _ = low_order;

        // Non-canonical scalar (>= L) must fail.
        let non_canonical = [0xeeu8; 32];
        assert!(BlindedPrivateKey::from_bytes(&non_canonical).is_err());
        assert!(Alpha::from_bytes(&non_canonical).is_err());
        assert!(blind_private_key_red25519(&non_canonical, &alpha).is_err());

        // Identity public key must fail.
        let identity = EdwardsPoint::identity().compress().to_bytes();
        assert!(BlindedPublicKey::from_bytes(&identity).is_err());
        assert!(blind_public_key(&identity, &alpha).is_err());

        // Non-canonical S must fail at parse time.
        let mut bad_sig = [0u8; 64];
        bad_sig[..32].copy_from_slice(&VECTOR1_RVK);
        bad_sig[32..].copy_from_slice(&non_canonical);
        assert!(RedSignature::from_bytes(&bad_sig).is_err());

        // Overlong messages fail before allocation-heavy hashing.
        let private = BlindedPrivateKey::from_bytes(&VECTOR1_RSK).expect("canonical rsk");
        let public = BlindedPublicKey::from_bytes(&VECTOR1_RVK).expect("valid rvk");
        let long = [0u8; MAX_MESSAGE_LEN + 1];
        assert!(sign_with_transcript(&private, &long, &[0u8; 80]).is_err());
        let signature =
            sign_with_transcript(&private, &VECTOR1_MSG, &[0u8; 80]).expect("signing succeeds");
        assert!(verify(&public, &long, &signature).is_err());
    }

    #[test]
    fn ordinary_ed25519_behavior_unchanged() {
        use ed25519_dalek::Signer as _;
        let seed = [0x01u8; 32];
        let signing = ed25519_dalek::SigningKey::from_bytes(&seed);
        let message = b"ordinary ed25519 regression";
        let signature = signing.sign(message);
        signing
            .verifying_key()
            .verify_strict(message, &signature)
            .expect("ordinary ed25519 still verifies");
    }
}
