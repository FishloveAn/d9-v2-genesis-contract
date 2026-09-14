//! ATTACK CONSTRUCTION FOR NEGATIVE TESTS ONLY. Never used by the contract itself.
//!
//! Builds ed25519 "torsion twins" `A + T` of a public key `A` and signs for them with
//! `A`'s secret, reproducing the round-3 review/audit finding that ZIP215 (cofactored)
//! verification accepts such signatures: [8]sB = [8]R + [8]kA' and [8]A' = [8]A.
//! Shared by the conformance fixture generator (`examples/custody_pop_fixture.rs`) and
//! the contract's unit tests; seeds only ever live in the caller's memory.
#![allow(dead_code)]

use curve25519_dalek::constants::{ED25519_BASEPOINT_POINT, EIGHT_TORSION};
use curve25519_dalek::edwards::CompressedEdwardsY;
use curve25519_dalek::scalar::Scalar;
use sha2::{Digest, Sha512};

/// `A + EIGHT_TORSION[index]` for the canonical ed25519 public key `public`.
pub fn twin(public: &[u8; 32], index: usize) -> [u8; 32] {
    let point = CompressedEdwardsY(*public)
        .decompress()
        .expect("twin() needs a valid ed25519 public key");
    (point + EIGHT_TORSION[index]).compress().to_bytes()
}

/// An RFC 8032-shaped signature by the secret behind `seed`, but with `claimed_public`
/// (a torsion twin of the real key) hashed into the challenge.
pub fn sign_as(seed: &[u8; 32], claimed_public: &[u8; 32], message: &[u8]) -> [u8; 64] {
    let expanded = Sha512::digest(seed);
    let mut secret = [0u8; 32];
    secret.copy_from_slice(&expanded[..32]);
    secret[0] &= 248;
    secret[31] &= 127;
    secret[31] |= 64;
    let a = Scalar::from_bytes_mod_order(secret);
    let wide = |digest: sha2::digest::Output<Sha512>| {
        let mut bytes = [0u8; 64];
        bytes.copy_from_slice(&digest);
        Scalar::from_bytes_mod_order_wide(&bytes)
    };
    let r = wide(
        Sha512::new()
            .chain_update(&expanded[32..])
            .chain_update(message)
            .finalize(),
    );
    let big_r = (ED25519_BASEPOINT_POINT * r).compress().to_bytes();
    let k = wide(
        Sha512::new()
            .chain_update(big_r)
            .chain_update(claimed_public)
            .chain_update(message)
            .finalize(),
    );
    let s = r + k * a;
    let mut signature = [0u8; 64];
    signature[..32].copy_from_slice(&big_r);
    signature[32..].copy_from_slice(s.as_bytes());
    signature
}
