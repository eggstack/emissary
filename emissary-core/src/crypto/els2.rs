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
//! Implements the no-client-auth subset plus the standard PSK
//! client-authorization subset of the current encrypted LeaseSet
//! construction on top of the closed blinding primitive:
//! credential/subcredential derivation, exact 44-byte schedules for the two
//! nested layers, no-auth layer encryption/decryption for deterministic
//! self-validation, PSK layer-1 construction with fresh auth cookie/salt per
//! generation, secure salts through caller-provided randomness, UTC
//! epoch-day conversion, a zeroizing type-7 seed handoff, the optional
//! standard lookup-secret contribution to daily blinding, and the canonical
//! encrypted-service extended `.b32.i2p` address codec for the type-7 to
//! type-11 domain.
//!
//! No DH authorization, no persistent signature-type registry, and no
//! generic key-derivation API are provided here. Successor work extends
//! this module through its exact owner.

use crate::{
    crypto::{base32_decode, base32_encode, chachapoly::ChaCha, red25519},
    error::Error,
};

use hmac::Mac;
use rand::rand_core::{CryptoRng, RngCore};
use sha2::Digest;
use zeroize::{Zeroize, Zeroizing};

use alloc::{string::String, vec::Vec};

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

/// PSK client-authorization layer-1 flags byte.
pub const PSK_LAYER1_FLAGS: u8 = 0x03;

/// Key-derivation label for the outer layer.
const L1_INFO: &[u8] = b"ELS2_L1K";

/// Key-derivation label for the inner layer.
const L2_INFO: &[u8] = b"ELS2_L2K";

/// Key-derivation label for one PSK client record.
const PSK_INFO: &[u8] = b"ELS2PSKA";

/// Authenticated encrypted-data ceiling in bytes.
///
/// Pinned reference `EncryptedLeaseSet.MAX_ENCRYPTED_SIZE`. Bounds both
/// allocation and per-client work: the complete outer ciphertext
/// (`outerSalt || ChaCha(...)`) must fit. Entries are never dropped or
/// truncated to fit.
pub const MAX_ENCRYPTED_DATA_LEN: usize = 4096;

/// Length of one PSK client record (`clientID[8] || encryptedCookie[32]`).
pub const PSK_CLIENT_RECORD_LEN: usize = 40;

/// Absolute pre-allocation ceiling for PSK client entries.
///
/// Derived from the minimum framing (`32 + 1 + 32 + 2 + 32 + 1 = 100` bytes
/// for outer/auth/inner salts plus flags/count/type) against the 4096-byte
/// ceiling with zero inner payload: `(4096 - 100) / 40 = 99`. The exact
/// serialized size with the real inner payload is re-checked before any
/// per-client crypto; the 4096-byte bound is always stricter than the `u16`
/// client-count encoding.
pub const MAX_PSK_CLIENTS: usize = 99;

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

/// One 32-byte PSK client key.
///
/// Secret material: never `Debug`- or display-formattable and zeroized on
/// drop. The base session key is always the first logical authorized key;
/// indexed per-user keys follow in configured order before
/// publication-order randomization.
#[derive(Clone, PartialEq, Eq)]
pub struct PskKey(Zeroizing<[u8; 32]>);

impl PskKey {
    /// Wrap exactly 32 key bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(Zeroizing::new(bytes))
    }

    /// Return the key bytes for one-shot derivation.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Generation-local bounded PSK authorization sequence.
///
/// Carries the base key first followed by indexed per-user keys in
/// configured order (duplicates preserved exactly as configured). Secret
/// material: never `Debug`- or display-formattable; each key zeroizes on
/// drop. Empty sequences are rejected at construction; the exact
/// serialized size is re-checked before per-client crypto.
#[derive(Clone, PartialEq, Eq)]
pub struct PskAuthorization(Vec<PskKey>);

impl PskAuthorization {
    /// Wrap a non-empty bounded key sequence.
    ///
    /// Fails closed on empty input or on more than [`MAX_PSK_CLIENTS`]
    /// entries (absolute pre-allocation ceiling; the exact 4096-byte size
    /// is re-checked with the real inner payload before crypto).
    pub fn from_keys(keys: Vec<PskKey>) -> Result<Self, Error> {
        if keys.is_empty() || keys.len() > MAX_PSK_CLIENTS {
            return Err(Error::InvalidData);
        }
        Ok(Self(keys))
    }

    /// Number of authorized keys.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether no key is carried (never true for constructed values).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Borrow the ordered key slice.
    pub fn as_slice(&self) -> &[PskKey] {
        &self.0
    }
}

/// Standard lookup secret for daily blinded publication.
///
/// Carries the decoded UTF-8 bytes of the standard Base64 session property
/// for exactly one destination generation. Empty means no secret and
/// preserves the unsecreted derivation. Secret material: never `Debug`- or
/// display-formattable and zeroized on drop. Construction validates UTF-8
/// and fails closed; overlong inputs are not truncated here and instead
/// fail closed at blinding derivation through the frozen primitive bound.
#[derive(Clone, PartialEq, Eq)]
pub struct LookupSecret(Zeroizing<Vec<u8>>);

impl LookupSecret {
    /// Empty secret (unsecreted derivation).
    pub fn empty() -> Self {
        Self(Zeroizing::new(Vec::new()))
    }

    /// Wrap decoded secret bytes, failing closed on invalid UTF-8.
    ///
    /// The input is scrubbed before the error is returned.
    pub fn from_bytes(mut bytes: Vec<u8>) -> Result<Self, Error> {
        if core::str::from_utf8(&bytes).is_err() {
            bytes.zeroize();
            return Err(Error::InvalidData);
        }
        Ok(Self(Zeroizing::new(bytes)))
    }

    /// Return the secret bytes for one-shot derivation.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Whether no secret is carried.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Length of the secret in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
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

/// RFC-5869 HKDF-SHA256 with caller-provided output length.
///
/// Composed from the existing `hmac`/`sha2` primitives; no new KDF crate is
/// introduced. Intermediate chaining material is zeroized.
fn hkdf_sha256_n(salt: &[u8; 32], ikm: &[u8], info: &[u8], okm: &mut [u8]) {
    let mut prk = Zeroizing::new([0u8; 32]);
    {
        let mut mac =
            hmac::Hmac::<sha2::Sha256>::new_from_slice(salt).expect("HMAC accepts 32-byte salt");
        mac.update(ikm);
        prk.copy_from_slice(mac.finalize().into_bytes().as_ref());
    }

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
}

/// RFC-5869 HKDF-SHA256 with exact 44-byte output.
fn hkdf_sha256_44(salt: &[u8; 32], ikm: &[u8], info: &[u8]) -> [u8; 44] {
    let mut okm = [0u8; 44];
    hkdf_sha256_n(salt, ikm, info, &mut okm);
    okm
}

/// RFC-5869 HKDF-SHA256 with exact 52-byte output for PSK records.
///
/// Output layout is `clientKey[32] || clientIV[12] || clientID[8]`.
fn hkdf_sha256_52(salt: &[u8; 32], ikm: &[u8], info: &[u8]) -> [u8; 52] {
    let mut okm = [0u8; 52];
    hkdf_sha256_n(salt, ikm, info, &mut okm);
    okm
}

/// Derive one PSK client record schedule.
///
/// Input is `psk || subcredential || published_BE` with `ELS2PSKA`; output
/// is split into the ChaCha key/IV for the encrypted cookie and the 8-byte
/// client ID. Temporary input and output material is zeroized.
fn derive_psk_client(
    auth_salt: &[u8; 32],
    psk: &[u8; 32],
    subcredential: &[u8; 32],
    published: u32,
) -> ([u8; 32], [u8; 12], [u8; 8]) {
    let mut input = Zeroizing::new([0u8; 68]);
    input[..32].copy_from_slice(psk);
    input[32..64].copy_from_slice(subcredential);
    input[64..68].copy_from_slice(&published.to_be_bytes());
    let mut okm = Zeroizing::new(hkdf_sha256_52(auth_salt, &input[..], PSK_INFO));
    let mut key = [0u8; 32];
    let mut iv = [0u8; 12];
    let mut id = [0u8; 8];
    key.copy_from_slice(&okm[..32]);
    iv.copy_from_slice(&okm[32..44]);
    id.copy_from_slice(&okm[44..52]);
    okm.zeroize();
    (key, iv, id)
}

/// Derive the auth-cookie-bound inner-layer schedule.
///
/// Input is `authCookie || subcredential || published_BE` with `ELS2_L2K`.
/// The auth cookie affects the inner layer only; the outer layer input is
/// unchanged from the no-auth schedule.
fn derive_layer_keys_with_cookie(
    salt: &[u8; 32],
    auth_cookie: &[u8; 32],
    subcredential: &[u8; 32],
    published: u32,
) -> ([u8; 32], [u8; 12]) {
    let mut input = Zeroizing::new([0u8; 68]);
    input[..32].copy_from_slice(auth_cookie);
    input[32..64].copy_from_slice(subcredential);
    input[64..68].copy_from_slice(&published.to_be_bytes());
    let mut okm = hkdf_sha256_44(salt, &input[..], L2_INFO);
    let mut key = [0u8; 32];
    let mut iv = [0u8; 12];
    key.copy_from_slice(&okm[..32]);
    iv.copy_from_slice(&okm[32..44]);
    okm.zeroize();
    (key, iv)
}

/// Checked complete PSK outer-ciphertext length.
///
/// Returns `Some(total)` for `32 + 1 + 32 + 2 + 40*N + 32 + 1 + inner_len`
/// with fully checked arithmetic, or `None` on overflow.
pub fn psk_outer_len(inner_len: usize, num_clients: usize) -> Option<usize> {
    num_clients
        .checked_mul(PSK_CLIENT_RECORD_LEN)?
        .checked_add(inner_len)?
        .checked_add(SALT_LEN)?
        .checked_add(1)?
        .checked_add(SALT_LEN)?
        .checked_add(2)?
        .checked_add(SALT_LEN)?
        .checked_add(1)
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

/// Encrypt the PSK client-authorization layers with explicit salts/cookie.
///
/// Plaintext layout follows the pinned reference: `outerSalt ||
/// ChaCha(L1, 0x03 || authSalt || count_BE || records || innerCT)` where
/// `innerCT = innerSalt || ChaCha(L2cookie, 0x03 || inner_ls2)` and each
/// record is `clientID[8] || ChaCha(clientKey, clientIV, authCookie)`.
/// Records are emitted in the slice order given; callers randomize that
/// order for publication when more than one key exists. The complete size
/// is checked with checked arithmetic against [`MAX_ENCRYPTED_DATA_LEN`]
/// before any per-client crypto; oversize input fails closed without
/// dropping entries. Temporary key material is zeroized.
pub fn encrypt_psk_with_salts(
    subcredential: &[u8; 32],
    published: u32,
    inner_ls2_bytes: &[u8],
    inner_salt: &[u8; 32],
    outer_salt: &[u8; 32],
    auth_salt: &[u8; 32],
    auth_cookie: &[u8; 32],
    psks: &[PskKey],
) -> Result<Vec<u8>, Error> {
    if psks.is_empty() || psks.len() > MAX_PSK_CLIENTS {
        return Err(Error::InvalidData);
    }
    if psks.len() > u16::MAX as usize {
        return Err(Error::InvalidData);
    }
    if inner_ls2_bytes.is_empty() || inner_ls2_bytes.len() > MAX_INNER_PAYLOAD_LEN {
        return Err(Error::InvalidData);
    }
    let total = psk_outer_len(inner_ls2_bytes.len(), psks.len()).ok_or(Error::InvalidData)?;
    if total > MAX_ENCRYPTED_DATA_LEN {
        return Err(Error::InvalidData);
    }

    let (l2_key, l2_iv) =
        derive_layer_keys_with_cookie(inner_salt, auth_cookie, subcredential, published);
    let mut inner_plain = Vec::with_capacity(1 + inner_ls2_bytes.len());
    inner_plain.push(INNER_LEASESET2_TYPE);
    inner_plain.extend_from_slice(inner_ls2_bytes);
    chacha_apply(&l2_key, &l2_iv, &mut inner_plain);
    let mut inner_ct = Vec::with_capacity(SALT_LEN + inner_plain.len());
    inner_ct.extend_from_slice(inner_salt);
    inner_ct.extend_from_slice(&inner_plain);
    inner_plain.zeroize();

    let mut records = Vec::with_capacity(psks.len() * PSK_CLIENT_RECORD_LEN);
    for psk in psks {
        let (ckey, civ, cid) =
            derive_psk_client(auth_salt, psk.as_bytes(), subcredential, published);
        let mut encrypted = Zeroizing::new(*auth_cookie);
        chacha_apply(&ckey, &civ, &mut encrypted[..]);
        records.extend_from_slice(&cid);
        records.extend_from_slice(&encrypted[..]);
    }

    let mut l1_plain = Vec::with_capacity(1 + SALT_LEN + 2 + records.len() + inner_ct.len());
    l1_plain.push(PSK_LAYER1_FLAGS);
    l1_plain.extend_from_slice(auth_salt);
    l1_plain.extend_from_slice(&(psks.len() as u16).to_be_bytes());
    l1_plain.extend_from_slice(&records);
    l1_plain.extend_from_slice(&inner_ct);
    records.zeroize();
    inner_ct.zeroize();

    let (l1_key, l1_iv) = derive_layer_keys(outer_salt, subcredential, published, L1_INFO);
    chacha_apply(&l1_key, &l1_iv, &mut l1_plain);
    let mut out = Vec::with_capacity(SALT_LEN + l1_plain.len());
    out.extend_from_slice(outer_salt);
    out.extend_from_slice(&l1_plain);
    l1_plain.zeroize();

    debug_assert_eq!(out.len(), total);
    if out.len() != total || out.len() > MAX_ENCRYPTED_DATA_LEN {
        out.zeroize();
        return Err(Error::InvalidData);
    }
    Ok(out)
}

/// Encrypt the PSK layers with fresh salts/cookie.
///
/// Generates fresh inner/outer/auth salts and a fresh auth cookie from the
/// caller RNG. Record order is freshly randomized when more than one key
/// exists; single-key output is deterministic given the salts/cookie.
pub fn encrypt_psk(
    subcredential: &[u8; 32],
    published: u32,
    inner_ls2_bytes: &[u8],
    psks: &[PskKey],
    mut rng: impl RngCore + CryptoRng,
) -> Result<Vec<u8>, Error> {
    if psks.is_empty() || psks.len() > MAX_PSK_CLIENTS {
        return Err(Error::InvalidData);
    }
    if inner_ls2_bytes.is_empty() || inner_ls2_bytes.len() > MAX_INNER_PAYLOAD_LEN {
        return Err(Error::InvalidData);
    }
    let total = psk_outer_len(inner_ls2_bytes.len(), psks.len()).ok_or(Error::InvalidData)?;
    if total > MAX_ENCRYPTED_DATA_LEN {
        return Err(Error::InvalidData);
    }

    let mut inner_salt = [0u8; SALT_LEN];
    let mut outer_salt = [0u8; SALT_LEN];
    let mut auth_salt = [0u8; SALT_LEN];
    let mut auth_cookie = Zeroizing::new([0u8; 32]);
    rng.fill_bytes(&mut inner_salt);
    rng.fill_bytes(&mut outer_salt);
    rng.fill_bytes(&mut auth_salt);
    rng.fill_bytes(&mut auth_cookie[..]);

    let mut order: Vec<usize> = (0..psks.len()).collect();
    for i in (1..order.len()).rev() {
        let j = (rng.next_u32() as usize) % (i + 1);
        order.swap(i, j);
    }
    let shuffled: Vec<PskKey> = order.iter().map(|&i| psks[i].clone()).collect();
    let out = encrypt_psk_with_salts(
        subcredential,
        published,
        inner_ls2_bytes,
        &inner_salt,
        &outer_salt,
        &auth_salt,
        &auth_cookie,
        &shuffled,
    );
    inner_salt.zeroize();
    outer_salt.zeroize();
    auth_salt.zeroize();
    out
}

/// Decrypt a PSK outer object with one candidate PSK.
///
/// Decrypts the outer layer, selects the client record matching the
/// candidate key's client ID, recovers the auth cookie, re-derives the
/// cookie-bound inner layer, and returns the inner LeaseSet2 bytes. Any
/// flags/type/salt/count/record/cookie mismatch, missing client match, or
/// oversize framing fails closed without fallback.
pub fn decrypt_psk(
    subcredential: &[u8; 32],
    published: u32,
    outer_ciphertext: &[u8],
    psk: &[u8; 32],
) -> Result<Vec<u8>, Error> {
    if outer_ciphertext.len() < SALT_LEN + 1 + SALT_LEN + 2 + SALT_LEN + 1
        || outer_ciphertext.len() > MAX_ENCRYPTED_DATA_LEN
    {
        return Err(Error::InvalidData);
    }
    let mut outer_salt = [0u8; SALT_LEN];
    outer_salt.copy_from_slice(&outer_ciphertext[..SALT_LEN]);
    let (l1_key, l1_iv) = derive_layer_keys(&outer_salt, subcredential, published, L1_INFO);
    let mut l1_plain = outer_ciphertext[SALT_LEN..].to_vec();
    chacha_apply(&l1_key, &l1_iv, &mut l1_plain);

    if l1_plain.is_empty() || l1_plain[0] != PSK_LAYER1_FLAGS {
        l1_plain.zeroize();
        return Err(Error::InvalidData);
    }
    if l1_plain.len() < 1 + SALT_LEN + 2 {
        l1_plain.zeroize();
        return Err(Error::InvalidData);
    }
    let mut auth_salt = [0u8; SALT_LEN];
    auth_salt.copy_from_slice(&l1_plain[1..1 + SALT_LEN]);
    let count = u16::from_be_bytes([l1_plain[1 + SALT_LEN], l1_plain[1 + SALT_LEN + 1]]) as usize;
    if count == 0 || count > MAX_PSK_CLIENTS {
        l1_plain.zeroize();
        return Err(Error::InvalidData);
    }
    let header_len = 1 + SALT_LEN + 2;
    let records_len = count.checked_mul(PSK_CLIENT_RECORD_LEN).ok_or(Error::InvalidData)?;
    if l1_plain.len()
        < header_len.checked_add(records_len).ok_or(Error::InvalidData)? + SALT_LEN + 1
    {
        l1_plain.zeroize();
        return Err(Error::InvalidData);
    }
    let records = l1_plain[header_len..header_len + records_len].to_vec();
    let inner_ct = l1_plain[header_len + records_len..].to_vec();
    l1_plain.zeroize();

    if inner_ct.len() < SALT_LEN + 1 || inner_ct.len() > MAX_OUTER_CIPHERTEXT_LEN {
        return Err(Error::InvalidData);
    }

    let (ckey, civ, cid) = derive_psk_client(&auth_salt, psk, subcredential, published);
    let mut encrypted_cookie: Option<[u8; 32]> = None;
    let mut offset = 0;
    while offset + PSK_CLIENT_RECORD_LEN <= records.len() {
        let record = &records[offset..offset + PSK_CLIENT_RECORD_LEN];
        if record[..8] == cid {
            let mut cookie = [0u8; 32];
            cookie.copy_from_slice(&record[8..40]);
            encrypted_cookie = Some(cookie);
            break;
        }
        offset += PSK_CLIENT_RECORD_LEN;
    }
    let encrypted_cookie = encrypted_cookie.ok_or(Error::InvalidData)?;
    let mut auth_cookie = Zeroizing::new(encrypted_cookie);
    chacha_apply(&ckey, &civ, &mut auth_cookie[..]);

    let mut inner_salt = [0u8; SALT_LEN];
    inner_salt.copy_from_slice(&inner_ct[..SALT_LEN]);
    let (l2_key, l2_iv) =
        derive_layer_keys_with_cookie(&inner_salt, &auth_cookie, subcredential, published);
    let mut inner_plain = inner_ct[SALT_LEN..].to_vec();
    chacha_apply(&l2_key, &l2_iv, &mut inner_plain);
    if inner_plain.is_empty() || inner_plain[0] != INNER_LEASESET2_TYPE {
        inner_plain.zeroize();
        return Err(Error::InvalidData);
    }
    Ok(inner_plain[1..].to_vec())
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
    blinded_day_material_with_secret(seed, unblinded_pubkey, day, &LookupSecret::empty())
}

/// Derive the current-day blinded publication material with a lookup secret.
///
/// An empty secret reproduces [`blinded_day_material`] exactly. A nonempty
/// secret feeds the frozen daily alpha derivation; the same
/// destination/day/secret triple is deterministic while different secrets
/// yield different blinded public and storage keys. Overlong secrets fail
/// closed through the frozen primitive bound without truncation and without
/// falling back to the empty-secret derivation.
pub fn blinded_day_material_with_secret(
    seed: &SigningSeed,
    unblinded_pubkey: &[u8; 32],
    day: &[u8; 8],
    secret: &LookupSecret,
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
        secret.as_bytes(),
    )?;
    let blinded_public = red25519::blind_public_key(unblinded_pubkey, &alpha)?;
    let blinded_private = red25519::blind_private_key_ed25519(seed.as_bytes(), &alpha);
    if red25519::derive_public(&blinded_private) != blinded_public {
        return Err(Error::InvalidData);
    }
    let storage_key = red25519::blinded_storage_key(&blinded_public);
    Ok((alpha, blinded_public, blinded_private, storage_key))
}

/// Decoded wire length of the extended encrypted-service address payload.
pub const EXTENDED_B32_DECODED_LEN: usize = 35;

/// Label length (without suffix) of the extended encrypted-service address.
pub const EXTENDED_B32_LABEL_LEN: usize = 56;

/// Suffix of the extended encrypted-service address.
pub const EXTENDED_B32_SUFFIX: &str = ".b32.i2p";

/// Header flag: two-byte sigtype form (rejected in this milestone).
const B32_FLAG_TWO_BYTE_SIGTYPES: u8 = 0x01;

/// Header flag: a lookup secret is required to resolve the service.
const B32_FLAG_SECRET_REQUIRED: u8 = 0x02;

/// Header flag: per-client authorization is required to resolve the service.
const B32_FLAG_AUTH_REQUIRED: u8 = 0x04;

/// Decoded public metadata of an extended encrypted-service address.
///
/// Carries only the unblinded type-7 public key and the two public
/// requirement flags. It never embeds secret, private, or blinded key
/// material.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncryptedServiceAddress {
    /// Unblinded type-7 Ed25519 public key.
    pub unblinded_public_key: [u8; 32],

    /// Whether a lookup secret is required.
    pub secret_required: bool,

    /// Whether per-client authorization is required.
    pub auth_required: bool,
}

/// Compute the IEEE CRC-32 (`java.util.zip.CRC32` polynomial) of `data`.
///
/// Small exact local helper; no CRC dependency is introduced. Pinned
/// against the standard check value (`"123456789"` -> `0xCBF43926`).
fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ 0xedb8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xffff_ffff
}

/// Encode the canonical extended encrypted-service address.
///
/// Covers only the current type-7 to type-11 one-byte-sigtype domain:
/// 35-byte payload (`flags || 0x07 || 0x0b || public key`) with the IEEE
/// CRC-32 of the public key XORed into the first three header bytes before
/// I2P Base32 encoding, plus the `.b32.i2p` suffix. The encoded label is
/// always exactly 56 characters. The secret itself is never encoded; only
/// the public requirement flags travel in the address. The caller must
/// supply a valid type-7 public key.
pub fn encode_encrypted_service_b32(
    unblinded_pubkey: &[u8; 32],
    secret_required: bool,
    auth_required: bool,
) -> String {
    let mut flags = 0u8;
    if secret_required {
        flags |= B32_FLAG_SECRET_REQUIRED;
    }
    if auth_required {
        flags |= B32_FLAG_AUTH_REQUIRED;
    }

    let crc = crc32_ieee(unblinded_pubkey);
    let mut wire = [0u8; EXTENDED_B32_DECODED_LEN];
    wire[0] = flags ^ (crc & 0xff) as u8;
    wire[1] = UNBLINDED_SIGTYPE as u8 ^ ((crc >> 8) & 0xff) as u8;
    wire[2] = BLINDED_SIGTYPE as u8 ^ ((crc >> 16) & 0xff) as u8;
    wire[3..].copy_from_slice(unblinded_pubkey);

    let mut out = base32_encode(wire);
    out.push_str(EXTENDED_B32_SUFFIX);
    out
}

/// Decode and strictly validate an extended encrypted-service address.
///
/// Accepts ASCII case-insensitive input with the exact `.b32.i2p` suffix
/// and exactly 56 label characters decoding to the 35-byte current-domain
/// payload. Reverses the CRC XOR before interpreting flags and sigtypes,
/// rejects reserved flag bits and the two-byte-sigtype form, requires
/// sigtypes exactly 7 and 11, re-verifies the checksum shape through the
/// header values, and validates the 32-byte public key through the frozen
/// blinding-input gate before treating it as an address identity. Ordinary
/// 52-character destination-hash addresses are rejected as this form.
pub fn decode_encrypted_service_b32(host: &str) -> Result<EncryptedServiceAddress, Error> {
    if !host.is_ascii() {
        return Err(Error::InvalidData);
    }
    let lower = host.to_ascii_lowercase();
    let label = lower.strip_suffix(EXTENDED_B32_SUFFIX).ok_or(Error::InvalidData)?;
    if label.len() != EXTENDED_B32_LABEL_LEN {
        return Err(Error::InvalidData);
    }
    let wire = base32_decode(label).ok_or(Error::InvalidData)?;
    if wire.len() != EXTENDED_B32_DECODED_LEN {
        return Err(Error::InvalidData);
    }

    let mut public_key = [0u8; 32];
    public_key.copy_from_slice(&wire[3..]);

    let crc = crc32_ieee(&public_key);
    let flags = wire[0] ^ (crc & 0xff) as u8;
    let unblinded_sigtype = wire[1] ^ ((crc >> 8) & 0xff) as u8;
    let blinded_sigtype = wire[2] ^ ((crc >> 16) & 0xff) as u8;

    if flags & B32_FLAG_TWO_BYTE_SIGTYPES != 0 {
        return Err(Error::InvalidData);
    }
    if flags & 0xf8 != 0 {
        return Err(Error::InvalidData);
    }
    if unblinded_sigtype != UNBLINDED_SIGTYPE as u8 || blinded_sigtype != BLINDED_SIGTYPE as u8 {
        return Err(Error::InvalidData);
    }

    // Validate the public key through the frozen blinding-input gate
    // (torsion-free, non-identity) with a fixed valid day/empty secret so
    // only the key itself can fail here.
    red25519::generate_alpha(
        &public_key,
        UNBLINDED_SIGTYPE,
        BLINDED_SIGTYPE,
        b"20200101",
        b"",
    )
    .map_err(|_| Error::InvalidData)?;

    Ok(EncryptedServiceAddress {
        unblinded_public_key: public_key,
        secret_required: flags & B32_FLAG_SECRET_REQUIRED != 0,
        auth_required: flags & B32_FLAG_AUTH_REQUIRED != 0,
    })
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

    #[test]
    fn lookup_secret_gates_utf8_and_reports_emptiness() {
        let empty = LookupSecret::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        assert_eq!(empty.as_bytes(), b"");

        let secret = LookupSecret::from_bytes(b"lookup-secret".to_vec()).expect("valid secret");
        assert!(!secret.is_empty());
        assert_eq!(secret.len(), 13);
        assert_eq!(secret.as_bytes(), b"lookup-secret");

        let empty_bytes = LookupSecret::from_bytes(Vec::new()).expect("empty bytes valid");
        assert!(empty_bytes.is_empty());

        assert!(LookupSecret::from_bytes(vec![0xff, 0xfe]).is_err());
        assert!(LookupSecret::from_bytes(vec![0x80]).is_err());
    }

    #[test]
    fn secret_aware_day_material_matches_direct_alpha() {
        let seed = SigningSeed::from_bytes(SEED);

        // Empty secret reproduces the unsecreted derivation exactly.
        let secret = LookupSecret::empty();
        let (alpha, blinded, _, storage) =
            blinded_day_material_with_secret(&seed, &UNBLINDED, &DAY, &secret)
                .expect("empty-secret derives");
        let legacy = blinded_day_material(&seed, &UNBLINDED, &DAY).expect("legacy derives");
        assert_eq!(alpha.as_bytes(), legacy.0.as_bytes());
        assert_eq!(blinded.as_bytes(), legacy.1.as_bytes());
        assert_eq!(storage, legacy.3);

        // Nonempty secret matches the direct primitive known answer.
        let secret = LookupSecret::from_bytes(b"lookup-secret".to_vec()).expect("valid secret");
        let (secret_alpha, secret_blinded, secret_private, secret_storage) =
            blinded_day_material_with_secret(&seed, &UNBLINDED, &DAY, &secret)
                .expect("secret derives");
        let direct = red25519::generate_alpha(&UNBLINDED, 7, 11, &DAY, b"lookup-secret")
            .expect("direct secret alpha derives");
        assert_eq!(secret_alpha.as_bytes(), direct.as_bytes());
        assert_eq!(
            red25519::derive_public(&secret_private),
            secret_blinded,
            "secret private/public agreement holds"
        );
        assert_eq!(
            secret_storage,
            red25519::blinded_storage_key(&secret_blinded)
        );

        // The secret changes the blinded identity deterministically.
        assert_ne!(secret_blinded.as_bytes(), blinded.as_bytes());
        assert_ne!(secret_storage, storage);
        let again =
            blinded_day_material_with_secret(&seed, &UNBLINDED, &DAY, &secret).expect("re-derives");
        assert_eq!(secret_blinded.as_bytes(), again.1.as_bytes());
        assert_eq!(secret_storage, again.3);

        // A different secret yields a different blinded identity.
        let other = LookupSecret::from_bytes(b"other-secret".to_vec()).expect("valid secret");
        let (_, other_blinded, _, other_storage) =
            blinded_day_material_with_secret(&seed, &UNBLINDED, &DAY, &other)
                .expect("other secret derives");
        assert_ne!(secret_blinded.as_bytes(), other_blinded.as_bytes());
        assert_ne!(secret_storage, other_storage);

        // Overlong secrets fail closed without truncation or fallback.
        let long = LookupSecret::from_bytes(vec![b'x'; 65]).expect("stored without truncation");
        assert_eq!(long.len(), 65);
        assert!(blinded_day_material_with_secret(&seed, &UNBLINDED, &DAY, &long).is_err());
    }

    #[test]
    fn crc32_ieee_matches_standard_check_value() {
        assert_eq!(crc32_ieee(b""), 0x0000_0000);
        assert_eq!(crc32_ieee(b"123456789"), 0xcbf4_3926);
        assert_eq!(crc32_ieee(b"hello"), 0x3610_a686);
    }

    #[test]
    fn extended_b32_vectors_pin_flags_crc_and_shape() {
        // CRC over the fixture public key, pinned through the local helper
        // and cross-checked against the `crc32_ieee` standard check above.
        let crc = crc32_ieee(&UNBLINDED);
        assert_eq!(crc, 0xe116_47f0);

        // Byte-exact labels independently recomputed with `binascii.crc32`
        // and RFC-4648 Base32 (lowercased I2P alphabet) over
        // `flags || 0x07 || 0x0b || public key` with the CRC XOR applied.
        let expected = [
            (
                false,
                false,
                "6bab3cui4poxicprsx6vfwznhs5f24wkm4e36hmucin7g5eiag2a6324.b32.i2p",
            ),
            (
                true,
                false,
                "6jab3cui4poxicprsx6vfwznhs5f24wkm4e36hmucin7g5eiag2a6324.b32.i2p",
            ),
            (
                false,
                true,
                "6rab3cui4poxicprsx6vfwznhs5f24wkm4e36hmucin7g5eiag2a6324.b32.i2p",
            ),
        ];

        for (secret_required, auth_required, flags) in [
            (false, false, 0x00u8),
            (true, false, 0x02u8),
            (false, true, 0x04u8),
        ] {
            let host = encode_encrypted_service_b32(&UNBLINDED, secret_required, auth_required);
            assert!(
                host.ends_with(EXTENDED_B32_SUFFIX),
                "missing suffix: {host}"
            );
            let label = host.strip_suffix(EXTENDED_B32_SUFFIX).expect("suffix present");
            assert_eq!(label.len(), EXTENDED_B32_LABEL_LEN);

            let wire = base32_decode(label).expect("label decodes");
            assert_eq!(wire.len(), EXTENDED_B32_DECODED_LEN);
            assert_eq!(wire[0], flags ^ (crc & 0xff) as u8);
            assert_eq!(wire[1], 7u8 ^ ((crc >> 8) & 0xff) as u8);
            assert_eq!(wire[2], 11u8 ^ ((crc >> 16) & 0xff) as u8);
            assert_eq!(&wire[3..], &UNBLINDED);

            let decoded = decode_encrypted_service_b32(&host).expect("round-trips");
            assert_eq!(decoded.unblinded_public_key, UNBLINDED);
            assert_eq!(decoded.secret_required, secret_required);
            assert_eq!(decoded.auth_required, auth_required);

            let pinned = expected
                .iter()
                .find(|(s, a, _)| *s == secret_required && *a == auth_required)
                .expect("pinned vector present");
            assert_eq!(host, pinned.2);
        }

        // Combined flags round-trip for future authorization reuse.
        let host = encode_encrypted_service_b32(&UNBLINDED, true, true);
        let decoded = decode_encrypted_service_b32(&host).expect("round-trips");
        assert!(decoded.secret_required);
        assert!(decoded.auth_required);

        // Decoding is ASCII case-insensitive.
        let upper = encode_encrypted_service_b32(&UNBLINDED, true, false).to_ascii_uppercase();
        let decoded = decode_encrypted_service_b32(&upper).expect("upper decodes");
        assert_eq!(decoded.unblinded_public_key, UNBLINDED);
        assert!(decoded.secret_required);
        assert!(!decoded.auth_required);
    }

    #[test]
    fn extended_b32_decoder_rejects_adversarial_inputs() {
        let host = encode_encrypted_service_b32(&UNBLINDED, false, false);

        // Ordinary destination-hash addresses are not the extended form.
        assert!(decode_encrypted_service_b32(&base32_encode(UNBLINDED)).is_err());
        assert!(decode_encrypted_service_b32(
            "udhdrtrcetjm5sxzskjyr5ztpeszydbh4dpl3pl4utgqqw2v4jna.b32.i2p"
        )
        .is_err());

        // Missing or wrong suffix.
        let label = host.strip_suffix(EXTENDED_B32_SUFFIX).expect("suffix present");
        assert!(decode_encrypted_service_b32(label).is_err());
        assert!(decode_encrypted_service_b32(&format!("{label}.b32.i3p")).is_err());
        assert!(decode_encrypted_service_b32("").is_err());

        // Invalid Base32 and wrong decoded length.
        assert!(decode_encrypted_service_b32(&format!("!{}.b32.i2p", &label[1..])).is_err());
        assert!(decode_encrypted_service_b32(&format!("{}a.b32.i2p", &label[..55])).is_err());
        assert!(decode_encrypted_service_b32(&format!("{label}aa.b32.i2p")).is_err());

        // Reserved flag bits and two-byte-sigtype form.
        let wire = base32_decode(label).expect("label decodes");
        let crc = crc32_ieee(&UNBLINDED);
        for bad_flags in [0x01u8, 0x08, 0x10, 0x80, 0xf8, 0xff] {
            let mut bad = wire.clone();
            bad[0] = bad_flags ^ (crc & 0xff) as u8;
            let bad_host = format!("{}.b32.i2p", base32_encode(&bad));
            assert!(
                decode_encrypted_service_b32(&bad_host).is_err(),
                "flags {bad_flags:#04x} were accepted"
            );
        }

        // Wrong unblinded/blinded sigtype.
        for (b1, b2) in [(7u8, 10u8), (6u8, 11u8), (11u8, 11u8), (7u8, 7u8)] {
            let mut bad = wire.clone();
            bad[1] = b1 ^ ((crc >> 8) & 0xff) as u8;
            bad[2] = b2 ^ ((crc >> 16) & 0xff) as u8;
            let bad_host = format!("{}.b32.i2p", base32_encode(&bad));
            assert!(
                decode_encrypted_service_b32(&bad_host).is_err(),
                "sigtypes {b1}/{b2} were accepted"
            );
        }

        // Invalid public keys never become address identities.
        let zero_host = encode_encrypted_service_b32(&[0u8; 32], false, false);
        assert!(decode_encrypted_service_b32(&zero_host).is_err());

        // Checksum corruption in any header byte fails closed.
        for index in 0..3 {
            let mut bad = wire.clone();
            bad[index] ^= 0x01;
            assert!(
                decode_encrypted_service_b32(&format!("{}.b32.i2p", base32_encode(&bad))).is_err(),
                "corrupted header byte {index} was accepted"
            );
        }
        let mut tampered = wire.clone();
        tampered[0] ^= 0x08;
        assert!(
            decode_encrypted_service_b32(&format!("{}.b32.i2p", base32_encode(&tampered))).is_err()
        );

        // Corrupted public-key bytes fail through the checksum/point gates.
        let mut bad_key = wire.clone();
        bad_key[10] ^= 0x40;
        assert!(
            decode_encrypted_service_b32(&format!("{}.b32.i2p", base32_encode(&bad_key))).is_err()
        );

        // Trailing bytes after the hostname are not part of the codec.
        assert!(decode_encrypted_service_b32(&format!("{host} ")).is_err());
        assert!(decode_encrypted_service_b32(&format!("{host}x")).is_err());
    }

    const PSK_A: [u8; 32] = [0xA1; 32];
    const PSK_B: [u8; 32] = [0xB2; 32];
    const AUTH_SALT: [u8; 32] = [0x33; 32];
    const AUTH_COOKIE: [u8; 32] = [0x44; 32];

    fn independent_hkdf_52(salt: &[u8; 32], ikm: &[u8], info: &[u8]) -> [u8; 52] {
        let mut prk = [0u8; 32];
        {
            let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(salt)
                .expect("HMAC accepts 32-byte salt");
            use hmac::Mac;
            mac.update(ikm);
            prk.copy_from_slice(mac.finalize().into_bytes().as_ref());
        }
        let mut okm = [0u8; 52];
        let mut prev: Option<[u8; 32]> = None;
        for (index, chunk) in okm.chunks_mut(32).enumerate() {
            let mut mac =
                hmac::Hmac::<sha2::Sha256>::new_from_slice(&prk).expect("HMAC accepts PRK");
            use hmac::Mac;
            if let Some(p) = prev {
                mac.update(&p);
            }
            mac.update(info);
            mac.update(&[(index + 1) as u8]);
            let out = mac.finalize().into_bytes();
            chunk.copy_from_slice(&out[..chunk.len()]);
            let mut next = [0u8; 32];
            next.copy_from_slice(out.as_ref());
            prev = Some(next);
        }
        okm
    }

    #[test]
    fn psk_types_enforce_bounded_nonempty_sequences() {
        assert!(PskAuthorization::from_keys(Vec::new()).is_err());
        let one = PskAuthorization::from_keys(vec![PskKey::from_bytes(PSK_A)]).unwrap();
        assert_eq!(one.len(), 1);
        assert!(!one.is_empty());
        assert_eq!(one.as_slice()[0].as_bytes(), &PSK_A);

        let too_many: Vec<PskKey> =
            (0..MAX_PSK_CLIENTS + 1).map(|_| PskKey::from_bytes(PSK_A)).collect();
        assert!(PskAuthorization::from_keys(too_many).is_err());
        let max: Vec<PskKey> = (0..MAX_PSK_CLIENTS).map(|_| PskKey::from_bytes(PSK_A)).collect();
        assert!(PskAuthorization::from_keys(max).is_ok());
    }

    #[test]
    fn psk_outer_len_is_checked_and_bounded() {
        assert_eq!(
            psk_outer_len(16, 1),
            Some(32 + 1 + 32 + 2 + 40 + 32 + 1 + 16)
        );
        assert_eq!(MAX_PSK_CLIENTS, 99);
        assert!(psk_outer_len(16, 100).is_some_and(|len| len > MAX_ENCRYPTED_DATA_LEN));
        assert!(psk_outer_len(usize::MAX, 1).is_none());
        assert!(psk_outer_len(16, usize::MAX).is_none());
        let min_frame = psk_outer_len(0, 0).unwrap();
        assert_eq!(min_frame, 100);
        let max_fit = psk_outer_len(36, 99).unwrap();
        assert!(max_fit <= MAX_ENCRYPTED_DATA_LEN);
        assert!(psk_outer_len(37, 99).unwrap() > MAX_ENCRYPTED_DATA_LEN);
    }

    #[test]
    fn psk_kat_single_client_matches_independent_derivation() {
        let (sub, _) = fixture_subcredential();
        let psks = [PskKey::from_bytes(PSK_A)];

        let mut input = [0u8; 68];
        input[..32].copy_from_slice(&PSK_A);
        input[32..64].copy_from_slice(&sub);
        input[64..68].copy_from_slice(&PUBLISHED.to_be_bytes());
        let expected = independent_hkdf_52(&AUTH_SALT, &input, b"ELS2PSKA");
        let (key, iv, id) = derive_psk_client(&AUTH_SALT, &PSK_A, &sub, PUBLISHED);
        assert_eq!(&key[..], &expected[..32]);
        assert_eq!(&iv[..], &expected[32..44]);
        assert_eq!(&id[..], &expected[44..52]);

        let outer = encrypt_psk_with_salts(
            &sub,
            PUBLISHED,
            &INNER,
            &INNER_SALT,
            &OUTER_SALT,
            &AUTH_SALT,
            &AUTH_COOKIE,
            &psks,
        )
        .unwrap();
        let total = psk_outer_len(INNER.len(), 1).unwrap();
        assert_eq!(outer.len(), total);
        assert!(outer.len() <= MAX_ENCRYPTED_DATA_LEN);
        assert_eq!(&outer[..32], &OUTER_SALT);

        let (l1_key, l1_iv) = derive_layer_keys(&OUTER_SALT, &sub, PUBLISHED, L1_INFO);
        let mut plain = outer[SALT_LEN..].to_vec();
        chacha_apply(&l1_key, &l1_iv, &mut plain);
        assert_eq!(plain[0], PSK_LAYER1_FLAGS);
        assert_eq!(&plain[1..33], &AUTH_SALT);
        assert_eq!(&plain[33..35], &1u16.to_be_bytes());
        assert_eq!(&plain[35..43], &id);
        let mut cookie = AUTH_COOKIE;
        chacha_apply(&key, &iv, &mut cookie);
        assert_eq!(&plain[43..75], &cookie);

        let inner_ct = &plain[75..];
        assert_eq!(&inner_ct[..32], &INNER_SALT);
        let (l2_key, l2_iv) =
            derive_layer_keys_with_cookie(&INNER_SALT, &AUTH_COOKIE, &sub, PUBLISHED);
        let mut inner_plain = inner_ct[32..].to_vec();
        chacha_apply(&l2_key, &l2_iv, &mut inner_plain);
        assert_eq!(inner_plain[0], INNER_LEASESET2_TYPE);
        assert_eq!(&inner_plain[1..], &INNER);

        let recovered = decrypt_psk(&sub, PUBLISHED, &outer, &PSK_A).unwrap();
        assert_eq!(recovered, INNER);

        let (plain_l2_key, _) = derive_layer_keys(&INNER_SALT, &sub, PUBLISHED, L2_INFO);
        assert_ne!(l2_key, plain_l2_key);
        let (outer_l1_key, _) = derive_layer_keys(&OUTER_SALT, &sub, PUBLISHED, L1_INFO);
        assert_eq!(l1_key, outer_l1_key);
    }

    #[test]
    fn psk_multi_client_order_independent_and_randomized() {
        let (sub, _) = fixture_subcredential();
        let forward = [PskKey::from_bytes(PSK_A), PskKey::from_bytes(PSK_B)];
        let reverse = [PskKey::from_bytes(PSK_B), PskKey::from_bytes(PSK_A)];

        let fwd = encrypt_psk_with_salts(
            &sub,
            PUBLISHED,
            &INNER,
            &INNER_SALT,
            &OUTER_SALT,
            &AUTH_SALT,
            &AUTH_COOKIE,
            &forward,
        )
        .unwrap();
        let rev = encrypt_psk_with_salts(
            &sub,
            PUBLISHED,
            &INNER,
            &INNER_SALT,
            &OUTER_SALT,
            &AUTH_SALT,
            &AUTH_COOKIE,
            &reverse,
        )
        .unwrap();
        assert_ne!(fwd, rev);
        for psk in [&PSK_A, &PSK_B] {
            assert_eq!(decrypt_psk(&sub, PUBLISHED, &fwd, psk).unwrap(), INNER);
            assert_eq!(decrypt_psk(&sub, PUBLISHED, &rev, psk).unwrap(), INNER);
        }

        use crate::runtime::{mock::MockRuntime, Runtime};
        let prod_a = encrypt_psk(&sub, PUBLISHED, &INNER, &forward, MockRuntime::rng()).unwrap();
        let prod_b = encrypt_psk(&sub, PUBLISHED, &INNER, &forward, MockRuntime::rng()).unwrap();
        assert!(prod_a.len() <= MAX_ENCRYPTED_DATA_LEN);
        for psk in [&PSK_A, &PSK_B] {
            assert_eq!(decrypt_psk(&sub, PUBLISHED, &prod_a, psk).unwrap(), INNER);
            assert_eq!(decrypt_psk(&sub, PUBLISHED, &prod_b, psk).unwrap(), INNER);
        }
    }

    #[test]
    fn psk_duplicate_entries_preserved_as_separate_records() {
        let (sub, _) = fixture_subcredential();
        let dup = [PskKey::from_bytes(PSK_A), PskKey::from_bytes(PSK_A)];
        let outer = encrypt_psk_with_salts(
            &sub,
            PUBLISHED,
            &INNER,
            &INNER_SALT,
            &OUTER_SALT,
            &AUTH_SALT,
            &AUTH_COOKIE,
            &dup,
        )
        .unwrap();
        assert_eq!(outer.len(), psk_outer_len(INNER.len(), 2).unwrap());

        let (l1_key, l1_iv) = derive_layer_keys(&OUTER_SALT, &sub, PUBLISHED, L1_INFO);
        let mut plain = outer[SALT_LEN..].to_vec();
        chacha_apply(&l1_key, &l1_iv, &mut plain);
        assert_eq!(&plain[33..35], &2u16.to_be_bytes());
        assert_eq!(&plain[35..75], &plain[75..115]);
        assert_eq!(decrypt_psk(&sub, PUBLISHED, &outer, &PSK_A).unwrap(), INNER);

        let max_inner = MAX_ENCRYPTED_DATA_LEN - 100 - 40 * MAX_PSK_CLIENTS;
        let dup_max: Vec<PskKey> =
            (0..MAX_PSK_CLIENTS).map(|_| PskKey::from_bytes(PSK_A)).collect();
        let fitting = vec![0u8; max_inner];
        assert!(encrypt_psk_with_salts(
            &sub,
            PUBLISHED,
            &fitting,
            &INNER_SALT,
            &OUTER_SALT,
            &AUTH_SALT,
            &AUTH_COOKIE,
            &dup_max,
        )
        .is_ok());
        let too_big = vec![0u8; max_inner + 1];
        assert!(encrypt_psk_with_salts(
            &sub,
            PUBLISHED,
            &too_big,
            &INNER_SALT,
            &OUTER_SALT,
            &AUTH_SALT,
            &AUTH_COOKIE,
            &dup_max,
        )
        .is_err());
    }

    #[test]
    fn psk_negative_paths_fail_closed_without_fallback() {
        let (sub, _) = fixture_subcredential();
        let psks = [PskKey::from_bytes(PSK_A)];
        let outer = encrypt_psk_with_salts(
            &sub,
            PUBLISHED,
            &INNER,
            &INNER_SALT,
            &OUTER_SALT,
            &AUTH_SALT,
            &AUTH_COOKIE,
            &psks,
        )
        .unwrap();

        assert!(encrypt_psk_with_salts(
            &sub,
            PUBLISHED,
            &INNER,
            &INNER_SALT,
            &OUTER_SALT,
            &AUTH_SALT,
            &AUTH_COOKIE,
            &[],
        )
        .is_err());
        assert!(decrypt_psk(&sub, PUBLISHED, &outer, &[0x55; 32]).is_err());
        assert!(decrypt_psk(&sub, PUBLISHED + 1, &outer, &PSK_A).is_err());
        assert!(decrypt_no_auth(&sub, PUBLISHED, &outer).is_err());
        let no_auth =
            encrypt_no_auth_with_salts(&sub, PUBLISHED, &INNER, &INNER_SALT, &OUTER_SALT).unwrap();
        assert!(decrypt_psk(&sub, PUBLISHED, &no_auth, &PSK_A).is_err());

        let mut tampered = outer.clone();
        tampered[SALT_LEN] ^= 0xff;
        assert!(decrypt_psk(&sub, PUBLISHED, &tampered, &PSK_A).is_err());

        let (l1_key, l1_iv) = derive_layer_keys(&OUTER_SALT, &sub, PUBLISHED, L1_INFO);
        let mut plain = outer[SALT_LEN..].to_vec();
        chacha_apply(&l1_key, &l1_iv, &mut plain);
        let mut bad = plain.clone();
        bad[35] ^= 0x01;
        let mut bad_plain = bad.clone();
        chacha_apply(&l1_key, &l1_iv, &mut bad_plain);
        let mut bad_outer = OUTER_SALT.to_vec();
        bad_outer.extend_from_slice(&bad_plain);
        assert!(decrypt_psk(&sub, PUBLISHED, &bad_outer, &PSK_A).is_err());

        let mut bad_cookie = plain.clone();
        bad_cookie[43] ^= 0x01;
        let mut bad_cookie_ct = bad_cookie.clone();
        chacha_apply(&l1_key, &l1_iv, &mut bad_cookie_ct);
        let mut bad_cookie_outer = OUTER_SALT.to_vec();
        bad_cookie_outer.extend_from_slice(&bad_cookie_ct);
        assert!(decrypt_psk(&sub, PUBLISHED, &bad_cookie_outer, &PSK_A).is_err());

        let mut bad_salt = plain.clone();
        bad_salt[1] ^= 0x01;
        let mut bad_salt_ct = bad_salt.clone();
        chacha_apply(&l1_key, &l1_iv, &mut bad_salt_ct);
        let mut bad_salt_outer = OUTER_SALT.to_vec();
        bad_salt_outer.extend_from_slice(&bad_salt_ct);
        assert!(decrypt_psk(&sub, PUBLISHED, &bad_salt_outer, &PSK_A).is_err());

        let big_inner = vec![0u8; MAX_ENCRYPTED_DATA_LEN];
        assert!(encrypt_psk_with_salts(
            &sub,
            PUBLISHED,
            &big_inner,
            &INNER_SALT,
            &OUTER_SALT,
            &AUTH_SALT,
            &AUTH_COOKIE,
            &psks,
        )
        .is_err());
        let mut oversize = outer.clone();
        oversize.extend_from_slice(&[0u8; MAX_ENCRYPTED_DATA_LEN]);
        assert!(decrypt_psk(&sub, PUBLISHED, &oversize, &PSK_A).is_err());

        let mut other_sub = sub;
        other_sub[0] ^= 0xff;
        assert_ne!(sub, other_sub);
        assert!(decrypt_psk(&other_sub, PUBLISHED, &outer, &PSK_A).is_err());
    }
}
