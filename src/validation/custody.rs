use super::*;
use std::collections::BTreeSet;

/// Runtime `MaxSignatories`: d9-v2-node `runtime/src/configs/mod.rs`
/// (`ConstU32<20>` for pallet_multisig and d9-multisig-registry) and d9-v2-tools
/// `derive_admins::MAX_SIGNATORIES`. A larger set cannot exist on-chain.
pub const MULTISIG_MAX_SIGNATORIES: usize = 20;

// VERIFIED: sr25519/ed25519 named keys copied from sp-keyring 48.0.0 (the release
// depending on this crate's pinned sp-core =43.0.0), git 8ee7713ab4c1665ed777cfd52063c28b05a431ba,
// ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sp-keyring-48.0.0/src/sr25519.rs
// (sha256 2e95b38c42274fb3a6b567ca5c42d183a4f5f5942475fbaf28aaa6b3f012cf4c) and
// .../src/ed25519.rs (sha256 55ad030154c497510c991f09aea6adf8e92006c196078a280a48a8e2b8813541),
// `impl From<Keyring> for [u8; 32]`.
// VERIFIED: `DEV_PHRASE` roots are the bare phrase from sp-core 43.0.0 `src/crypto.rs:45`.
// VERIFIED: ecdsa accounts are `blake2_256(compressed 33-byte public key)`, as
// sp-runtime 48.0.0 `src/lib.rs:402` (`MultiSigner::Ecdsa` into_account); sp-keyring
// has no ecdsa table, so these bytes were computed with sp-core `ecdsa::Pair::from_string`.
// Every entry is re-derived from its seed URI in the test below; a wrong byte fails.
// CHOICE: all 14 sp-keyring names (the 12 `well_known()` accounts plus //One and //Two)
// in each scheme, plus the bare DEV_PHRASE root in each scheme (the ecdsa root too), because
// each is publicly derivable; omitting any would only narrow the deny list.
const DEVELOPMENT_KEYS: [(&str, &str, [u8; 32]); 45] = [
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
        "sr25519",
        "DEV_PHRASE",
        hex32("46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a"),
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
    (
        "ed25519",
        "DEV_PHRASE",
        hex32("345071da55e5dccefaaa440339415ef9f2663338a38f7da0df21be5ab4e055ef"),
    ),
    (
        "ecdsa-account",
        "DEV_PHRASE",
        hex32("bc9539b36a87a586b1aa20fbe23a1db3ef3edcd65b44a2dc4444cc552687633f"),
    ),
    (
        "ecdsa-account",
        "//Alice",
        hex32("01e552298e47454041ea31273b4b630c64c104e4514aa3643490b8aaca9cf8ed"),
    ),
    (
        "ecdsa-account",
        "//Bob",
        hex32("3f6eaf1be5add88d84ca8b02d350074935dbf04f53f4287cb6abfd6b33413f8f"),
    ),
    (
        "ecdsa-account",
        "//Charlie",
        hex32("6672035dd6010e55e08eb707d171b5ec790bfea44f93ea8b1d22503033de45cd"),
    ),
    (
        "ecdsa-account",
        "//Dave",
        hex32("2452e6b66a46450f6858ef72934278407eb42de707c9abe67b8cb41707565ef3"),
    ),
    (
        "ecdsa-account",
        "//Eve",
        hex32("b71a0812a1edaa236625ba525279201448be05f3a97c02f796db63944bf29895"),
    ),
    (
        "ecdsa-account",
        "//Ferdie",
        hex32("fb5ead485f2af13bc4ad5ec43055c15cfe893ed543f8052f2db45cc5c73d0a30"),
    ),
    (
        "ecdsa-account",
        "//Alice//stash",
        hex32("0dcdff87f204fcf502c4cdd417ca73df9d06bf640d2f50af5fbe8225fe026b4b"),
    ),
    (
        "ecdsa-account",
        "//Bob//stash",
        hex32("8140beed032fe6592e6b19e905ac3146465fcb2cc01de4abd0fed2c13a8ae3e2"),
    ),
    (
        "ecdsa-account",
        "//Charlie//stash",
        hex32("62cb88bb1e8556d6ac19c961e601ae090a05fa8a414e76f407e0512603984e96"),
    ),
    (
        "ecdsa-account",
        "//Dave//stash",
        hex32("8455a656286fe7a47ac3820c75524b327a6c3d8b5c3d424176951c25ef743042"),
    ),
    (
        "ecdsa-account",
        "//Eve//stash",
        hex32("cb4267cf5374f84a104d5f74956c4660e747ec3556aa685bf101bd3f6eb39382"),
    ),
    (
        "ecdsa-account",
        "//Ferdie//stash",
        hex32("1506715124f1fd6b515f594ed37faf8b540a54e88fcea95c5e033459c3c4e6ee"),
    ),
    (
        "ecdsa-account",
        "//One",
        hex32("400098782d72cf06b10a0ca1e9324dcf2ca0a4b0028ecdeeb97f7108cee06ef7"),
    ),
    (
        "ecdsa-account",
        "//Two",
        hex32("ec6357524928b2ecf8f316aba8491b2f01dc74c96b590135d50f53f587e0117f"),
    ),
];

pub(super) const fn hex32(hex: &str) -> [u8; 32] {
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

/// Names the well-known Substrate development key (`"sr25519 //Alice"`,
/// `"ecdsa-account //Bob"`, `"ed25519 DEV_PHRASE"`) whose 32 bytes equal `key`,
/// if any. Shared so producers need not retype the deny list.
pub fn well_known_development_key(key: &[u8; 32]) -> Option<String> {
    DEVELOPMENT_KEYS
        .iter()
        .find(|(_, _, known)| known == key)
        .map(|(scheme, uri, _)| format!("{scheme} {uri}"))
}

fn bytes(address: &Address) -> [u8; 32] {
    *address.account_id().as_ref()
}

// CHOICE: the deny list applies to every purpose, synthetic fixtures included.
// This contract has no development purpose: both purposes feed the native
// producer, and a fixture that passes with //Alice would train consumers to
// accept it. Development chains use the separate testnet chain-spec path.
pub(super) fn not_development(key: &[u8; 32], path: String) -> Check {
    match well_known_development_key(key) {
        Some(name) => Err(fail(
            "dev_key_authority",
            path,
            format!(
                "well-known development key ({name}) cannot hold, sign for or validate authority"
            ),
        )),
        None => Ok(()),
    }
}

// VERIFIED: frame-support 48.0.0 `src/lib.rs:145-147`,
// `impl TypeId for PalletId { const TYPE_ID: [u8; 4] = *b"modl"; }`; sp-runtime's
// `AccountIdConversion` places TYPE_ID first. Every PalletId account starts with it.
const PALLET_ACCOUNT_PREFIX: &[u8; 4] = b"modl";

/// Accounts in this document that no multisig may be or contain as a signatory.
struct Reserved {
    roles: Vec<(String, [u8; 32])>,
    pallet_accounts: [[u8; 32]; 2],
    validators: BTreeSet<[u8; 32]>,
}

impl Reserved {
    fn identity(&self, key: &[u8; 32], path: String) -> Check {
        if self.pallet_accounts.contains(key) || key.starts_with(PALLET_ACCOUNT_PREFIX) {
            return Err(fail(
                "authority_pallet_account",
                path,
                "a keyless PalletId account (ammAccount, miningPoolAccount or any b\"modl\" account) cannot be or sign for an authority",
            ));
        }
        if self.validators.contains(key) {
            return Err(fail(
                "authority_validator_account",
                path,
                "a validator account cannot be or sign for a multisig authority",
            ));
        }
        Ok(())
    }
}

/// DEC-21 k-of-n structure, mirroring d9-v2-tools `derive_admins::derive_role`.
/// The equality address == derive(signatories, threshold) is the producer's check.
fn multisig(authority: &MultisigAuthority, path: &str, reserved: &Reserved) -> Check {
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
        not_development(&key, signatory_path.clone())?;
        // Nesting outside this document (a signatory that is some other
        // multisig) is undetectable here; it is ceremony evidence (rule CUSTODY).
        if let Some((role, _)) = reserved
            .roles
            .iter()
            .find(|(role, role_address)| role.as_str() != path && *role_address == key)
        {
            let mut error = fail(
                "multisig_nested_signatory",
                signatory_path,
                "a signatory cannot be another authority's multisig address",
            );
            error.related_path = Some(format!("{role}/address"));
            return Err(error);
        }
        reserved.identity(&key, signatory_path)?;
        previous = Some(key);
    }
    let address_path = format!("{path}/address");
    not_development(&address, address_path.clone())?;
    reserved.identity(&address, address_path)
}

pub(super) fn check(i: &ContractInput) -> Check {
    let b = &i.bootstrap;
    let mut authorities = vec![
        ("/bootstrap/sudo".to_owned(), &b.sudo),
        ("/bootstrap/usdtOwner".to_owned(), &b.usdt_owner),
    ];
    for (index, role) in b.admins.iter().enumerate() {
        authorities.push((
            format!("/bootstrap/admins/{index}/multisig"),
            &role.multisig,
        ));
    }
    let reserved = Reserved {
        roles: authorities
            .iter()
            .map(|(path, authority)| (path.clone(), bytes(&authority.address)))
            .collect(),
        pallet_accounts: [bytes(&b.amm_account), bytes(&b.mining_pool_account)],
        validators: b.validators.iter().map(|v| bytes(&v.account)).collect(),
    };
    for (path, authority) in &authorities {
        multisig(authority, path, &reserved)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::{crypto::DEV_PHRASE, ecdsa, ed25519, sr25519, Pair};
    use sp_runtime::traits::IdentifyAccount;

    #[test]
    fn deny_list_matches_sp_core_derivation_of_each_seed_uri() {
        let mut seen = BTreeSet::new();
        for (scheme, uri, key) in DEVELOPMENT_KEYS {
            let suri = if uri == "DEV_PHRASE" { DEV_PHRASE } else { uri };
            let derived: [u8; 32] = match scheme {
                "sr25519" => sr25519::Pair::from_string(suri, None).unwrap().public().0,
                "ed25519" => ed25519::Pair::from_string(suri, None).unwrap().public().0,
                "ecdsa-account" => {
                    let public = ecdsa::Pair::from_string(suri, None).unwrap().public();
                    let account = sp_runtime::MultiSigner::from(public).into_account();
                    *AsRef::<[u8; 32]>::as_ref(&account)
                }
                _ => unreachable!(),
            };
            assert_eq!(derived, key, "{scheme} {uri}");
            assert!(seen.insert(key), "duplicate {scheme} {uri}");
        }
        for scheme in ["sr25519", "ed25519", "ecdsa-account"] {
            let uris: BTreeSet<_> = DEVELOPMENT_KEYS
                .iter()
                .filter(|(s, _, _)| *s == scheme)
                .map(|(_, uri, _)| *uri)
                .collect();
            assert_eq!(
                uris.len(),
                15,
                "{scheme}: 14 sp-keyring names plus DEV_PHRASE"
            );
        }
    }
}
