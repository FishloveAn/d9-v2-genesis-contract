//! Synthetic custody fixture generator for `fixtures/complete.json` (D9-400 ruling 4).
//!
//! Generates every custody signatory as a fresh keypair from OS randomness, signs
//! each proof-of-possession in memory with `custody_pop_message`, and writes ONLY
//! public keys, multisig addresses, signatures and the ceremony nonce. Seeds are
//! never written, logged or returned; they are wiped from the local copies
//! immediately after key construction. Before exiting it scans the given roots for
//! every seed (raw bytes and lower/upper hex) and prints only the match count.
//!
//! usage: cargo run --example custody_pop_fixture -- <network> <chain-id> <out.json> <scan-root>...
//!
//! Synthetic conformance keys only. Not a key ceremony tool: real custody keys are
//! generated and held by their custodians, who sign the same message themselves.
use d9_genesis_contract::{
    custody_pop_message, custody_pop_payload, multisig_account, Address, Network, PopMessageForm,
    SignatureScheme,
};
use serde_json::{json, Value};
use sp_core::{ed25519, sr25519, Pair};
use std::io::Read;
use std::path::Path;

const PALLETS: [&str; 12] = [
    "d9-amm",
    "d9-burn-mining",
    "d9-cross-chain",
    "d9-governance",
    "d9-judicial-penalty",
    "d9-merchant",
    "d9-mining-pool",
    "d9-node-registry",
    "d9-node-rewards",
    "d9-referrals",
    "d9-upgrade-coordinator",
    "d9-voting",
];

enum Key {
    Sr(Box<sr25519::Pair>),
    Ed(Box<ed25519::Pair>),
}

struct Custodian {
    key: Key,
    form: PopMessageForm,
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
        let payload = custody_pop_payload(self.form, message);
        match &self.key {
            Key::Sr(pair) => pair.sign(&payload).0,
            Key::Ed(pair) => pair.sign(&payload).0,
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
fn custodian(ed: bool, form: PopMessageForm, seeds: &mut Vec<[u8; 32]>) -> Custodian {
    let (key, mut seed) = if ed {
        let (pair, seed) = ed25519::Pair::generate();
        (Key::Ed(Box::new(pair)), seed)
    } else {
        let (pair, seed) = sr25519::Pair::generate();
        (Key::Sr(Box::new(pair)), seed)
    };
    // Kept in memory only for the post-run leak scan, then wiped.
    seeds.push(seed);
    wipe(&mut seed);
    Custodian { key, form }
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

    let mut seeds = Vec::new();
    let mut labels = vec!["sudo".to_owned(), "usdtOwner".to_owned()];
    labels.extend(PALLETS.iter().map(|pallet| format!("admin/{pallet}")));
    // Each role is 2-of-3: sr25519 raw, sr25519 bytes-wrapped, ed25519 raw, so the
    // fixture exercises both schemes and both message forms.
    let mut roles: Vec<(String, Vec<Custodian>, [u8; 32])> = labels
        .into_iter()
        .map(|label| {
            let mut custodians = vec![
                custodian(false, PopMessageForm::Raw, &mut seeds),
                custodian(false, PopMessageForm::BytesWrapped, &mut seeds),
                custodian(true, PopMessageForm::Raw, &mut seeds),
            ];
            custodians.sort_by_key(Custodian::public);
            let keys: Vec<[u8; 32]> = custodians.iter().map(Custodian::public).collect();
            let address = multisig_account(&keys, 2);
            (label, custodians, address)
        })
        .collect();

    let ss58 = |bytes: [u8; 32]| Address::from_account_id(bytes.into()).as_str().to_owned();
    let mut out_roles = Vec::new();
    for (label, custodians, address) in &roles {
        let signatories: Vec<Value> = custodians
            .iter()
            .map(|custodian| {
                let message = custody_pop_message(
                    network,
                    chain_id,
                    label,
                    address,
                    &custodian.public(),
                    &nonce,
                );
                json!({
                    "address": ss58(custodian.public()),
                    "evidence": { "signature": {
                        "scheme": serde_label(custodian.scheme()),
                        "message": serde_label(custodian.form),
                        "signature": hex(&custodian.sign(&message)),
                    }},
                })
            })
            .collect();
        out_roles.push(json!({
            "role": label, "address": ss58(*address), "threshold": 2, "signatories": signatories,
        }));
    }
    // A genuine proof by sudo's first signatory, but for the usdtOwner role and
    // multisig: valid signature, wrong role. Used by a negative conformance case.
    let (usdt_address, sudo_first) = (roles[1].2, &roles[0].1[0]);
    let other_role = custody_pop_message(
        network,
        chain_id,
        "usdtOwner",
        &usdt_address,
        &sudo_first.public(),
        &nonce,
    );
    let document = json!({
        "ceremonyNonce": hex(&nonce),
        "roles": out_roles,
        "alternates": { "sudoSignatory0AsUsdtOwner": hex(&sudo_first.sign(&other_role)) },
    });
    let bytes = serde_json::to_vec_pretty(&document).expect("document serializes");
    if let Err(error) = std::fs::write(out, &bytes) {
        eprintln!("{out}: {error}");
        return std::process::ExitCode::FAILURE;
    }
    roles.clear();

    let mut needles: Vec<Vec<u8>> = Vec::new();
    for seed in &seeds {
        needles.push(seed.to_vec());
        needles.push(hex(seed).into_bytes());
        needles.push(hex(seed).to_uppercase().into_bytes());
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
        needle.iter_mut().for_each(|b| *b = 0);
    }
    for seed in &mut seeds {
        wipe(seed);
    }
    println!(
        "wrote {out}: 14 roles, 42 signatories; seed leak scan: {} seeds x 3 encodings over {files} files, {matches} files matched; canary control matched {control_matches} of {control_files} files",
        seeds.len()
    );
    if matches == 0 && control_matches == 1 {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}
