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

//! Neutral modern type-5 encrypted LeaseSet2 crypto helper.
//!
//! Implements the no-client-auth, no-lookup-secret subset of the current
//! encrypted LeaseSet construction on top of the closed blinding primitive:
//! credential/subcredential derivation, exact 44-byte schedules for the two
//! nested layers, no-auth layer encryption/decryption for deterministic
//! self-validation, secure salts through caller-provided randomness, UTC
//! epoch-day conversion, and a zeroizing type-7 seed handoff.
//!
//! No lookup-secret contribution, no per-client authorization, no persistent
//! signature-type registry, and no generic key-derivation API are provided
//! here. Successor work extends this module through its exact owner.

use crate::{
    crypto::{chachapoly::ChaCha, red25519},
    error::Error,
};

use hmac::Mac;
use rand::rand_core::{CryptoRng, RngCore};
use sha2::Digest;
use zeroize::{Zeroize, Zeroizing};

use alloc::vec::Vec;

/// Unblinded type-7 code consumed as derivation input.
pub const UNBLINDED_SIGTYPE: u16 = 7;

/// Blinded type-11 code produced for publication.
pub const BLINDED_SIGTYPE: u16 = 11;

/// DatabaseStore type byte for the modern encrypted object.
pub const ENCRYPTED_LEASESET_TYPE: u8 = 5;

/// Inner layer type byte for an ordinary LeaseSet2 payload.
pub const INNER_LEASESET2_TYPE: u8 = 3;

/// No-auth middle-layer flags byte.
pub const NO_AUTH_LAYER1_FLAGS: u8 = 0;

/// Key-derivation label for the outer layer.
const L1_INFO: &[u8] = b"ELS2_L1K";

/// Key-derivation label for the inner layer.
const L2_INFO: &[u8] = b"ELS2_L2K";

/// Salt length in bytes.
const SALT_LEN: usize = 32;

/// Maximum accepted inner LeaseSet2 payload for wrapping.
///
/// Bounds allocation before any key derivation or cipher work. Ordinary
/// payloads are on the order of one kilobyte; this ceiling leaves ample
/// headroom while keeping floodfill-facing work bounded.
const MAX_INNER_PAYLOAD_LEN: usize = 32_768;

/// Maximum accepted outer ciphertext length.
///
/// Covers the two 32-byte salts, two single-byte layer markers, and the
/// bounded inner payload. The signed outer preimage additionally stays far
/// below the Red25519 message ceiling.
const MAX_OUTER_CIPHERTEXT_LEN: usize = MAX_INNER_PAYLOAD_LEN + 128;

/// Seconds per UTC day.
const SECS_PER_DAY: u64 = 86_400;

/// Type-7 signing seed handoff for daily blinded publication.
///
/// Secret material: never `Debug`- or display-formattable and zeroized on
/// drop. The unblinded public key travels alongside this seed through the
/// narrow publication seam; this type never derives network identity by
/// itself.
#[derive(Clone)]
pub struct SigningSeed(Zeroizing<[u8; 32]>);

impl SigningSeed {
    /// Wrap a 32-byte type-7 seed.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(Zeroizing::new(bytes))
    }

    /// Return the seed bytes for one-shot derivation.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Derive the credential for an unblinded signing public key.
///
/// `credential = SHA-256("credential" || A || 0x0007 || 0x000b)`.
pub fn credential(unblinded_pubkey: &[u8; 32]) -> [u8; 32] {
    let mut keydata = [0u8; 36];
    keydata[..32].copy_from_slice(unblinded_pubkey);
    keydata[32..34].copy_from_slice(&UNBLINDED_SIGTYPE.to_be_bytes());
    keydata[34..36].copy_from_slice(&BLINDED_SIGTYPE.to_be_bytes());

    let mut hasher = sha2::Sha256::new();
    hasher.update(b"credential");
    hasher.update(keydata);
    hasher.finalize().into()
}

/// Derive the subcredential binding a blinded key to its unblinded source.
///
/// `subcredential = SHA-256("subcredential" || credential || A')`.
pub fn subcredential(unblinded_pubkey: &[u8; 32], blinded_pubkey: &[u8; 32]) -> [u8; 32] {
    let cred = credential(unblinded_pubkey);
    let mut hasher = sha2::Sha256::new();
    hasher.update(b"subcredential");
    hasher.update(cred);
    hasher.update(blinded_pubkey);
    hasher.finalize().into()
}

/// RFC-5869 HKDF-SHA256 with exact 44-byte output.
///
/// Composed from the existing `hmac`/`sha2` primitives; no new KDF crate is
/// introduced. Intermediate chaining material is zeroized.
fn hkdf_sha256_44(salt: &[u8; 32], ikm: &[u8], info: &[u8]) -> [u8; 44] {
    let mut prk = Zeroizing::new([0u8; 32]);
    {
        let mut mac =
            hmac::Hmac::<sha2::Sha256>::new_from_slice(salt).expect("HMAC accepts 32-byte salt");
        mac.update(ikm);
        prk.copy_from_slice(mac.finalize().into_bytes().as_ref());
    }

    let mut okm = [0u8; 44];
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
        chunk.copy_from_slice(&output[..chunk.len()]);
        previous.copy_from_slice(output.as_ref());
        previous_len = 32;
    }
    previous.zeroize();
    okm
}

/// Derive the ChaCha key and IV for one layer.
///
/// Input is `subcredential || published_BE`; `info` selects the layer
/// schedule (`ELS2_L1K` outer, `ELS2_L2K` inner).
fn derive_layer_keys(
    salt: &[u8; 32],
    subcredential: &[u8; 32],
    published: u32,
    info: &[u8],
) -> ([u8; 32], [u8; 12]) {
    let mut input = Zeroizing::new([0u8; 36]);
    input[..32].copy_from_slice(subcredential);
    input[32..36].copy_from_slice(&published.to_be_bytes());
    let mut okm = hkdf_sha256_44(salt, &input[..], info);
    let mut key = [0u8; 32];
    let mut iv = [0u8; 12];
    key.copy_from_slice(&okm[..32]);
    iv.copy_from_slice(&okm[32..44]);
    okm.zeroize();
    (key, iv)
}

/// Apply ChaCha20 with the existing counter-1 helper in place.
fn chacha_apply(key: &[u8; 32], iv: &[u8; 12], data: &mut [u8]) {
    ChaCha::with_iv(*key, *iv).encrypt_ref(data);
}

/// Encrypt the inner layer with explicit salts (deterministic for tests).
///
/// Plaintext is `0x03 || inner_ls2_bytes`; output is
/// `innerSalt || ChaCha(innerKey, innerIV, plaintext)`.
pub fn encrypt_inner_with_salts(
    subcredential: &[u8; 32],
    published: u32,
    inner_ls2_bytes: &[u8],
    inner_salt: &[u8; 32],
) -> Result<Vec<u8>, Error> {
    if inner_ls2_bytes.is_empty() || inner_ls2_bytes.len() > MAX_INNER_PAYLOAD_LEN {
        return Err(Error::InvalidData);
    }
    let (key, iv) = derive_layer_keys(inner_salt, subcredential, published, L2_INFO);
    let mut plaintext = Vec::with_capacity(1 + inner_ls2_bytes.len());
    plaintext.push(INNER_LEASESET2_TYPE);
    plaintext.extend_from_slice(inner_ls2_bytes);
    chacha_apply(&key, &iv, &mut plaintext);
    let mut out = Vec::with_capacity(SALT_LEN + plaintext.len());
    out.extend_from_slice(inner_salt);
    out.extend_from_slice(&plaintext);
    Ok(out)
}

/// Encrypt the outer no-auth layer with an explicit salt.
///
/// Plaintext is `0x00 || inner_ciphertext`; output is
/// `outerSalt || ChaCha(outerKey, outerIV, plaintext)`.
pub fn encrypt_outer_with_salts(
    subcredential: &[u8; 32],
    published: u32,
    inner_ciphertext: &[u8],
    outer_salt: &[u8; 32],
) -> Result<Vec<u8>, Error> {
    if inner_ciphertext.is_empty()
        || inner_ciphertext.len() > MAX_OUTER_CIPHERTEXT_LEN.saturating_sub(SALT_LEN + 1)
    {
        return Err(Error::InvalidData);
    }
    let (key, iv) = derive_layer_keys(outer_salt, subcredential, published, L1_INFO);
    let mut plaintext = Vec::with_capacity(1 + inner_ciphertext.len());
    plaintext.push(NO_AUTH_LAYER1_FLAGS);
    plaintext.extend_from_slice(inner_ciphertext);
    chacha_apply(&key, &iv, &mut plaintext);
    let mut out = Vec::with_capacity(SALT_LEN + plaintext.len());
    out.extend_from_slice(outer_salt);
    out.extend_from_slice(&plaintext);
    Ok(out)
}

/// Encrypt both nested no-auth layers with explicit salts.
///
/// Returns the layer-0 `outerCiphertext` field content for the given
/// already-signed ordinary inner LeaseSet2 bytes.
pub fn encrypt_no_auth_with_salts(
    subcredential: &[u8; 32],
    published: u32,
    inner_ls2_bytes: &[u8],
    inner_salt: &[u8; 32],
    outer_salt: &[u8; 32],
) -> Result<Vec<u8>, Error> {
    let inner = encrypt_inner_with_salts(subcredential, published, inner_ls2_bytes, inner_salt)?;
    encrypt_outer_with_salts(subcredential, published, &inner, outer_salt)
}

/// Encrypt the inner layer with fresh salts.
pub fn encrypt_inner(
    subcredential: &[u8; 32],
    published: u32,
    inner_ls2_bytes: &[u8],
    rng: impl RngCore + CryptoRng,
) -> Result<Vec<u8>, Error> {
    let mut rng = rng;
    let mut salt = [0u8; SALT_LEN];
    rng.fill_bytes(&mut salt);
    encrypt_inner_with_salts(subcredential, published, inner_ls2_bytes, &salt)
}

/// Encrypt the outer no-auth layer with a fresh salt.
pub fn encrypt_outer(
    subcredential: &[u8; 32],
    published: u32,
    inner_ciphertext: &[u8],
    rng: impl RngCore + CryptoRng,
) -> Result<Vec<u8>, Error> {
    let mut rng = rng;
    let mut salt = [0u8; SALT_LEN];
    rng.fill_bytes(&mut salt);
    encrypt_outer_with_salts(subcredential, published, inner_ciphertext, &salt)
}

/// Encrypt both nested no-auth layers with fresh salts.
pub fn encrypt_no_auth(
    subcredential: &[u8; 32],
    published: u32,
    inner_ls2_bytes: &[u8],
    mut rng: impl RngCore + CryptoRng,
) -> Result<Vec<u8>, Error> {
    if inner_ls2_bytes.is_empty() || inner_ls2_bytes.len() > MAX_INNER_PAYLOAD_LEN {
        return Err(Error::InvalidData);
    }
    let mut inner_salt = [0u8; SALT_LEN];
    let mut outer_salt = [0u8; SALT_LEN];
    rng.fill_bytes(&mut inner_salt);
    rng.fill_bytes(&mut outer_salt);
    encrypt_no_auth_with_salts(
        subcredential,
        published,
        inner_ls2_bytes,
        &inner_salt,
        &outer_salt,
    )
}

/// Decrypt the outer no-auth layer, returning the inner ciphertext.
///
/// Rejects short inputs and any flags byte other than `0x00`.
pub fn decrypt_outer(
    subcredential: &[u8; 32],
    published: u32,
    outer_ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    if outer_ciphertext.len() < SALT_LEN + 1 || outer_ciphertext.len() > MAX_OUTER_CIPHERTEXT_LEN {
        return Err(Error::InvalidData);
    }
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&outer_ciphertext[..SALT_LEN]);
    let (key, iv) = derive_layer_keys(&salt, subcredential, published, L1_INFO);
    let mut plaintext = outer_ciphertext[SALT_LEN..].to_vec();
    chacha_apply(&key, &iv, &mut plaintext);
    if plaintext.is_empty() || plaintext[0] != NO_AUTH_LAYER1_FLAGS {
        plaintext.zeroize();
        return Err(Error::InvalidData);
    }
    Ok(plaintext[1..].to_vec())
}

/// Decrypt the inner layer, returning the ordinary inner LeaseSet2 bytes.
///
/// Rejects short inputs and any inner type byte other than `0x03`.
pub fn decrypt_inner(
    subcredential: &[u8; 32],
    published: u32,
    inner_ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    if inner_ciphertext.len() < SALT_LEN + 1 || inner_ciphertext.len() > MAX_OUTER_CIPHERTEXT_LEN {
        return Err(Error::InvalidData);
    }
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&inner_ciphertext[..SALT_LEN]);
    let (key, iv) = derive_layer_keys(&salt, subcredential, published, L2_INFO);
    let mut plaintext = inner_ciphertext[SALT_LEN..].to_vec();
    chacha_apply(&key, &iv, &mut plaintext);
    if plaintext.is_empty() || plaintext[0] != INNER_LEASESET2_TYPE {
        plaintext.zeroize();
        return Err(Error::InvalidData);
    }
    Ok(plaintext[1..].to_vec())
}

/// Decrypt both nested no-auth layers, returning the inner LeaseSet2 bytes.
pub fn decrypt_no_auth(
    subcredential: &[u8; 32],
    published: u32,
    outer_ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    let inner = decrypt_outer(subcredential, published, outer_ciphertext)?;
    let plain = decrypt_inner(subcredential, published, &inner)?;
    Ok(plain)
}

/// Format the 8-byte ASCII `YYYYMMDD` day string for epoch seconds.
///
/// Pure integer Gregorian conversion; no wall-clock or calendar dependency.
pub fn day_string_from_epoch_secs(secs: u64) -> [u8; 8] {
    let (year, month, day) = civil_from_days((secs / SECS_PER_DAY) as i64);
    let mut out = [b'0'; 8];
    out[0] = b'0' + (year / 1000) as u8;
    out[1] = b'0' + ((year / 100) % 10) as u8;
    out[2] = b'0' + ((year / 10) % 10) as u8;
    out[3] = b'0' + (year % 10) as u8;
    out[4] = b'0' + (month / 10) as u8;
    out[5] = b'0' + (month % 10) as u8;
    out[6] = b'0' + (day / 10) as u8;
    out[7] = b'0' + (day % 10) as u8;
    out
}

/// Return the epoch seconds of the next UTC-day boundary after `secs`.
pub fn next_day_boundary_secs(secs: u64) -> u64 {
    (secs / SECS_PER_DAY).saturating_add(1).saturating_mul(SECS_PER_DAY)
}

/// Convert days since the Unix epoch to calendar components.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 {
        y += 1;
    }
    (y, m, d)
}

/// Derive the current-day blinded publication material for an empty secret.
///
/// Returns the daily blinding scalar, blinded public key, blinded private
/// scalar, and blinded storage key. Any malformed seed or public input fails
/// closed before publication.
pub fn blinded_day_material(
    seed: &SigningSeed,
    unblinded_pubkey: &[u8; 32],
    day: &[u8; 8],
) -> Result<
    (
        red25519::Alpha,
        red25519::BlindedPublicKey,
        red25519::BlindedPrivateKey,
        [u8; 32],
    ),
    Error,
> {
    let alpha = red25519::generate_alpha(
        unblinded_pubkey,
        UNBLINDED_SIGTYPE,
        BLINDED_SIGTYPE,
        day,
        b"",
    )?;
    let blinded_public = red25519::blind_public_key(unblinded_pubkey, &alpha)?;
    let blinded_private = red25519::blind_private_key_ed25519(seed.as_bytes(), &alpha);
    if red25519::derive_public(&blinded_private) != blinded_public {
        return Err(Error::InvalidData);
    }
    let storage_key = red25519::blinded_storage_key(&blinded_public);
    Ok((alpha, blinded_public, blinded_private, storage_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: [u8; 32] = [0x01; 32];
    const UNBLINDED: [u8; 32] = [
        0x8a, 0x88, 0xe3, 0xdd, 0x74, 0x09, 0xf1, 0x95, 0xfd, 0x52, 0xdb, 0x2d, 0x3c, 0xba, 0x5d,
        0x72, 0xca, 0x67, 0x09, 0xbf, 0x1d, 0x94, 0x12, 0x1b, 0xf3, 0x74, 0x88, 0x01, 0xb4, 0x0f,
        0x6f, 0x5c,
    ];
    const DAY: [u8; 8] = *b"20260909";
    const PUBLISHED: u32 = 1_788_000_000;
    const INNER: [u8; 16] = [
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa,
        0xbb,
    ];
    const INNER_SALT: [u8; 32] = [0x11; 32];
    const OUTER_SALT: [u8; 32] = [0x22; 32];

    fn fixture_subcredential() -> ([u8; 32], [u8; 32]) {
        let seed = SigningSeed::from_bytes(SEED);
        let (_, blinded, _, _) =
            blinded_day_material(&seed, &UNBLINDED, &DAY).expect("fixture day material derives");
        let sub = subcredential(&UNBLINDED, blinded.as_bytes());
        (sub, *blinded.as_bytes())
    }

    #[test]
    fn credential_and_subcredential_known_answer() {
        let cred = credential(&UNBLINDED);
        let expected_cred = {
            let mut keydata = [0u8; 36];
            keydata[..32].copy_from_slice(&UNBLINDED);
            keydata[32..34].copy_from_slice(&7u16.to_be_bytes());
            keydata[34..36].copy_from_slice(&11u16.to_be_bytes());
            let mut hasher = sha2::Sha256::new();
            hasher.update(b"credential");
            hasher.update(keydata);
            let digest = hasher.finalize();
            let mut out = [0u8; 32];
            out.copy_from_slice(&digest);
            out
        };
        assert_eq!(cred, expected_cred);

        let (_, blinded) = fixture_subcredential();
        let sub = subcredential(&UNBLINDED, &blinded);
        let expected_sub = {
            let mut hasher = sha2::Sha256::new();
            hasher.update(b"subcredential");
            hasher.update(cred);
            hasher.update(blinded);
            let digest = hasher.finalize();
            let mut out = [0u8; 32];
            out.copy_from_slice(&digest);
            out
        };
        assert_eq!(sub, expected_sub);
        assert_ne!(cred, sub);
    }

    #[test]
    fn layer_key_schedules_are_independent_and_pinned() {
        let (sub, _) = fixture_subcredential();
        let salt = [0x33; 32];
        let (outer_key, outer_iv) = derive_layer_keys(&salt, &sub, PUBLISHED, L1_INFO);
        let (inner_key, inner_iv) = derive_layer_keys(&salt, &sub, PUBLISHED, L2_INFO);
        assert_ne!(outer_key, inner_key);
        assert_ne!(outer_iv, inner_iv);

        let expected = hkdf_sha256_44(
            &salt,
            &{
                let mut input = [0u8; 36];
                input[..32].copy_from_slice(&sub);
                input[32..36].copy_from_slice(&PUBLISHED.to_be_bytes());
                input
            },
            L1_INFO,
        );
        assert_eq!(&outer_key[..], &expected[..32]);
        assert_eq!(&outer_iv[..], &expected[32..44]);

        let mut other_published = derive_layer_keys(&salt, &sub, PUBLISHED + 1, L1_INFO);
        assert_ne!(outer_key, other_published.0);
        other_published.0.zeroize();
    }

    #[test]
    fn no_auth_layers_round_trip_with_fixed_salts() {
        let (sub, _) = fixture_subcredential();
        let outer = encrypt_no_auth_with_salts(&sub, PUBLISHED, &INNER, &INNER_SALT, &OUTER_SALT)
            .expect("deterministic encryption succeeds");
        assert_eq!(&outer[..32], &OUTER_SALT);
        assert_eq!(outer.len(), 32 + 1 + 32 + 1 + INNER.len());

        let recovered = decrypt_no_auth(&sub, PUBLISHED, &outer).expect("decrypts");
        assert_eq!(recovered, INNER);

        let again = encrypt_no_auth_with_salts(&sub, PUBLISHED, &INNER, &INNER_SALT, &OUTER_SALT)
            .expect("deterministic repeat succeeds");
        assert_eq!(outer, again);
    }

    #[test]
    fn layer_flags_and_type_gates_reject() {
        let (sub, _) = fixture_subcredential();
        let outer = encrypt_no_auth_with_salts(&sub, PUBLISHED, &INNER, &INNER_SALT, &OUTER_SALT)
            .expect("encrypts");

        assert!(decrypt_outer(&sub, PUBLISHED + 1, &outer).is_err());
        assert!(decrypt_outer(&sub, PUBLISHED, &outer[..10]).is_err());
        assert!(decrypt_outer(&sub, PUBLISHED, &[]).is_err());

        let mut tampered = outer.clone();
        tampered[32] ^= 0xff;
        assert!(decrypt_no_auth(&sub, PUBLISHED, &tampered).is_err());

        let inner = decrypt_outer(&sub, PUBLISHED, &outer).expect("outer decrypts");
        let mut bad_inner = inner.clone();
        bad_inner[32] ^= 0xff;
        let mut bad_outer_plain = Vec::with_capacity(1 + bad_inner.len());
        bad_outer_plain.push(NO_AUTH_LAYER1_FLAGS);
        bad_outer_plain.extend_from_slice(&bad_inner);
        let (key, iv) = derive_layer_keys(&OUTER_SALT, &sub, PUBLISHED, L1_INFO);
        chacha_apply(&key, &iv, &mut bad_outer_plain);
        let mut bad_outer = OUTER_SALT.to_vec();
        bad_outer.extend_from_slice(&bad_outer_plain);
        assert!(decrypt_no_auth(&sub, PUBLISHED, &bad_outer).is_err());
    }

    #[test]
    fn empty_and_oversize_inputs_fail_before_crypto() {
        let (sub, _) = fixture_subcredential();
        assert!(encrypt_inner_with_salts(&sub, PUBLISHED, &[], &INNER_SALT).is_err());
        assert!(encrypt_outer_with_salts(&sub, PUBLISHED, &[], &OUTER_SALT).is_err());
        let big = [0u8; MAX_INNER_PAYLOAD_LEN + 1];
        assert!(encrypt_inner_with_salts(&sub, PUBLISHED, &big, &INNER_SALT).is_err());
    }

    #[test]
    fn utc_day_conversion_known_answers() {
        assert_eq!(day_string_from_epoch_secs(0), *b"19700101");
        assert_eq!(day_string_from_epoch_secs(1_788_000_000), *b"20260829");
        assert_eq!(day_string_from_epoch_secs(1_789_123_456), *b"20260911");
        assert_eq!(next_day_boundary_secs(0), 86_400);
        assert_eq!(next_day_boundary_secs(86_399), 86_400);
        assert_eq!(next_day_boundary_secs(86_400), 172_800);
        assert_eq!(
            day_string_from_epoch_secs(next_day_boundary_secs(1_788_000_000)),
            *b"20260830"
        );
        // Leap day 2024-02-29 00:00:00 UTC.
        assert_eq!(day_string_from_epoch_secs(1_709_164_800), *b"20240229");
        // Year boundary 2026-01-01.
        assert_eq!(day_string_from_epoch_secs(1_767_225_600), *b"20260101");
    }

    #[test]
    fn blinded_day_material_matches_blinding_primitive() {
        let seed = SigningSeed::from_bytes(SEED);
        let (alpha, blinded, private, storage) =
            blinded_day_material(&seed, &UNBLINDED, &DAY).expect("derives");
        let direct_alpha =
            red25519::generate_alpha(&UNBLINDED, 7, 11, &DAY, b"").expect("direct alpha derives");
        assert_eq!(alpha.as_bytes(), direct_alpha.as_bytes());
        assert_eq!(
            red25519::derive_public(&private),
            blinded,
            "private/public agreement holds"
        );
        assert_eq!(storage, red25519::blinded_storage_key(&blinded));

        let next = blinded_day_material(&seed, &UNBLINDED, b"20260910").expect("next day derives");
        assert_ne!(blinded.as_bytes(), next.1.as_bytes());
        assert_ne!(storage, next.3);
    }
}
