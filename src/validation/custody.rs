use super::*;

/// Runtime `MaxSignatories`: d9-v2-node `runtime/src/configs/mod.rs`
/// (`ConstU32<20>` for pallet_multisig and d9-multisig-registry) and d9-v2-tools
/// `derive_admins::MAX_SIGNATORIES`. A larger set cannot exist on-chain.
pub const MULTISIG_MAX_SIGNATORIES: usize = 20;

// VERIFIED: public keys copied from sp-keyring 48.0.0 (the release depending on
// this crate's pinned sp-core =43.0.0), git 8ee7713ab4c1665ed777cfd52063c28b05a431ba,
// ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sp-keyring-48.0.0/src/sr25519.rs
// (sha256 2e95b38c42274fb3a6b567ca5c42d183a4f5f5942475fbaf28aaa6b3f012cf4c) and
// .../src/ed25519.rs (sha256 55ad030154c497510c991f09aea6adf8e92006c196078a280a48a8e2b8813541),
// `impl From<Keyring> for [u8; 32]`. The test below re-derives every entry from its
// seed URI with sp-core, so a mistyped byte fails the build.
// CHOICE: all 14 sp-keyring entries (the 12 `well_known()` accounts plus //One and
// //Two) because each is a publicly derivable development key; omitting two would
// only narrow the deny list.
const DEVELOPMENT_KEYS: [(&str, &str, [u8; 32]); 28] = [
    (
        "sr25519",
        "//Alice",
        hex32("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"),
    ),
    (
        "sr25519",
        "//Bob",
        hex32("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"),
    ),
    (
        "sr25519",
        "//Charlie",
        hex32("90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22"),
    ),
    (
        "sr25519",
        "//Dave",
        hex32("306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20"),
    ),
    (
        "sr25519",
        "//Eve",
        hex32("e659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e"),
    ),
    (
        "sr25519",
        "//Ferdie",
        hex32("1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c"),
    ),
    (
        "sr25519",
        "//Alice//stash",
        hex32("be5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f"),
    ),
    (
        "sr25519",
        "//Bob//stash",
        hex32("fe65717dad0447d715f660a0a58411de509b42e6efb8375f562f58a554d5860e"),
    ),
    (
        "sr25519",
        "//Charlie//stash",
        hex32("1e07379407fecc4b89eb7dbd287c2c781cfb1907a96947a3eb18e4f8e7198625"),
    ),
    (
        "sr25519",
        "//Dave//stash",
        hex32("e860f1b1c7227f7c22602f53f15af80747814dffd839719731ee3bba6edc126c"),
    ),
    (
        "sr25519",
        "//Eve//stash",
        hex32("8ac59e11963af19174d0b94d5d78041c233f55d2e19324665bafdfb62925af2d"),
    ),
    (
        "sr25519",
        "//Ferdie//stash",
        hex32("101191192fc877c24d725b337120fa3edc63d227bbc92705db1e2cb65f56981a"),
    ),
    (
        "sr25519",
        "//One",
        hex32("ac859f8a216eeb1b320b4c76d118da3d7407fa523484d0a980126d3b4d0d220a"),
    ),
    (
        "sr25519",
        "//Two",
        hex32("1254f7017f0b8347ce7ab14f96d818802e7e9e0c0d1b7c9acb3c726b080e7a03"),
    ),
    (
        "ed25519",
        "//Alice",
        hex32("88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee"),
    ),
    (
        "ed25519",
        "//Bob",
        hex32("d17c2d7823ebf260fd138f2d7e27d114c0145d968b5ff5006125f2414fadae69"),
    ),
    (
        "ed25519",
        "//Charlie",
        hex32("439660b36c6c03afafca027b910b4fecf99801834c62a5e6006f27d978de234f"),
    ),
    (
        "ed25519",
        "//Dave",
        hex32("5e639b43e0052c47447dac87d6fd2b6ec50bdd4d0f614e4299c665249bbd09d9"),
    ),
    (
        "ed25519",
        "//Eve",
        hex32("1dfe3e22cc0d45c70779c1095f7489a8ef3cf52d62fbd8c2fa38c9f1723502b5"),
    ),
    (
        "ed25519",
        "//Ferdie",
        hex32("568cb4a574c6d178feb39c27dfc8b3f789e5f5423e19c71633c748b9acf086b5"),
    ),
    (
        "ed25519",
        "//Alice//stash",
        hex32("451781cd0c5504504f69ceec484cc66e4c22a2b6a9d20fb1a426d91ad074a2a8"),
    ),
    (
        "ed25519",
        "//Bob//stash",
        hex32("292684abbb28def63807c5f6e84e9e8689769eb37b1ab130d79dbfbf1b9a0d44"),
    ),
    (
        "ed25519",
        "//Charlie//stash",
        hex32("dd6a6118b6c11c9c9e5a4f34ed3d545e2c74190f90365c60c230fa82e9423bb9"),
    ),
    (
        "ed25519",
        "//Dave//stash",
        hex32("1d0432d75331ab299065bee79cdb1bdc2497c597a3087b4d955c67e3c000c1e2"),
    ),
    (
        "ed25519",
        "//Eve//stash",
        hex32("c833bdd2e1a7a18acc1c11f8596e2e697bb9b42d6b6051e474091a1d43a294d7"),
    ),
    (
        "ed25519",
        "//Ferdie//stash",
        hex32("199d749dbf4b8135cb1f3c8fd697a390fc0679881a8a110c1d06375b3b62cd09"),
    ),
    (
        "ed25519",
        "//One",
        hex32("16f97016bbea8f7b45ae6757b49efc1080accc175d8f018f9ba719b60b0815e4"),
    ),
    (
        "ed25519",
        "//Two",
        hex32("5079bcd20fd97d7d2f752c4607012600b401950260a91821f73e692071c82bf5"),
    ),
];

const fn hex32(hex: &str) -> [u8; 32] {
    const fn nibble(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            _ => panic!("lowercase hex literal"),
        }
    }
    let bytes = hex.as_bytes();
    assert!(bytes.len() == 64, "32-byte hex literal");
    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = (nibble(bytes[2 * i]) << 4) | nibble(bytes[2 * i + 1]);
        i += 1;
    }
    out
}

/// Names the well-known Substrate development key (`"sr25519 //Alice"`) whose
/// public key equals `account`, if any. Shared so producers need not retype it.
pub fn well_known_development_key(account: &[u8; 32]) -> Option<String> {
    DEVELOPMENT_KEYS
        .iter()
        .find(|(_, _, key)| key == account)
        .map(|(scheme, uri, _)| format!("{scheme} {uri}"))
}

fn bytes(address: &Address) -> [u8; 32] {
    *address.account_id().as_ref()
}

// CHOICE: the deny list applies to every purpose, synthetic fixtures included.
// This contract has no development purpose: both purposes feed the native
// producer, and a fixture that passes with //Alice would train consumers to
// accept it. Development chains use the separate testnet chain-spec path.
fn not_development(address: &Address, path: String) -> Check {
    match well_known_development_key(&bytes(address)) {
        Some(name) => Err(fail(
            "dev_key_authority",
            path,
            format!("well-known development key ({name}) cannot hold or sign for authority"),
        )),
        None => Ok(()),
    }
}

/// DEC-21 k-of-n structure, mirroring d9-v2-tools `derive_admins::derive_role`.
/// The equality address == derive(signatories, threshold) is the producer's check.
fn multisig(authority: &MultisigAuthority, path: &str) -> Check {
    let n = authority.signatories.len();
    if authority.threshold < 2 {
        return Err(fail(
            "multisig_threshold_too_low",
            format!("{path}/threshold"),
            "DEC-21 requires threshold >= 2; 1-of-n is single-key authority",
        ));
    }
    if n > MULTISIG_MAX_SIGNATORIES {
        return Err(fail(
            "multisig_signatory_limit",
            format!("{path}/signatories"),
            format!("{n} signatories exceeds runtime MaxSignatories ({MULTISIG_MAX_SIGNATORIES})"),
        ));
    }
    if usize::from(authority.threshold) > n {
        return Err(fail(
            "multisig_threshold_exceeds_signatories",
            format!("{path}/threshold"),
            format!("threshold {} exceeds {n} signatories", authority.threshold),
        ));
    }
    let address = bytes(&authority.address);
    let mut previous: Option<[u8; 32]> = None;
    for (index, signatory) in authority.signatories.iter().enumerate() {
        let key = bytes(signatory);
        let signatory_path = format!("{path}/signatories/{index}");
        if previous.is_some_and(|previous| previous >= key) {
            return Err(fail(
                "multisig_signatory_order",
                signatory_path,
                "signatories must be unique and strictly ascending by AccountId32 bytes",
            ));
        }
        if key == address {
            return Err(fail(
                "multisig_self_signatory",
                signatory_path,
                "a multisig account cannot be its own signatory",
            ));
        }
        not_development(signatory, signatory_path)?;
        previous = Some(key);
    }
    not_development(&authority.address, format!("{path}/address"))
}

pub(super) fn check(i: &ContractInput) -> Check {
    let b = &i.bootstrap;
    multisig(&b.sudo, "/bootstrap/sudo")?;
    multisig(&b.usdt_owner, "/bootstrap/usdtOwner")?;
    for (index, role) in b.admins.iter().enumerate() {
        multisig(
            &role.multisig,
            &format!("/bootstrap/admins/{index}/multisig"),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::{ed25519, sr25519, Pair};

    #[test]
    fn deny_list_matches_sp_core_derivation_of_each_seed_uri() {
        let mut seen = std::collections::BTreeSet::new();
        for (scheme, uri, key) in DEVELOPMENT_KEYS {
            let derived: [u8; 32] = match scheme {
                "sr25519" => sr25519::Pair::from_string(uri, None).unwrap().public().0,
                "ed25519" => ed25519::Pair::from_string(uri, None).unwrap().public().0,
                _ => unreachable!(),
            };
            assert_eq!(derived, key, "{scheme} {uri}");
            assert!(seen.insert(key), "duplicate {scheme} {uri}");
        }
    }
}
