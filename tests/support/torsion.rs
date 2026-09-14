//! ATTACK CONSTRUCTION FOR NEGATIVE TESTS ONLY. Never used by the contract itself.
//!
//! Builds ed25519 "torsion twins" `A + T` of a public key `A` and signs for them with
//! `A`'s secret, reproducing the round-3 review/audit finding that ZIP215 (cofactored)
//! verification accepts such signatures: [8]sB = [8]R + [8]kA' and [8]A' = [8]A.
//! Included by the conformance fixture generator (`examples/custody_pop_fixture.rs`) and
//! the integration tests (`tests/conformance.rs`). The expanded secret is zeroized on drop
//! by ed25519-dalek; the caller owns the seed.
#![allow(dead_code)]

use curve25519_dalek::constants::EIGHT_TORSION;
use curve25519_dalek::edwards::CompressedEdwardsY;
use ed25519_dalek::hazmat::{raw_sign, ExpandedSecretKey};
use ed25519_dalek::VerifyingKey;
use sha2::Sha512;

/// `A + EIGHT_TORSION[index]` for the canonical ed25519 public key `public`.
pub fn twin(public: &[u8; 32], index: usize) -> [u8; 32] {
    let point = CompressedEdwardsY(*public)
        .decompress()
        .expect("twin() needs a valid ed25519 public key");
    (point + EIGHT_TORSION[index]).compress().to_bytes()
}

/// An RFC 8032 signature by the secret behind `seed`, with `claimed_public` (a torsion
/// twin of the real key) hashed into the challenge.
// VERIFIED: ed25519-dalek 2.2.0 `hazmat::raw_sign` (`src/hazmat.rs:137-146`) signs with the
// supplied verifying key in the challenge; `From<&SecretKey> for ExpandedSecretKey`
// (`src/signing.rs:809-815`) is the RFC 8032 SHA-512 expansion.
pub fn sign_as(seed: &[u8; 32], claimed_public: &[u8; 32], message: &[u8]) -> [u8; 64] {
    let expanded = ExpandedSecretKey::from(seed);
    let claimed = VerifyingKey::from_bytes(claimed_public)
        .expect("sign_as() needs a claimed key that decompresses");
    raw_sign::<Sha512>(&expanded, message, &claimed).to_bytes()
}
