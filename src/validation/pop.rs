use super::custody::bytes;
use super::*;
use sha2::Digest as _;
use sp_core::{ed25519, sr25519, Pair};

/// Domain separator of the custody proof-of-possession message, version 1.
pub const CUSTODY_POP_DOMAIN: &[u8] = b"D9-V2-CUSTODY-POP/1";

/// The canonical custody proof-of-possession message. This is the only place the
/// message is built; signers (tools, wallets) and this validator must all use it.
///
/// `CUSTODY_POP_DOMAIN` followed by six length-prefixed fields, each
/// `u64 big-endian byte length || bytes` (the length encoding `dataset_digest` uses):
/// network label (`"mainnet"`/`"testnet"`), `chain.id`, role (`"sudo"`,
/// `"usdtOwner"` or `"admin/<pallet>"`), multisig AccountId32, signatory
/// AccountId32 (its sr25519/ed25519 public key) and the 32-byte ceremony nonce.
/// Changing any of them invalidates the proof, so a testnet proof cannot be
/// replayed on mainnet and a proof for one role or multisig cannot be reused.
pub fn custody_pop_message(
    network: Network,
    chain_id: &str,
    role: &str,
    multisig_address: &[u8; 32],
    signatory: &[u8; 32],
    nonce: &[u8; 32],
) -> Vec<u8> {
    let mut message = CUSTODY_POP_DOMAIN.to_vec();
    for field in [
        network.label().as_bytes(),
        chain_id.as_bytes(),
        role.as_bytes(),
        multisig_address,
        signatory,
        nonce,
    ] {
        message.extend((field.len() as u64).to_be_bytes());
        message.extend(field);
    }
    message
}

/// SHA-256 of `custody_pop_message`: the exact `user_data` an enclave-held signatory's
/// Nitro attestation must carry, and `EnclaveAttestation::pop_message_sha256`. The
/// signer's attest mode and the producer's verifier must both use this function.
pub fn custody_pop_message_sha256(
    network: Network,
    chain_id: &str,
    role: &str,
    multisig_address: &[u8; 32],
    signatory: &[u8; 32],
    nonce: &[u8; 32],
) -> [u8; 32] {
    sha2::Sha256::digest(custody_pop_message(
        network,
        chain_id,
        role,
        multisig_address,
        signatory,
        nonce,
    ))
    .into()
}

/// The bytes a signature covers for `form`: the message itself (`raw`), or the
/// polkadot-js `signRaw` wrapping `b"<Bytes>" ++ message ++ b"</Bytes>"`
/// (`bytes-wrapped`) that browser and hardware wallets apply.
pub fn custody_pop_payload(form: PopMessageForm, message: &[u8]) -> Vec<u8> {
    match form {
        PopMessageForm::Raw => message.to_vec(),
        PopMessageForm::BytesWrapped => [b"<Bytes>".as_slice(), message, b"</Bytes>"].concat(),
    }
}

/// Largest accepted decoded attestation document. Nitro documents are a few KiB.
pub const MAX_ATTESTATION_DOCUMENT_BYTES: usize = 16 * 1024;

/// Parse a validated fixed-width lowercase hex string. The scalar types guarantee
/// the width and alphabet at decode time, so this cannot fail for them.
fn hex_array<const N: usize>(hex: &str) -> [u8; N] {
    let mut out = [0u8; N];
    for (index, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[2 * index..2 * index + 2], 16)
            .expect("SAFETY: scalar decoding guarantees lowercase hex of this width");
    }
    out
}

/// Decoded length of canonical standard base64 with `=` padding, or `None` if the
/// text is not canonical (bad alphabet, length, padding or non-zero trailing bits).
fn canonical_base64_len(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return None;
    }
    let value = |c: u8| match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    };
    let padding = bytes.iter().rev().take_while(|&&c| c == b'=').count();
    if padding > 2 || (padding > 0 && bytes.len() < 4) {
        return None;
    }
    let body = &bytes[..bytes.len() - padding];
    let mut last = 0;
    for &c in body {
        last = value(c)?;
    }
    // Canonical encoders zero the unused low bits of the final sextet.
    let unused_bits_zero = match padding {
        1 => last & 0b11 == 0,
        2 => last & 0b1111 == 0,
        _ => true,
    };
    unused_bits_zero.then_some(bytes.len() / 4 * 3 - padding)
}

/// Ruling 4 (yvan 2026-09-14 04:34 UTC): every custody signatory proves possession
/// of its key. Runs after structure, identity, derivation, distinctness,
/// independence and rehome checks, so those report first.
pub(super) fn check(
    i: &ContractInput,
    authority: &MultisigAuthority,
    path: &str,
    role: &str,
) -> Check {
    let nonce: [u8; 32] = hex_array(i.custody.ceremony_nonce.as_str());
    let multisig = bytes(&authority.address);
    for (index, signatory) in authority.signatories.iter().enumerate() {
        let signatory_path = format!("{path}/signatories/{index}/evidence");
        let key = bytes(&signatory.address);
        let message =
            custody_pop_message(i.chain.network, &i.chain.id, role, &multisig, &key, &nonce);
        match &signatory.evidence {
            PossessionEvidence::Signature(evidence) => signature(
                evidence,
                &key,
                &message,
                &format!("{signatory_path}/signature"),
            )?,
            PossessionEvidence::EnclaveAttested(evidence) => enclave_attested(
                evidence,
                &message,
                &format!("{signatory_path}/enclaveAttested"),
            )?,
        }
    }
    Ok(())
}

fn signature(evidence: &SignatureEvidence, key: &[u8; 32], message: &[u8], path: &str) -> Check {
    let payload = custody_pop_payload(evidence.message, message);
    let signature: [u8; 64] = hex_array(evidence.signature.as_str());
    // VERIFIED: sp-core 43.0.0 `src/sr25519.rs:49` (`SIGNING_CTX = b"substrate"`)
    // and `:265-269` (`verify` = schnorrkel `verify_simple(SIGNING_CTX, ..)`);
    // `src/ed25519.rs:118-124` (`verify` = ed25519-zebra over the message bytes).
    let valid = match evidence.scheme {
        SignatureScheme::Sr25519 => sr25519::Pair::verify(
            &sr25519::Signature::from_raw(signature),
            &payload,
            &sr25519::Public::from_raw(*key),
        ),
        SignatureScheme::Ed25519 => ed25519::Pair::verify(
            &ed25519::Signature::from_raw(signature),
            &payload,
            &ed25519::Public::from_raw(*key),
        ),
        SignatureScheme::Ecdsa => {
            return Err(fail(
                "custody_pop_scheme",
                format!("{path}/scheme"),
                "ecdsa cannot prove possession for an AccountId32: the account is blake2_256 of the public key, so there is no public key to verify against; use sr25519 or ed25519",
            ))
        }
    };
    if !valid {
        return Err(fail(
            "custody_pop_invalid",
            format!("{path}/signature"),
            "signature does not prove possession of this signatory key for this network, chain id, role, multisig and ceremony nonce",
        ));
    }
    Ok(())
}

/// Contract-side checks only (yvan 2026-09-14 04:57 UTC). The COSE signature,
/// the certificate chain to the AWS Nitro root, PCR0 and `user_data` are verified
/// by the producer with `d9-enclave-common` `attest_verify`; this crate does not
/// claim to verify the attestation.
fn enclave_attested(evidence: &EnclaveAttestation, message: &[u8], path: &str) -> Check {
    match canonical_base64_len(&evidence.attestation_document) {
        Some(len) if len > 0 && len <= MAX_ATTESTATION_DOCUMENT_BYTES => {}
        _ => {
            return Err(fail(
                "custody_pop_attestation_malformed",
                format!("{path}/attestationDocument"),
                format!("attestation document must be non-empty canonical padded base64 of at most {MAX_ATTESTATION_DOCUMENT_BYTES} bytes"),
            ))
        }
    }
    if evidence.expected_pcr0.as_str().bytes().all(|b| b == b'0') {
        return Err(fail(
            "custody_pop_attestation_malformed",
            format!("{path}/expectedPcr0"),
            "expected PCR0 cannot be all zeros (a debug-mode enclave measurement)",
        ));
    }
    let expected: [u8; 32] = sha2::Sha256::digest(message).into();
    if hex_array::<32>(evidence.pop_message_sha256.as_str()) != expected {
        return Err(fail(
            "custody_pop_attestation_binding",
            format!("{path}/popMessageSha256"),
            "popMessageSha256 must equal SHA-256 of custody_pop_message for this network, chain id, role, multisig, signatory and ceremony nonce",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custody_pop_message_matches_the_documented_byte_vector() {
        let message = custody_pop_message(
            Network::Testnet,
            "d9_testnet_fixture",
            "admin/d9-amm",
            &[0x11; 32],
            &[0x22; 32],
            &[0x33; 32],
        );
        // Independently computed from the CONTRACT.md 9.6 spec (Python).
        let expected = "44392d56322d435553544f44592d504f502f310000000000000007746573746e6574000000000000001264395f746573746e65745f66697874757265000000000000000c61646d696e2f64392d616d6d000000000000002011111111111111111111111111111111111111111111111111111111111111110000000000000020222222222222222222222222222222222222222222222222222222222222222200000000000000203333333333333333333333333333333333333333333333333333333333333333";
        let hex: String = message.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(message.len(), 200);
        assert_eq!(hex, expected);
        let wrapped = custody_pop_payload(PopMessageForm::BytesWrapped, &message);
        assert_eq!(&wrapped[..7], b"<Bytes>");
        assert_eq!(&wrapped[7..207], message.as_slice());
        assert_eq!(&wrapped[207..], b"</Bytes>");
        assert_eq!(custody_pop_payload(PopMessageForm::Raw, &message), message);
        let digest = custody_pop_message_sha256(
            Network::Testnet,
            "d9_testnet_fixture",
            "admin/d9-amm",
            &[0x11; 32],
            &[0x22; 32],
            &[0x33; 32],
        );
        let digest: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        // Independently computed with Python hashlib over the vector above.
        assert_eq!(
            digest,
            "f1e23b3ad4e34e4fdb7d135f4f088a2301dae85465a3aeea27edcb9b2ccf9281"
        );
    }

    #[test]
    fn attestation_documents_must_be_canonical_padded_base64() {
        assert_eq!(canonical_base64_len("c3ludGhldGlj"), Some(9));
        assert_eq!(canonical_base64_len("YQ=="), Some(1));
        assert_eq!(canonical_base64_len("YWI="), Some(2));
        assert_eq!(canonical_base64_len(""), Some(0));
        for bad in ["YQ", "YR==", "YWJ=", "Y===", "Y Q=", "YQ=a", "-_8="] {
            assert_eq!(canonical_base64_len(bad), None, "{bad}");
        }
    }
}
