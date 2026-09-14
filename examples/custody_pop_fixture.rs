//! Synthetic custody fixture generator for `fixtures/complete.json` (D9-400).
//!
//! Generates every custody signatory and the fixture session keys as fresh keypairs from
//! OS randomness, signs each raw proof-of-possession in memory with
//! `custody_pop_message`, and writes ONLY public keys, multisig addresses, signatures
//! and the ceremony nonce. It also emits the ed25519 torsion-twin authorities used by
//! negative conformance cases (see `tests/support/torsion.rs`). Seeds are never written
//! or logged. Before exiting it scans the given roots for every seed (raw bytes and
//! lower/upper hex) and prints only the match count.
//!
//! usage: cargo run --example custody_pop_fixture -- <network> <chain-id> <out.json> <scan-root>...
//!
//! Synthetic conformance keys only. Not a key ceremony tool: real custody keys are
//! generated and held by their custodians, who sign the same message themselves.
use d9_genesis_contract::{
    custody_pop_message, multisig_account, Address, AdminPallet, CustodyRole, Network,
    SignatureScheme,
};
use serde_json::{json, Value};
use sp_core::{ed25519, sr25519, Pair};
use std::io::Read;
use std::path::Path;

#[path = "../tests/support/torsion.rs"]
mod torsion;

/// Every seed this run creates: 14 roles x 3, one twin source, and two validators x four
/// session keys (babe, grandpa, liveness, discovery).
const SEED_CAPACITY: usize = 14 * 3 + 1 + 2 * 4;

enum Key {
    Sr(Box<sr25519::Pair>),
    Ed(Box<ed25519::Pair>),
}

struct Custodian {
    key: Key,
    /// Index of this key's seed in `seeds` (in memory only), for twin signing.
    seed: usize,
}

impl Custodian {
    fn public(&self) -> [u8; 32] {
        match &self.key {
            Key::Sr(pair) => pair.public().0,
            Key::Ed(pair) => pair.public().0,
        }
    }
    fn scheme(&self) -> SignatureScheme {
        match self.key {
            Key::Sr(_) => SignatureScheme::Sr25519,
            Key::Ed(_) => SignatureScheme::Ed25519,
        }
    }
    fn sign(&self, message: &[u8]) -> [u8; 64] {
        match &self.key {
            Key::Sr(pair) => pair.sign(message).0,
            Key::Ed(pair) => pair.sign(message).0,
        }
    }
}

fn wipe(seed: &mut [u8; 32]) {
    for byte in seed.iter_mut() {
        // SAFETY: `byte` is a valid, aligned, exclusive reference into `seed`.
        unsafe { std::ptr::write_volatile(byte, 0) };
    }
}

/// `Pair::generate` fills the seed from `OsRng` (sp-core 43.0.0 `src/crypto.rs:851-855`).
/// The returned seed is moved into `seeds` (fixed capacity, so never reallocated) and
/// the local copy is wiped; sp-core's own temporaries and the pair's secret state are
/// not explicitly zeroized.
fn custodian(ed: bool, seeds: &mut Vec<[u8; 32]>) -> Custodian {
    assert!(seeds.len() < SEED_CAPACITY, "seed capacity exceeded");
    let (key, mut seed) = if ed {
        let (pair, seed) = ed25519::Pair::generate();
        (Key::Ed(Box::new(pair)), seed)
    } else {
        let (pair, seed) = sr25519::Pair::generate();
        (Key::Sr(Box::new(pair)), seed)
    };
    seeds.push(seed);
    wipe(&mut seed);
    Custodian {
        key,
        seed: seeds.len() - 1,
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn serde_label<T: serde::Serialize>(value: T) -> Value {
    serde_json::to_value(value).expect("unit enum serializes")
}

/// Counts files containing any needle. Needles are bucketed by their first two
/// bytes so each file is scanned in one pass.
fn scan(root: &Path, buckets: &[Vec<Vec<u8>>], matches: &mut usize, files: &mut usize) {
    let Ok(metadata) = std::fs::symlink_metadata(root) else {
        return;
    };
    if metadata.is_dir() {
        let skip = root
            .file_name()
            .is_some_and(|name| name == "target" || name == ".git");
        if skip {
            return;
        }
        let Ok(entries) = std::fs::read_dir(root) else {
            return;
        };
        for entry in entries.flatten() {
            scan(&entry.path(), buckets, matches, files);
        }
    } else if metadata.is_file() && metadata.len() <= 64 * 1024 * 1024 {
        let Ok(bytes) = std::fs::read(root) else {
            return;
        };
        *files += 1;
        for (position, pair) in bytes.windows(2).enumerate() {
            let bucket = &buckets[usize::from(u16::from_be_bytes([pair[0], pair[1]]))];
            if bucket
                .iter()
                .any(|needle| bytes[position..].starts_with(needle))
            {
                *matches += 1;
                return;
            }
        }
    }
}

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [network, chain_id, out, roots @ ..] = args.as_slice() else {
        eprintln!("usage: custody_pop_fixture <network> <chain-id> <out.json> <scan-root>...");
        return std::process::ExitCode::FAILURE;
    };
    let network: Network = match serde_json::from_value(json!(network)) {
        Ok(network) => network,
        Err(error) => {
            eprintln!("network: {error}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let mut nonce = [0u8; 32];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut nonce))
        .expect("OS randomness is available");

    // Fixed capacity: a reallocation would leave an unwiped copy of earlier seeds.
    let mut seeds: Vec<[u8; 32]> = Vec::with_capacity(SEED_CAPACITY);
    let mut labels = vec![CustodyRole::Sudo, CustodyRole::UsdtOwner];
    labels.extend(AdminPallet::ALL.into_iter().map(CustodyRole::Admin));
    // Each role is 2-of-3: two sr25519 and one ed25519 key, all with raw proofs.
    let mut roles: Vec<(CustodyRole, Vec<Custodian>, [u8; 32])> = labels
        .into_iter()
        .map(|label| {
            let mut custodians = vec![
                custodian(false, &mut seeds),
                custodian(false, &mut seeds),
                custodian(true, &mut seeds),
            ];
            custodians.sort_by_key(Custodian::public);
            let keys: Vec<[u8; 32]> = custodians.iter().map(Custodian::public).collect();
            let address = multisig_account(&keys, 2);
            (label, custodians, address)
        })
        .collect();

    let ss58 = |bytes: [u8; 32]| Address::from_account_id(bytes.into()).as_str().to_owned();
    let signatory = |key: [u8; 32], scheme: SignatureScheme, signature: [u8; 64]| {
        json!({
            "address": ss58(key),
            "evidence": { "signature": {
                "scheme": serde_label(scheme),
                "signature": hex(&signature),
            }},
        })
    };
    // Signs a proof for `role` over `multisig`: custodian keys sign normally, a
    // `(public, seed index)` twin signs with the twin signer (attack construction).
    let authority = |role: CustodyRole,
                     members: &[(&Custodian, Option<[u8; 32]>)],
                     seeds: &[[u8; 32]]| {
        let mut keys: Vec<([u8; 32], &Custodian, bool)> = members
            .iter()
            .map(|(custodian, twin)| {
                (
                    twin.unwrap_or(custodian.public()),
                    *custodian,
                    twin.is_some(),
                )
            })
            .collect();
        keys.sort_by_key(|(key, _, _)| *key);
        let publics: Vec<[u8; 32]> = keys.iter().map(|(key, _, _)| *key).collect();
        let address = multisig_account(&publics, 2);
        let signatories: Vec<Value> = keys
            .iter()
            .map(|(key, custodian, is_twin)| {
                let message = custody_pop_message(network, chain_id, role, &address, key, &nonce);
                let signature = if *is_twin {
                    let signature = torsion::sign_as(&seeds[custodian.seed], key, &message);
                    assert!(ed25519::Pair::verify(
                        &ed25519::Signature::from_raw(signature),
                        &message,
                        &ed25519::Public::from_raw(*key),
                    ));
                    signature
                } else {
                    custodian.sign(&message)
                };
                signatory(*key, custodian.scheme(), signature)
            })
            .collect();
        json!({ "address": ss58(address), "threshold": 2, "signatories": signatories })
    };
    let mut out_roles = Vec::new();
    for (label, custodians, _) in &roles {
        let members: Vec<(&Custodian, Option<[u8; 32]>)> = custodians
            .iter()
            .map(|custodian| (custodian, None))
            .collect();
        let mut value = authority(*label, &members, &seeds);
        value["role"] = json!(label.label());
        out_roles.push(value);
    }
    let ed_of = |custodians: &[Custodian]| -> usize {
        custodians
            .iter()
            .position(|custodian| matches!(custodian.key, Key::Ed(_)))
            .expect("each role has one ed25519 custodian")
    };
    // Torsion twins (decision A negative cases). Each twin signature is checked above to
    // verify under sp-core ed25519 before it is written.
    let sudo = &roles[0].1;
    let sudo_ed = ed_of(sudo);
    let sudo_twin_members: Vec<(&Custodian, Option<[u8; 32]>)> = sudo
        .iter()
        .enumerate()
        .map(|(index, custodian)| {
            let twin = (index == sudo_ed).then(|| torsion::twin(&custodian.public(), 1));
            (custodian, twin)
        })
        .collect();
    let sudo_with_twin = authority(CustodyRole::Sudo, &sudo_twin_members, &seeds);
    let one_seed = custodian(true, &mut seeds);
    let one_seed_members = [
        (&one_seed, None),
        (&one_seed, Some(torsion::twin(&one_seed.public(), 1))),
        (&one_seed, Some(torsion::twin(&one_seed.public(), 2))),
    ];
    let sudo_twins_of_one_seed = authority(CustodyRole::Sudo, &one_seed_members, &seeds);
    let admin3 = &roles[5].1; // roles: sudo, usdtOwner, then AdminPallet::ALL; [5] = admins[3]
    let admin3_ed = &admin3[ed_of(admin3)];
    let bypass_members: Vec<(&Custodian, Option<[u8; 32]>)> = sudo
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != sudo_ed)
        .map(|(_, custodian)| (custodian, None))
        .chain(std::iter::once((
            admin3_ed,
            Some(torsion::twin(&admin3_ed.public(), 3)),
        )))
        .collect();
    let sudo_independence_bypass = authority(CustodyRole::Sudo, &bypass_members, &seeds);
    // Real session public keys for the two fixture validators: ed25519 grandpa and
    // sr25519 babe, liveness and discovery (CR4-02: every sr25519 slot must decode).
    let session_keys: Vec<Value> = (0..2)
        .map(|_| {
            let mut sr = || hex(&custodian(false, &mut seeds).public());
            let (babe, liveness, discovery) = (sr(), sr(), sr());
            json!({
                "babe": babe,
                "grandpa": hex(&custodian(true, &mut seeds).public()),
                "liveness": liveness,
                "discovery": discovery,
            })
        })
        .collect();
    // A genuine proof by sudo's first signatory, but for the usdtOwner role and
    // multisig: valid signature, wrong role. Used by a negative conformance case.
    let (usdt_address, sudo_first) = (roles[1].2, &roles[0].1[0]);
    let other_role = custody_pop_message(
        network,
        chain_id,
        CustodyRole::UsdtOwner,
        &usdt_address,
        &sudo_first.public(),
        &nonce,
    );
    let document = json!({
        "ceremonyNonce": hex(&nonce),
        "roles": out_roles,
        "sessionKeys": session_keys,
        "alternates": {
            "sudoSignatory0AsUsdtOwner": hex(&sudo_first.sign(&other_role)),
            "sudoWithTorsionTwin": sudo_with_twin,
            "sudoTwinsOfOneSeed": sudo_twins_of_one_seed,
            "sudoIndependenceBypassViaAdmin3Twin": sudo_independence_bypass,
        },
    });
    assert_eq!(seeds.len(), SEED_CAPACITY, "every seed accounted for");
    let bytes = serde_json::to_vec_pretty(&document).expect("document serializes");
    if let Err(error) = std::fs::write(out, &bytes) {
        eprintln!("{out}: {error}");
        return std::process::ExitCode::FAILURE;
    }
    drop(one_seed);
    roles.clear();

    let mut needles: Vec<Vec<u8>> = Vec::with_capacity(seeds.len() * 3);
    for seed in &seeds {
        needles.push(seed.to_vec());
        // Hex is written into buffers owned by `needles`, which are all overwritten
        // before exit; no intermediate hex `String` of a seed is created.
        let mut lower = Vec::with_capacity(64);
        for byte in seed {
            const DIGITS: &[u8; 16] = b"0123456789abcdef";
            lower.push(DIGITS[usize::from(byte >> 4)]);
            lower.push(DIGITS[usize::from(byte & 0x0f)]);
        }
        let mut upper = lower.clone();
        upper.make_ascii_uppercase();
        needles.push(lower);
        needles.push(upper);
    }
    let mut buckets = vec![Vec::new(); 1 << 16];
    for needle in needles {
        buckets[usize::from(u16::from_be_bytes([needle[0], needle[1]]))].push(needle);
    }
    // Positive control: a random non-secret canary is written in hex beside the
    // output and must be found exactly once, proving the scanner can see a match.
    let mut canary = [0u8; 32];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut canary))
        .expect("OS randomness is available");
    let canary_path = format!("{out}.scan-canary");
    std::fs::write(&canary_path, hex(&canary)).expect("canary file is writable");
    let mut control = vec![Vec::new(); 1 << 16];
    let canary_hex = hex(&canary).into_bytes();
    control[usize::from(u16::from_be_bytes([canary_hex[0], canary_hex[1]]))].push(canary_hex);
    let (mut control_matches, mut control_files) = (0, 0);
    let (mut matches, mut files) = (0, 0);
    for root in roots {
        scan(
            Path::new(root),
            &control,
            &mut control_matches,
            &mut control_files,
        );
        scan(Path::new(root), &buckets, &mut matches, &mut files);
    }
    std::fs::remove_file(&canary_path).expect("canary file is removable");
    for needle in buckets.iter_mut().flatten() {
        for byte in needle.iter_mut() {
            // SAFETY: `byte` is a valid, aligned, exclusive reference into `needle`.
            unsafe { std::ptr::write_volatile(byte, 0) };
        }
    }
    for seed in &mut seeds {
        wipe(seed);
    }
    println!(
        "wrote {out}: 14 roles, 42 signatories, 3 twin authorities, 2 validators x 4 session keys; seed leak scan: {} seeds x 3 encodings over {files} files, {matches} files matched; canary control matched {control_matches} of {control_files} files",
        seeds.len()
    );
    if matches == 0 && control_matches == 1 {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}
