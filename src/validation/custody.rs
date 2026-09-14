use super::pop;
use super::*;
use std::collections::BTreeSet;

/// Runtime `MaxSignatories`: d9-v2-node `runtime/src/configs/mod.rs`
/// (`ConstU32<20>` for pallet_multisig and d9-multisig-registry) and d9-v2-tools
/// `derive_admins::MAX_SIGNATORIES`. A larger set cannot exist on-chain.
pub const MULTISIG_MAX_SIGNATORIES: usize = 20;

/// The `pallet_multisig` account of `signatories` at `threshold`:
/// `blake2_256(SCALE(b"modlpy/utilisuba", sorted Vec<AccountId32>, threshold as u16))`.
/// Input order does not matter; the signatories are sorted by bytes first, as
/// `pallet_multisig` requires of its callers. Uniqueness and bounds are not
/// checked here; CUSTODY validation enforces them separately.
///
/// This is intentionally the second of two implementations. d9-v2-tools
/// `d9-bootstrap derive-admins` keeps its own small copy because the key-generation
/// binary must not depend on this crate (and through it on sp-core). Both are pinned
/// to the same `@polkadot/util-crypto` golden vectors, and d9-v2-tools
/// `d9-genesis-composition` `custody::cross_check_derivations` (tools PR #72, not yet
/// merged) cross-checks the two.
// VERIFIED: pallet-multisig 45.0.0 (the version d9-v2-node e13a19d `Cargo.lock` pins)
// `src/lib.rs:634-638`, `multi_account_id`; 48.0.0 `src/lib.rs:645-649` is byte-identical:
// `(b"modlpy/utilisuba", who, threshold).using_encoded(blake2_256)` decoded through
// `TrailingZeroInput` into a 32-byte AccountId, i.e. the hash bytes themselves.
// `BlakeTwo256::hash_of` is `Encode::using_encoded(s, sp_io::hashing::blake2_256)`
// (sp-runtime 48.0.0 `src/traits/mod.rs:1009-1011, 1073-1075`). `[u8; 32]` and
// AccountId32 SCALE-encode identically (32 raw bytes, no length prefix).
// CHOICE: one expression over sp-runtime's hasher and codec (already dependencies)
// instead of depending on pallet-multisig, which is not in this crate's tree and would
// pull a FRAME pallet into a host-side contract crate for a single hash.
pub fn multisig_account(signatories: &[[u8; 32]], threshold: u16) -> [u8; 32] {
    use sp_runtime::traits::{BlakeTwo256, Hash};
    let mut sorted = signatories.to_vec();
    sorted.sort_unstable();
    BlakeTwo256::hash_of(&(b"modlpy/utilisuba", &sorted[..], threshold)).0
}

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
// VERIFIED: d9's own committed authority SURIs, from d9-v2-node origin/main e13a19d:
// - `//LocalValidator1`..`//LocalValidator6`: the `local_dev` preset authorities,
//   `runtime/src/genesis_config_presets.rs:144-163` (`LOCAL_DEV_SR25519_PUBS`,
//   `LOCAL_DEV_ED25519_PUBS`) and `local-keys/README.md` / `generate-keystores.sh:57`;
//   the test below also asserts equality with those node constants.
// - `//Mainnet{0..5}//{stash,babe,imon,audi,live,grandpa}`: `distinct_authority` in the
//   same file (`:1531-1550`), the mainnet-shaped test authority set.
// - `//OCWTest//{Babe,Grandpa,Liveness,Discovery}`: `runtime/src/ocw_signing_tests.rs:26-38`.
// Searched and not added: `legacy/chain_spec.rs:126` seeds `//` (not a valid SURI in
// sp-core 43.0.0); d9-v2-node `from_seed(&[..])` raw-byte test seeds (not SURIs);
// d9-v2-tools a5938f7 `admin-keys.md` hardware derivation paths (secret roots). The
// remaining node SURIs (`//Alice`..`//Dave`, `//Bob`) are already sp-keyring entries.
// Every entry is re-derived from its seed URI in the test below; a wrong byte fails.
// CHOICE: every listed URI in all three schemes (sr25519, ed25519, ecdsa account),
// even where the source used one scheme, because each is publicly derivable and a
// uniform rule is easier to audit than per-URI scheme selection.
const DEVELOPMENT_KEYS: [(&str, &str, [u8; 32]); 183] = [
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
    (
        "sr25519",
        "//LocalValidator1",
        hex32("8e5052a337be646ae18ef81eee5d820403671cbf79090ae80641c40a47698d3a"),
    ),
    (
        "ed25519",
        "//LocalValidator1",
        hex32("43d1b4c81c4dd6ba8e7d3ad8b883818858a5e2671a069d3e58ce4d36bce6725b"),
    ),
    (
        "ecdsa-account",
        "//LocalValidator1",
        hex32("de8fd0716fc5eee8091744e0925555bfd2357c901c8f354d99db64b5239f69b8"),
    ),
    (
        "sr25519",
        "//LocalValidator2",
        hex32("3c225fc7fe1b1fea0f8109f20abaa48346c785b83311b93d66d23eabbec64c33"),
    ),
    (
        "ed25519",
        "//LocalValidator2",
        hex32("7dc507114268649fa8881d45573e6d2925890dc5f081d324b54400a22cc08415"),
    ),
    (
        "ecdsa-account",
        "//LocalValidator2",
        hex32("e6e5f387685e8f242cb0f89be28c5fa836fa37bbd5c8c5dcc8dc3c4b7cb5ad09"),
    ),
    (
        "sr25519",
        "//LocalValidator3",
        hex32("ae48fa9d973ec2cea3d48da7657d3061451e54c9d44fcfda3b19204fa2487040"),
    ),
    (
        "ed25519",
        "//LocalValidator3",
        hex32("e8359425fbd47f877ce6718009ce46ccba1953d5bc378085d05ea427e5888791"),
    ),
    (
        "ecdsa-account",
        "//LocalValidator3",
        hex32("c9c5ef08738f1a33f5ef39f2c217bcaef814195f6d1af20fff01c67a15f42a75"),
    ),
    (
        "sr25519",
        "//LocalValidator4",
        hex32("206f7539bde2fa9182654152271d84c0ebe78bb7c5071ab35162998230773464"),
    ),
    (
        "ed25519",
        "//LocalValidator4",
        hex32("0c9148a02698491cf4751fb99accc5f3f336b63be458cdc9ce6fb49af3deed4e"),
    ),
    (
        "ecdsa-account",
        "//LocalValidator4",
        hex32("aa5d8ce8aecf1f0e46ca051c4b9b416729886c7e4917bc8f851b6e7bb7f13bce"),
    ),
    (
        "sr25519",
        "//LocalValidator5",
        hex32("588d0a678cbfa52557ba35143cabc8992bba88c7e6c2ba722b47e224a99d3925"),
    ),
    (
        "ed25519",
        "//LocalValidator5",
        hex32("fb36512092c124f2982792e1f5a7d468de521d6fda4d0ecd4497db5181ca9832"),
    ),
    (
        "ecdsa-account",
        "//LocalValidator5",
        hex32("935b4ca295908564effc36a1520d33de3d3abafd5fc2e8640ae9334e13ef16b3"),
    ),
    (
        "sr25519",
        "//LocalValidator6",
        hex32("60e107d1d885a2cdbee8fdc4c8e9b5560566b4f639eacc54ebb577d9515c6c7b"),
    ),
    (
        "ed25519",
        "//LocalValidator6",
        hex32("3f62dca0a1fa14648de3aae69b49aba26ccd08d5a3ec5ceb575781f011dc98f2"),
    ),
    (
        "ecdsa-account",
        "//LocalValidator6",
        hex32("3b5b56445045af626f43193286c559a726f0446a20917c25020399b425d0bcb1"),
    ),
    (
        "sr25519",
        "//Mainnet0//stash",
        hex32("18deec92bec3ffad01088e9c315479d1f0e3b2575e68cfe9687f17bcfc68074c"),
    ),
    (
        "ed25519",
        "//Mainnet0//stash",
        hex32("671f27541c8688c941f8cb34ddbefd353e541bf57fff3b5998f8d153d8780ff4"),
    ),
    (
        "ecdsa-account",
        "//Mainnet0//stash",
        hex32("4ce2e93922c586cb767e485da5fc69bfa8675327802ff24a653008d41e5bd25e"),
    ),
    (
        "sr25519",
        "//Mainnet0//babe",
        hex32("04f3d0def83998208166a04edd3f2e7c3aff5819db0c996ac3771070e24ba67e"),
    ),
    (
        "ed25519",
        "//Mainnet0//babe",
        hex32("7715f28902e047a04248b30bc14e2f2a131ce770097fbce3b708a9b35efb8a27"),
    ),
    (
        "ecdsa-account",
        "//Mainnet0//babe",
        hex32("28f77bb6304c57b8e02e45e361e18628ebcce2bfb81f54135182565129dd16e2"),
    ),
    (
        "sr25519",
        "//Mainnet0//imon",
        hex32("e4c64bd7444f942ca9cfd8bc82b9f08dfc8c3cffc05e6cf130cd9e0bd0733d2e"),
    ),
    (
        "ed25519",
        "//Mainnet0//imon",
        hex32("f1182a4d5a76c52b4eb3352e27b31cb7f9247ecbb64e93793420d0bf008ac2e7"),
    ),
    (
        "ecdsa-account",
        "//Mainnet0//imon",
        hex32("66dc97c10570618f9976929e38c5f798b62078638a64f57ad6e2afd8e087b60d"),
    ),
    (
        "sr25519",
        "//Mainnet0//audi",
        hex32("5267a3dd7b12ccef6af564aaf7bf05f02948e19e8935fd4e26d85828802c923d"),
    ),
    (
        "ed25519",
        "//Mainnet0//audi",
        hex32("171b12ca5f6d718fead7751b989f10c78c8457cb1b3e4cd096297fa73774d81e"),
    ),
    (
        "ecdsa-account",
        "//Mainnet0//audi",
        hex32("0c8f095091a62f3492e640f2f5f2048d50ddccf268958b08b4711354d8f3ba68"),
    ),
    (
        "sr25519",
        "//Mainnet0//live",
        hex32("fe46dca20ffcc8558aca89c48b904c49d57dcf38d526d7948bd601b8d5599673"),
    ),
    (
        "ed25519",
        "//Mainnet0//live",
        hex32("1c4a55a4e028c90a852d3e39fdfa69a05f4908590a12d339615c333325869204"),
    ),
    (
        "ecdsa-account",
        "//Mainnet0//live",
        hex32("06fc0a90f435ed9cc44fffb3cf8fbf244275ec75420ff4b32d568bf084c18190"),
    ),
    (
        "sr25519",
        "//Mainnet0//grandpa",
        hex32("8ab3d0d27f6cc3326f8879b77da32e3d9fe5b5f60973271d925fb823b07a7a31"),
    ),
    (
        "ed25519",
        "//Mainnet0//grandpa",
        hex32("9731bde6d6cd3aad948ad60fa9f94183c5c5d3ade167cefe048cd6a8c1f21584"),
    ),
    (
        "ecdsa-account",
        "//Mainnet0//grandpa",
        hex32("8db868387994438deda5fdc206ce9dae2ffcd85b23f1027d03270a29b5289905"),
    ),
    (
        "sr25519",
        "//Mainnet1//stash",
        hex32("286019a339680593f57f28c0bfd38bf85db31e9a960c53760591d350abc0a75b"),
    ),
    (
        "ed25519",
        "//Mainnet1//stash",
        hex32("e93b568d5b418a4062c096fdd2ac8f934ba4b31b09d919cc54c07c1c5bd05849"),
    ),
    (
        "ecdsa-account",
        "//Mainnet1//stash",
        hex32("f83b0e71fc6e6f6e6c07af82bbb871f9203c90d772c8e81aa691806e02be7948"),
    ),
    (
        "sr25519",
        "//Mainnet1//babe",
        hex32("be32bdedacc5120f0eb2c1ca4969b30f72f16e7c71696f771c081fd8bd0b2f17"),
    ),
    (
        "ed25519",
        "//Mainnet1//babe",
        hex32("8ebfda9accefde1b1328efa65d6db330a5109a3a7f7c2b3211f4a4cff4936721"),
    ),
    (
        "ecdsa-account",
        "//Mainnet1//babe",
        hex32("89ae855a0c018b2933877c8d8009da4b6e8e8f7a48a7fea4a765b0811be9b674"),
    ),
    (
        "sr25519",
        "//Mainnet1//imon",
        hex32("6cec7aef514914c0415ad2c2eeecd8becfe8986ba24225c955c5900eed61da12"),
    ),
    (
        "ed25519",
        "//Mainnet1//imon",
        hex32("05a0b14a6ec2af539d53b6309600c0c57c21b5b5db8ea0bd62dd40f8e01e52d0"),
    ),
    (
        "ecdsa-account",
        "//Mainnet1//imon",
        hex32("66ed9cc79feb5bfdc10773a207e30801dfde2539e64f48bf6e6a71cd029c3813"),
    ),
    (
        "sr25519",
        "//Mainnet1//audi",
        hex32("8c219683af2f8cd67179faa6e79b331f57ff67799ce86d69a95596fbf40e1d08"),
    ),
    (
        "ed25519",
        "//Mainnet1//audi",
        hex32("ec87a68349d06302bdb21f79493a30acc78f4f524f0ca81da8780267116bbc66"),
    ),
    (
        "ecdsa-account",
        "//Mainnet1//audi",
        hex32("6eeb42a55279a806e5fa38e548b8a33aea3bf081aec2c2f7ab66d79d72e5a863"),
    ),
    (
        "sr25519",
        "//Mainnet1//live",
        hex32("e4aa7fd324e786c9bc49206825f204bf59a3427c9dccb17346f7b023175db039"),
    ),
    (
        "ed25519",
        "//Mainnet1//live",
        hex32("fe914d8a49c93b8db069ea42a506ba8b98a460f357b5abc28107aacdb95885b5"),
    ),
    (
        "ecdsa-account",
        "//Mainnet1//live",
        hex32("52629b7963dd0ed7225ff3b21890355c062c39f969a386ef2606d085d0c2611f"),
    ),
    (
        "sr25519",
        "//Mainnet1//grandpa",
        hex32("c04023e2072d2d3bc2ca4e6e68b584a9ecac4a79208b169e90dc73c052ccf317"),
    ),
    (
        "ed25519",
        "//Mainnet1//grandpa",
        hex32("49bf3915d7fd115f556fd838f3f6abc331aa547722bbd6eff46703e9b3b52738"),
    ),
    (
        "ecdsa-account",
        "//Mainnet1//grandpa",
        hex32("bf41269f6d7d8c715ebd299932ccc4d150eac36a4d4d9f9efd25bea223fc7313"),
    ),
    (
        "sr25519",
        "//Mainnet2//stash",
        hex32("04a62cbd88a75815e689f86a50293fb92abd6e63b19dd8f32fbf15646467c565"),
    ),
    (
        "ed25519",
        "//Mainnet2//stash",
        hex32("d4793d753cc17d6112563d1ed94184dd19ccb1c419e430373bd8a5d3a603ad2e"),
    ),
    (
        "ecdsa-account",
        "//Mainnet2//stash",
        hex32("3c4368f025edb6624f8b4ea684c22b4e4968a0a649cbce70f9a236ce194a1d73"),
    ),
    (
        "sr25519",
        "//Mainnet2//babe",
        hex32("30558bc32b366a41b1ccd1d954ce773bd8d6b3b7fb96c8ae038260726a386f2d"),
    ),
    (
        "ed25519",
        "//Mainnet2//babe",
        hex32("53c3abd2431f38f89be4852b40079233d26415f97ea4535e3d6db65f6bb075bd"),
    ),
    (
        "ecdsa-account",
        "//Mainnet2//babe",
        hex32("a3574a3310ce0a7876c169b7ae70bfff0c46c4c9e0074b88d4b3e532fc4d3bae"),
    ),
    (
        "sr25519",
        "//Mainnet2//imon",
        hex32("22e45a4e821063cc9460ad7c4f8b2c1d82228e1767d267300c642d60a7724740"),
    ),
    (
        "ed25519",
        "//Mainnet2//imon",
        hex32("b61ef9505bc87436c2222d18a8333d07e50dfd4169d247fc6ff1416be21c944f"),
    ),
    (
        "ecdsa-account",
        "//Mainnet2//imon",
        hex32("9e487714ec7bc344de54364bd94744a8c5ae0252a149ea11fd00b2d7a48f2359"),
    ),
    (
        "sr25519",
        "//Mainnet2//audi",
        hex32("94018cd49a35b9aa3f968d0e2795e88703b76aa61e1d92bed8a55fadf8e8477a"),
    ),
    (
        "ed25519",
        "//Mainnet2//audi",
        hex32("3f5cab68c5bba742b2e809e645355c870deac0b71200c62f25706313ccbbcc3f"),
    ),
    (
        "ecdsa-account",
        "//Mainnet2//audi",
        hex32("6fc08e529e0b4ccd52a977ace6650408bee7182ae1d05a813fea41a9c1b50491"),
    ),
    (
        "sr25519",
        "//Mainnet2//live",
        hex32("967195ef08facb46c16dc020686dfc0b9487991db3174cdb4fb8b3a5cb344e1e"),
    ),
    (
        "ed25519",
        "//Mainnet2//live",
        hex32("8f9134813cbb29ac044020e09637ffb58e861f48bc3ae013bc03a02015818951"),
    ),
    (
        "ecdsa-account",
        "//Mainnet2//live",
        hex32("2185deecf010586ba44c07897ce482377b87ec5aab11b9df827fcc6ca09ac7ee"),
    ),
    (
        "sr25519",
        "//Mainnet2//grandpa",
        hex32("ee24d12497db1bef21dab9323bf4e3b5a4f32f85dfd5241d9656762105b34d44"),
    ),
    (
        "ed25519",
        "//Mainnet2//grandpa",
        hex32("602081ffb09ae665505345ff1b004f55e66d64310dbb138b6a48da2c7fe6ab89"),
    ),
    (
        "ecdsa-account",
        "//Mainnet2//grandpa",
        hex32("46e0e9892557d9e756f5f6da7999d7ab47e62bc11edea5745f8b30d2287503ba"),
    ),
    (
        "sr25519",
        "//Mainnet3//stash",
        hex32("461e5439c8036b366ab934fe9bf8be40d4cc182823abc1367fd3b374a4ebe83e"),
    ),
    (
        "ed25519",
        "//Mainnet3//stash",
        hex32("46bb29264c5da86d2f7e26870cd9ad690eb8b02387d4f9ec37f68dd5bf704ac4"),
    ),
    (
        "ecdsa-account",
        "//Mainnet3//stash",
        hex32("ef4ed511cb2d50f41571ff86711e495af7b7b19844d5b15d6b2c67ff49346200"),
    ),
    (
        "sr25519",
        "//Mainnet3//babe",
        hex32("9ed678d85a1ee10316eb40f195e919483fcd3ff961425e731ae9ec9f1657c172"),
    ),
    (
        "ed25519",
        "//Mainnet3//babe",
        hex32("595aaef78d41dc3bcec50a0890d7a8015cde974e38760a4105ab26e2ec67e92a"),
    ),
    (
        "ecdsa-account",
        "//Mainnet3//babe",
        hex32("e6a123ec0bd7aff13fbdc0f252a14ae2c574452b1b67975f2b91cf7e6889734f"),
    ),
    (
        "sr25519",
        "//Mainnet3//imon",
        hex32("9ed58ae73cff896c0f91b5b7cd5314d6113117bae8407ce3fedbd14dd0e11050"),
    ),
    (
        "ed25519",
        "//Mainnet3//imon",
        hex32("2bc4bcc09a623ec6a3e5f7c4f089a4b8926d2e997124b6f86fa0cc130bd25e9c"),
    ),
    (
        "ecdsa-account",
        "//Mainnet3//imon",
        hex32("ff35ec37ad38c326291e7d77ef348ff0ebe705a1ab5be6c95f8dea894f4a697d"),
    ),
    (
        "sr25519",
        "//Mainnet3//audi",
        hex32("5438f750b287e53a679f9994d37190eb189f6e85173d8a4afc1197070d1e5769"),
    ),
    (
        "ed25519",
        "//Mainnet3//audi",
        hex32("173c230b1d8e49bed47350528df0921f67930e0f18647af3fe3236a98023290b"),
    ),
    (
        "ecdsa-account",
        "//Mainnet3//audi",
        hex32("9813d5a07885cee46e3edb2ec1fba0b9047c3ddb176ee0dc6022c39b2e13ca64"),
    ),
    (
        "sr25519",
        "//Mainnet3//live",
        hex32("7613cde3170ddf506f33646a166c0b470ec1fd17c72c23d1172a5445d417e015"),
    ),
    (
        "ed25519",
        "//Mainnet3//live",
        hex32("83ef379d14e5131e385f6a72b528fd0fa649979d7af9a2ae1e35f138efeda74b"),
    ),
    (
        "ecdsa-account",
        "//Mainnet3//live",
        hex32("ea887c67e2010be408182d6a62213e48eb67caad61ed1bdb827ecb67299c411f"),
    ),
    (
        "sr25519",
        "//Mainnet3//grandpa",
        hex32("0c07916d27160092c1f77d90dc358c227b59ca513a4bbc1cb41f9998e623b262"),
    ),
    (
        "ed25519",
        "//Mainnet3//grandpa",
        hex32("76afffdc4631084d89b4029771aa36a9104bd89a13fb636d736e1c356b338702"),
    ),
    (
        "ecdsa-account",
        "//Mainnet3//grandpa",
        hex32("da1cd983c51b6e4e3b2b9dd79ada09db390bd71ef9176d3bba43cf7018d748eb"),
    ),
    (
        "sr25519",
        "//Mainnet4//stash",
        hex32("20920d6217e223d28ea86da23fed07050549518c0cac3c8f3e4a3b1b5dc77417"),
    ),
    (
        "ed25519",
        "//Mainnet4//stash",
        hex32("f8054d4f7c64138c3540c5c7bf9a41ba155d5d24228b83e464b430f6e7bab74d"),
    ),
    (
        "ecdsa-account",
        "//Mainnet4//stash",
        hex32("50ab666e8ce04da02a8e6ae1477c6bf5191bd3d76053cb432507bbd288cb48cb"),
    ),
    (
        "sr25519",
        "//Mainnet4//babe",
        hex32("80e2027d1435061bf030b5b825122995ad37b99478738a4ac3177908a53c656d"),
    ),
    (
        "ed25519",
        "//Mainnet4//babe",
        hex32("f3801ee708a85d96c95c0997cf8fb9b07ea7ba1f8e4a4beed8b7a498b5dbe9fc"),
    ),
    (
        "ecdsa-account",
        "//Mainnet4//babe",
        hex32("d68633bbe6d6dc542a9fb60c6f89a1d2e2cad523b6aa9e56e5505496433f5595"),
    ),
    (
        "sr25519",
        "//Mainnet4//imon",
        hex32("a6a1fa402579b5306fcd100b09f8ed593b56562996480b2591923e79ba3ad808"),
    ),
    (
        "ed25519",
        "//Mainnet4//imon",
        hex32("ee91972adbbcd13ae2d3d89ab1d85fd455cea6cedba4c68b38734fb515e0abfe"),
    ),
    (
        "ecdsa-account",
        "//Mainnet4//imon",
        hex32("d2fe6cf351a9a112968ba1e4f43e5a1b565137916378e50ecd5f58b307b93d3a"),
    ),
    (
        "sr25519",
        "//Mainnet4//audi",
        hex32("3eb99f840e0b9d0fddb9905b8b5c385f1e55e63fb10ec8a1c3bae33b6add142c"),
    ),
    (
        "ed25519",
        "//Mainnet4//audi",
        hex32("7c3f94312b85bbc4c8397fab3d506e24ea7f6cc4fff231f6c479d6aad4766473"),
    ),
    (
        "ecdsa-account",
        "//Mainnet4//audi",
        hex32("8ffc838aaca8daa22c833a5e3d4ecc010135d8c597c42b994c265c549f8b88db"),
    ),
    (
        "sr25519",
        "//Mainnet4//live",
        hex32("b6e77ba840e6df87c78e11fa83a8e45e9decc0265d155aa687d20eb30d3ae613"),
    ),
    (
        "ed25519",
        "//Mainnet4//live",
        hex32("52cc924c0adb5139b5d67c0bac0e42966ce9108d18315f92747c796f82c304aa"),
    ),
    (
        "ecdsa-account",
        "//Mainnet4//live",
        hex32("6bf864b2e0baffe79c126808669110514ea8dcb9b961db77569d7d749517cd44"),
    ),
    (
        "sr25519",
        "//Mainnet4//grandpa",
        hex32("f0e100ef0a37c83cc9e4ab0d32209622aa583450f9206c84fb03348d0257782c"),
    ),
    (
        "ed25519",
        "//Mainnet4//grandpa",
        hex32("7cb0e1ccd260712bb5bf9af609da00a965785e18822c0c81177172242e3dee8f"),
    ),
    (
        "ecdsa-account",
        "//Mainnet4//grandpa",
        hex32("dbae894ef9359830373f42666e25488001842f9c93957ffbc3890fb5496a0085"),
    ),
    (
        "sr25519",
        "//Mainnet5//stash",
        hex32("7ed7b9cbd38b58a408770f0cb46daecf4c84d737a16466b18194bb966fb86d38"),
    ),
    (
        "ed25519",
        "//Mainnet5//stash",
        hex32("cbf9a0cdb36e187c6b1272219ffe4b0fbab736511878df14b2301c29b356ba13"),
    ),
    (
        "ecdsa-account",
        "//Mainnet5//stash",
        hex32("2fb65840ca896fd1a2782ac4aa900e3b1ccc102020b91c8778c05055f6c72403"),
    ),
    (
        "sr25519",
        "//Mainnet5//babe",
        hex32("3ca6dc9db184d47290404d650d8d6e6687d8cd11530739e206948553cd738a56"),
    ),
    (
        "ed25519",
        "//Mainnet5//babe",
        hex32("963c2a2400fe288db7a31c8b8950d2129794e4c1f3db415e2e93718274feb534"),
    ),
    (
        "ecdsa-account",
        "//Mainnet5//babe",
        hex32("472589f1f2dea8707517e18339f3f169fdd877ba5ffd6ba5ec9c856c3a80c62f"),
    ),
    (
        "sr25519",
        "//Mainnet5//imon",
        hex32("4e589f44d0c2669369f2483668a9855959847d2c8854a0ec8466efb269b7e602"),
    ),
    (
        "ed25519",
        "//Mainnet5//imon",
        hex32("ebb07c25f4f07c6d5b14048157a6a14a583602c95cfbe485cfa44d4774b4653b"),
    ),
    (
        "ecdsa-account",
        "//Mainnet5//imon",
        hex32("0645962e8d2cb467d10af0c8d85c95d7ed0fee1e045680bfb071302f9f0e98c4"),
    ),
    (
        "sr25519",
        "//Mainnet5//audi",
        hex32("f4d27d75a2da1f1ebddc330264d83430fd83ff41109e89a2ec3c20999bc2dc1a"),
    ),
    (
        "ed25519",
        "//Mainnet5//audi",
        hex32("58c0f1090035cc8eab32b009bf1cda465d31710af4c4a7bb0dffff2f7ebe0920"),
    ),
    (
        "ecdsa-account",
        "//Mainnet5//audi",
        hex32("eca6911b0910fd7ba65bc78c2b022420e75307a7a6ff2ce1f8afd4bf9d86a5f7"),
    ),
    (
        "sr25519",
        "//Mainnet5//live",
        hex32("90d1653574fd01f78456032dc90bf7d039c93849fd41c129d418b0b7b873777a"),
    ),
    (
        "ed25519",
        "//Mainnet5//live",
        hex32("03a5aee8af6c43bcf3320cab224427b343ecfde2ff692ff7a60c949c5942e3d2"),
    ),
    (
        "ecdsa-account",
        "//Mainnet5//live",
        hex32("8bfb205387b13b2b8111931c1b1551d23e9367cd26204b2e8a18b4532f95fd1d"),
    ),
    (
        "sr25519",
        "//Mainnet5//grandpa",
        hex32("3af4bf135756921c153161b595d73a518f702019b863b82f7ab709ad3ab47a65"),
    ),
    (
        "ed25519",
        "//Mainnet5//grandpa",
        hex32("fd0de3d06441d8f4c0b0df988439e502653033feb5e4074b6e5f6c6187aaa0f8"),
    ),
    (
        "ecdsa-account",
        "//Mainnet5//grandpa",
        hex32("1ac905b0a8ed68a49645908d00c7695c90c32ff0bcafb743be52a1533f97e721"),
    ),
    (
        "sr25519",
        "//OCWTest//Babe",
        hex32("ae6f45796011a8eae8451a43a0d6f1188c0534553167c05df8d13a2ad7707a19"),
    ),
    (
        "ed25519",
        "//OCWTest//Babe",
        hex32("8e510967315ea01ee270debc04771bc61b6ea18148a1e9f80b657fa2cccd288e"),
    ),
    (
        "ecdsa-account",
        "//OCWTest//Babe",
        hex32("db827fce7e883032e23666c170bc144a1dedebeabbf3cd378cef8e92c11255e2"),
    ),
    (
        "sr25519",
        "//OCWTest//Grandpa",
        hex32("92bfddf22dba12139196d6d81859baeb504f3c6509507c5aadaee860d8995f16"),
    ),
    (
        "ed25519",
        "//OCWTest//Grandpa",
        hex32("25c5d727053a57a85fdd771f8d6775f61c2664978d8e327a44c1811d6d708df8"),
    ),
    (
        "ecdsa-account",
        "//OCWTest//Grandpa",
        hex32("70b1f62985dd219cf25580764899bb4ee6a699e40022406c57fc069b059b342d"),
    ),
    (
        "sr25519",
        "//OCWTest//Liveness",
        hex32("6ed2d2d14583fa47c6410a561be9b82dcca2d8c7e30575fffd9f9469e0016342"),
    ),
    (
        "ed25519",
        "//OCWTest//Liveness",
        hex32("77dd614544740d311ec00132982b0bf35dfb7ba8bc5c9a13fe369b2a7fd46672"),
    ),
    (
        "ecdsa-account",
        "//OCWTest//Liveness",
        hex32("28d40659a9ac4d604aae04bf1f436f363c3863d80be59eb2b0cd68dab5db25bb"),
    ),
    (
        "sr25519",
        "//OCWTest//Discovery",
        hex32("549bf07e831091c04aa472d29dd598493b809ae1e7eccdd83120222afdadd45c"),
    ),
    (
        "ed25519",
        "//OCWTest//Discovery",
        hex32("692e9dbcf19c9b6c001c2879e2155e160d2aacf95aee69cbd3d3add4786b3b49"),
    ),
    (
        "ecdsa-account",
        "//OCWTest//Discovery",
        hex32("c8ff6abf288c1b1eec861dd6732b2929435a21914caa4c4e912705398eb51b3e"),
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

// VERIFIED: forgeable ("weak") public keys (round-3 review/audit CR3-01 = CS3-1).
// Each verifies a constant, message-independent signature under the verifiers sp-core
// 43.0.0 and sp-io use, so anyone can sign for such an account. The unit test
// `weak_keys_are_universally_forgeable_without_the_check` proves each forgery with
// `sp_core::Pair::verify` and that all 14 ed25519 encodings found by the review are
// covered.
// - sr25519: only `[0; 32]`, the Ristretto identity. schnorrkel 0.11.5 decodes public
//   keys via `RistrettoBoth::from_compressed` (`src/points.rs:75-77`, reached from
//   `PublicKey::from_bytes`, `src/keys.rs:646-697`), i.e. curve25519-dalek 4.1.3
//   `CompressedRistretto::decompress` (`src/ristretto.rs:255-266`), which refuses
//   non-canonical and negative encodings. Ristretto has prime order, so the identity
//   is the only small-order point and `[0; 32]` its only encoding.
// - ed25519: every encoding that curve25519-dalek 4.1.3 `CompressedEdwardsY::decompress`
//   accepts (`src/edwards.rs:194-203`; `FieldElement::from_bytes` reduces y mod p, so
//   non-canonical y and sign bits decode, as ZIP215 requires) and whose point
//   `is_small_order` (`src/edwards.rs:1227-1229`, [8]P == identity): the 8
//   `EIGHT_TORSION` points and their non-canonical encodings.
// CHOICE: refused in every account position whatever its declared scheme, because an
// AccountId32 can be signed for with either curve through MultiSignature.
const ALL_ZERO: [u8; 32] = [0; 32];

/// Names why `key` is universally forgeable, if it is: the all-zero sr25519 Ristretto
/// identity, or any encoding of an ed25519 small-order point.
pub fn weak_public_key(key: &[u8; 32]) -> Option<&'static str> {
    use curve25519_dalek::edwards::CompressedEdwardsY;
    if *key == ALL_ZERO {
        return Some("all-zero: the sr25519 Ristretto identity and an ed25519 order-4 point");
    }
    CompressedEdwardsY(*key)
        .decompress()
        .filter(|point| point.is_small_order())
        .map(|_| "an ed25519 small-order point")
}

/// Refuses forgeable keys regardless of scheme or position: custody signatories and
/// role addresses, validator accounts and session keys.
pub(super) fn not_weak(key: &[u8; 32], path: String) -> Check {
    match weak_public_key(key) {
        Some(name) => Err(fail(
            "custody_pop_weak_key",
            path,
            format!("forgeable public key ({name}): anyone can produce a valid signature for it"),
        )),
        None => Ok(()),
    }
}

/// Decision A (yvan 2026-09-14 05:46 UTC): an ed25519 key must be a canonical encoding
/// of a non-identity point in the prime-order subgroup. A torsioned key `A + T` verifies
/// signatures made with `A`'s secret under ZIP215 while being a different AccountId32,
/// so one secret could stand as several signatories or bypass sudo independence.
// VERIFIED: curve25519-dalek 4.1.3 `CompressedEdwardsY::decompress`
// (`src/edwards.rs:194-203`), `EdwardsPoint::compress` (`:566`),
// `is_torsion_free` = [ℓ]P == identity (`:1257-1259`), `IsIdentity`
// (`src/traits.rs:35-47`).
// CHOICE: a precise sibling code `ed25519_key_not_prime_order` rather than
// `custody_pop_weak_key`: a torsioned or non-canonical key is not universally
// forgeable, it lets its holder claim extra identities.
pub(super) fn not_ed25519_prime_order(key: &[u8; 32], path: String) -> Check {
    use curve25519_dalek::edwards::CompressedEdwardsY;
    use curve25519_dalek::traits::IsIdentity;
    let refuse = |why: &str| {
        Err(fail(
            "ed25519_key_not_prime_order",
            path.clone(),
            format!("ed25519 key {why}; it must be a canonical, torsion-free, non-identity point"),
        ))
    };
    let Some(point) = CompressedEdwardsY(*key).decompress() else {
        return refuse("is not a curve point");
    };
    if point.compress().to_bytes() != *key {
        return refuse("is a non-canonical encoding");
    }
    if !point.is_torsion_free() {
        return refuse("has a torsion component");
    }
    if point.is_identity() {
        return refuse("is the identity");
    }
    Ok(())
}

pub(super) fn bytes(address: &Address) -> [u8; 32] {
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

/// CS-2/CS-4 reserved identity: ammAccount, miningPoolAccount or any PalletId account.
pub(super) fn not_pallet_account(key: &[u8; 32], bootstrap: &Bootstrap, path: String) -> Check {
    let reserved = [
        bytes(&bootstrap.amm_account),
        bytes(&bootstrap.mining_pool_account),
    ];
    if reserved.contains(key) || key.starts_with(PALLET_ACCOUNT_PREFIX) {
        return Err(fail(
            "authority_pallet_account",
            path,
            "a keyless PalletId account (ammAccount, miningPoolAccount or any b\"modl\" account) cannot be an authority, signatory or validator account",
        ));
    }
    Ok(())
}

/// Every validator's four session-key bytes, mapped to the first slot that uses them.
pub(super) fn session_keys(bootstrap: &Bootstrap) -> BTreeMap<[u8; 32], String> {
    let mut keys = BTreeMap::new();
    for (index, v) in bootstrap.validators.iter().enumerate() {
        for (role, key) in session_key_slots(v) {
            keys.entry(hex32(key.as_str()))
                .or_insert_with(|| format!("/bootstrap/validators/{index}/{role}"));
        }
    }
    keys
}

/// The node runtime's SessionKeys at d9-v2-node `1320fe8` (`runtime/src/lib.rs:73-80`):
/// babe, grandpa, liveness and authority_discovery (`discovery`). imOnline was removed
/// in revision 6 (yvan 2026-09-14).
pub(super) fn session_key_slots(v: &Validator) -> [(&'static str, &Digest); 4] {
    [
        ("babe", &v.babe),
        ("grandpa", &v.grandpa),
        ("discovery", &v.discovery),
        ("liveness", &v.liveness),
    ]
}

pub(super) fn not_session_key(
    key: &[u8; 32],
    session_keys: &BTreeMap<[u8; 32], String>,
    path: String,
) -> Check {
    if let Some(slot) = session_keys.get(key) {
        let mut error = fail(
            "authority_session_key",
            path,
            "a validator session key cannot be an authority, signatory or validator account",
        );
        error.related_path = Some(slot.clone());
        return Err(error);
    }
    Ok(())
}

/// Accounts in this document that no multisig may be or contain as a signatory.
struct Reserved<'a> {
    bootstrap: &'a Bootstrap,
    roles: Vec<(String, [u8; 32])>,
    validators: BTreeSet<[u8; 32]>,
    session_keys: BTreeMap<[u8; 32], String>,
}

impl Reserved<'_> {
    fn identity(&self, key: &[u8; 32], path: String) -> Check {
        not_pallet_account(key, self.bootstrap, path.clone())?;
        if self.validators.contains(key) {
            return Err(fail(
                "authority_validator_account",
                path,
                "a validator account cannot be or sign for a multisig authority",
            ));
        }
        not_session_key(key, &self.session_keys, path)
    }
}

/// DEC-21 k-of-n structure, mirroring d9-v2-tools `derive_admins::derive_role`,
/// plus the binding address == multisig_account(signatories, threshold).
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
        let key = bytes(&signatory.address);
        let signatory_path = format!("{path}/signatories/{index}");
        if previous.is_some_and(|previous| previous >= key) {
            return Err(fail(
                "multisig_signatory_order",
                signatory_path,
                "signatories must be unique and strictly ascending by AccountId32 bytes",
            ));
        }
        // CR3-01: before any identity or signature check, so a forgeable key never
        // reaches verification.
        not_weak(&key, signatory_path.clone())?;
        if let PossessionEvidence::Signature(SignatureEvidence {
            scheme: SignatureScheme::Ed25519,
            ..
        }) = &signatory.evidence
        {
            not_ed25519_prime_order(&key, signatory_path.clone())?;
        }
        if key == address {
            return Err(fail(
                "multisig_self_signatory",
                signatory_path,
                "a multisig account cannot be its own signatory",
            ));
        }
        not_development(&key, signatory_path.clone())?;
        // A signatory that is a multisig defined outside this document cannot be
        // seen here; its proof-of-possession cannot be produced, so PoP refuses it.
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
    not_weak(&address, address_path.clone())?;
    not_development(&address, address_path.clone())?;
    reserved.identity(&address, address_path.clone())?;
    let signatories: Vec<[u8; 32]> = authority
        .signatories
        .iter()
        .map(|signatory| bytes(&signatory.address))
        .collect();
    if multisig_account(&signatories, authority.threshold) != address {
        return Err(fail(
            "multisig_address_not_derived",
            address_path,
            "address must equal the pallet_multisig account of its signatories and threshold",
        ));
    }
    Ok(())
}

/// One custody role: JSON path, PoP role and the authority.
pub(super) struct Role<'a> {
    pub(super) path: String,
    pub(super) role: CustodyRole,
    pub(super) authority: &'a MultisigAuthority,
}

pub(super) fn roles(b: &Bootstrap) -> Vec<Role<'_>> {
    let mut roles = vec![
        Role {
            path: "/bootstrap/sudo".to_owned(),
            role: CustodyRole::Sudo,
            authority: &b.sudo,
        },
        Role {
            path: "/bootstrap/usdtOwner".to_owned(),
            role: CustodyRole::UsdtOwner,
            authority: &b.usdt_owner,
        },
    ];
    for (index, admin) in b.admins.iter().enumerate() {
        roles.push(Role {
            path: format!("/bootstrap/admins/{index}/multisig"),
            role: CustodyRole::Admin(admin.pallet),
            authority: &admin.multisig,
        });
    }
    roles
}

pub(super) fn check(i: &ContractInput) -> Check {
    let b = &i.bootstrap;
    let roles = roles(b);
    let reserved = Reserved {
        bootstrap: b,
        roles: roles
            .iter()
            .map(|role| (role.path.clone(), bytes(&role.authority.address)))
            .collect(),
        validators: b.validators.iter().map(|v| bytes(&v.account)).collect(),
        session_keys: session_keys(b),
    };
    for role in &roles {
        multisig(role.authority, &role.path, &reserved)?;
    }
    // CS-4 (yvan 2026-09-14): sudo is distinct from the USDT owner and from every
    // pallet admin. The 12 admins may share one multisig and the USDT owner may equal
    // an admin multisig.
    // CHOICE: usdtOwner == admin stays allowed; the ruling restricts only sudo, and
    // refusing it would add policy nobody approved.
    let sudo = &reserved.roles[0].1;
    if let Some((role, _)) = reserved.roles[1..]
        .iter()
        .find(|(_, address)| address == sudo)
    {
        let mut error = fail(
            "authority_role_not_distinct",
            "/bootstrap/sudo/address",
            "sudo must be a different account from the USDT owner and every pallet admin",
        );
        error.related_path = Some(format!("{role}/address"));
        return Err(error);
    }
    // Ruling 2 (yvan 2026-09-14 04:34 UTC, supersedes the 03:33 allowance): sudo and
    // the admin side (usdtOwner and every pallet admin) share no signatory. A
    // signatory may still sit in several admin-side multisigs. Disjointness subsumes
    // the quorum-containment predicate (CS2-3): no admin-side key set can reach any
    // part of the sudo quorum.
    let mut admin_side = BTreeMap::new();
    for role in &roles[1..] {
        for (index, signatory) in role.authority.signatories.iter().enumerate() {
            admin_side
                .entry(bytes(&signatory.address))
                .or_insert_with(|| format!("{}/signatories/{index}", role.path));
        }
    }
    for (index, signatory) in b.sudo.signatories.iter().enumerate() {
        if let Some(other) = admin_side.get(&bytes(&signatory.address)) {
            let mut error = fail(
                "sudo_signatory_not_independent",
                format!("/bootstrap/sudo/signatories/{index}"),
                "sudo signatories must be disjoint from the USDT owner and every pallet admin",
            );
            error.related_path = Some(other.clone());
            return Err(error);
        }
    }
    rehome_accounts(i, &roles)?;
    for role in &roles {
        pop::check(i, role.authority, &role.path, role.role)?;
    }
    Ok(())
}

/// Ruling 1 (yvan 2026-09-14 04:34 UTC). D9 rehomes may credit only the mining pool
/// (it pays merchant redemptions, node rewards and burn withdrawals) or the AMM;
/// asset rehomes only the AMM (the mining pool holds no assets). Any other
/// destination requires a new contract RC. topUps, reserveRefunds and rewardCredits
/// are exact-derived from source rows elsewhere, so rehome `to` is the only
/// free-form value destination.
///
/// CS3-5: a rehome may not drain a custody account, a validator identity or the
/// mining pool/AMM themselves.
// CHOICE: one code per direction. Destinations always report
// `rehome_destination_not_allowed` (CR3-04) with the matched identity kind in the
// message and its path in `related_path`; sources report the separate
// `rehome_source_not_allowed`, because the pool and AMM are valid destinations but
// never valid sources.
fn rehome_accounts(i: &ContractInput, roles: &[Role<'_>]) -> Check {
    let b = &i.bootstrap;
    // Both accounts are already required to equal their runtime PalletId derivation
    // in composition.rs, so comparing against them does not retype the derivation.
    let pool = bytes(&b.mining_pool_account);
    let amm = bytes(&b.amm_account);
    let mut identities: BTreeMap<[u8; 32], (&'static str, String)> = BTreeMap::new();
    let mut note = |key: [u8; 32], kind: &'static str, path: String| {
        identities.entry(key).or_insert((kind, path));
    };
    for role in roles {
        note(
            bytes(&role.authority.address),
            "custody multisig",
            format!("{}/address", role.path),
        );
        for (index, signatory) in role.authority.signatories.iter().enumerate() {
            note(
                bytes(&signatory.address),
                "custody signatory",
                format!("{}/signatories/{index}/address", role.path),
            );
        }
    }
    for (index, v) in b.validators.iter().enumerate() {
        note(
            bytes(&v.account),
            "validator account",
            format!("/bootstrap/validators/{index}/account"),
        );
    }
    for (key, slot) in session_keys(b) {
        note(key, "validator session key", slot);
    }
    note(
        pool,
        "mining pool account",
        "/bootstrap/miningPoolAccount".to_owned(),
    );
    note(amm, "AMM account", "/bootstrap/ammAccount".to_owned());

    let destination = |to: &Address, path: String, allowed: &[[u8; 32]], names: &str| -> Check {
        let key = bytes(to);
        let refuse = |kind: String, related: Option<String>| {
            let mut error = fail(
                "rehome_destination_not_allowed",
                path.clone(),
                format!("rehome destination is {kind}; it must be {names}, and any other destination requires a new contract RC"),
            );
            error.related_path = related;
            Err(error)
        };
        if let Some(name) = well_known_development_key(&key) {
            return refuse(format!("a development key ({name})"), None);
        }
        if let Some(name) = weak_public_key(&key) {
            return refuse(format!("a forgeable key ({name})"), None);
        }
        // Identities are noted custody, validators, session keys, then pool and AMM, so a
        // pool/AMM key that is also any other identity reports that identity.
        let pallet_target = |kind: &str| kind == "mining pool account" || kind == "AMM account";
        match identities.get(&key) {
            Some((kind, _)) if pallet_target(kind) && allowed.contains(&key) => Ok(()),
            Some((kind, related)) => refuse(format!("a {kind}"), Some(related.clone())),
            None if allowed.contains(&key) => Ok(()),
            None => refuse("another account".to_owned(), None),
        }
    };
    let source = |from: &Address, path: String| -> Check {
        if let Some((kind, related)) = identities.get(&bytes(from)) {
            let mut error = fail(
                "rehome_source_not_allowed",
                path,
                format!("a rehome cannot move funds out of a {kind}"),
            );
            error.related_path = Some(related.clone());
            return Err(error);
        }
        Ok(())
    };
    for (index, row) in i.changes.rehomes.iter().enumerate() {
        source(&row.from, format!("/changes/rehomes/{index}/from"))?;
        destination(
            &row.to,
            format!("/changes/rehomes/{index}/to"),
            &[pool, amm],
            "bootstrap.miningPoolAccount or bootstrap.ammAccount",
        )?;
    }
    for (index, row) in i.changes.asset_rehomes.iter().enumerate() {
        source(&row.from, format!("/changes/assetRehomes/{index}/from"))?;
        destination(
            &row.to,
            format!("/changes/assetRehomes/{index}/to"),
            &[amm],
            "bootstrap.ammAccount",
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::{crypto::DEV_PHRASE, ecdsa, ed25519, sr25519, Pair};
    use sp_runtime::traits::IdentifyAccount;

    fn hex(value: &str) -> [u8; 32] {
        hex32(value.strip_prefix("0x").unwrap())
    }

    // Golden vectors from an independent implementation: `@polkadot/util-crypto` v13
    // `createKeyMulti` + `encodeAddress(_, 9)`, copied from d9-v2-tools `a5938f7`
    // `network-bootstrap/d9-bootstrap/tests/derive_admins_test.rs` (generated
    // 2026-09-13). The same vectors pin the tools copy of this derivation.
    const ALICE: &str = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
    const BOB: &str = "0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48";
    const CHARLIE: &str = "0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22";
    const DAVE: &str = "0x306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20";
    const EVE: &str = "0xe659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e";

    fn ss58(account: [u8; 32]) -> String {
        Address::from_account_id(account.into()).as_str().to_owned()
    }

    #[test]
    fn multisig_account_matches_polkadot_js_golden_vectors() {
        let abc = multisig_account(&[hex(ALICE), hex(BOB), hex(CHARLIE)], 2);
        assert_eq!(
            abc,
            hex("0x49daa32c7287890f38b7e1a8cd2961723d36d20baa0bf3b82e0c4bdda93b1c0a")
        );
        assert_eq!(ss58(abc), "vkmmi2TwxAsTuBHdhVKzmM3ys8oHrUtk759Jnc5uamGY2hD");
        let abcde = multisig_account(
            &[hex(ALICE), hex(BOB), hex(CHARLIE), hex(DAVE), hex(EVE)],
            3,
        );
        assert_eq!(
            abcde,
            hex("0x36e11b1f4873b27df0a93a0670ea03e56aa7d677a42f67309c33a0d4d9b5bdae")
        );
        assert_eq!(
            ss58(abcde),
            "vKtnPZEwUHuKtTgevmzrnCM392AnFE1TBY3fDvDJUZju9Qm"
        );
        assert_eq!(
            ss58(multisig_account(&[hex(ALICE), hex(BOB)], 2)),
            "x4dxrZoSyCbJbzQXbBLuJqyQpn6ZvhsoHiotykmZFj7QXXY"
        );
        // polkadot-js gave the same address for [B, A, C] as for [A, B, C].
        assert_eq!(
            multisig_account(&[hex(CHARLIE), hex(BOB), hex(ALICE)], 2),
            abc
        );
        assert_ne!(
            multisig_account(&[hex(ALICE), hex(BOB), hex(CHARLIE)], 3),
            abc
        );
    }

    /// The 14 ed25519 small-order encodings reported by the round-3 review/audit
    /// (curve25519-dalek 4.1.3 `EIGHT_TORSION`, compressed, plus the 6 low-order
    /// non-canonical encodings in ed25519-zebra 4.2.0 `tests/small_order.rs:21-24`).
    const REVIEWED_ED25519_SMALL_ORDER: [&str; 14] = [
        "0100000000000000000000000000000000000000000000000000000000000000",
        "c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac037a",
        "0000000000000000000000000000000000000000000000000000000000000080",
        "26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc05",
        "ecffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f",
        "26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc85",
        "0000000000000000000000000000000000000000000000000000000000000000",
        "c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac03fa",
        "0100000000000000000000000000000000000000000000000000000000000080",
        "ecffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        "edffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f",
        "edffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        "eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f",
        "eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    ];

    #[test]
    fn weak_keys_are_universally_forgeable_without_the_check() {
        use curve25519_dalek::constants::EIGHT_TORSION;
        let messages: [&[u8]; 2] = [b"any message", b"a different custody PoP message"];
        // sr25519 identity: R = compressed Ristretto basepoint, s = 1 (marker bit set).
        let mut sr_forgery = [0u8; 64];
        sr_forgery[..32].copy_from_slice(&hex32(
            "e2f2ae0a6abc4e71a884a961c500515f58e30b6aa582dd8db6a65945e08d2d76",
        ));
        sr_forgery[32] = 1;
        sr_forgery[63] |= 0x80;
        for message in messages {
            assert!(sr25519::Pair::verify(
                &sr25519::Signature::from_raw(sr_forgery),
                message,
                &sr25519::Public::from_raw(ALL_ZERO),
            ));
        }
        assert!(weak_public_key(&ALL_ZERO).is_some());
        // ed25519 small-order keys: R = identity, s = 0 verifies for any message.
        let mut ed_forgery = [0u8; 64];
        ed_forgery[0] = 1;
        for encoding in REVIEWED_ED25519_SMALL_ORDER {
            let key = hex32(encoding);
            for message in messages {
                assert!(
                    ed25519::Pair::verify(
                        &ed25519::Signature::from_raw(ed_forgery),
                        message,
                        &ed25519::Public::from_raw(key),
                    ),
                    "forgery does not verify for {encoding}"
                );
            }
            assert!(weak_public_key(&key).is_some(), "{encoding} not refused");
        }
        for point in EIGHT_TORSION {
            assert!(weak_public_key(&point.compress().to_bytes()).is_some());
        }
        // Real keys are not weak.
        for _ in 0..8 {
            assert!(weak_public_key(&sr25519::Pair::generate().0.public().0).is_none());
            let ed = ed25519::Pair::generate().0.public().0;
            assert!(weak_public_key(&ed).is_none());
            assert!(not_ed25519_prime_order(&ed, String::new()).is_ok());
        }
    }

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
                61,
                "{scheme}: 14 sp-keyring names, DEV_PHRASE, 6 LocalValidator, 36 Mainnet, 4 OCWTest"
            );
        }
        // d9-v2-node e13a19d `runtime/src/genesis_config_presets.rs:144-163`.
        const LOCAL_DEV_SR25519_PUBS: [&str; 6] = [
            "8e5052a337be646ae18ef81eee5d820403671cbf79090ae80641c40a47698d3a",
            "3c225fc7fe1b1fea0f8109f20abaa48346c785b83311b93d66d23eabbec64c33",
            "ae48fa9d973ec2cea3d48da7657d3061451e54c9d44fcfda3b19204fa2487040",
            "206f7539bde2fa9182654152271d84c0ebe78bb7c5071ab35162998230773464",
            "588d0a678cbfa52557ba35143cabc8992bba88c7e6c2ba722b47e224a99d3925",
            "60e107d1d885a2cdbee8fdc4c8e9b5560566b4f639eacc54ebb577d9515c6c7b",
        ];
        const LOCAL_DEV_ED25519_PUBS: [&str; 6] = [
            "43d1b4c81c4dd6ba8e7d3ad8b883818858a5e2671a069d3e58ce4d36bce6725b",
            "7dc507114268649fa8881d45573e6d2925890dc5f081d324b54400a22cc08415",
            "e8359425fbd47f877ce6718009ce46ccba1953d5bc378085d05ea427e5888791",
            "0c9148a02698491cf4751fb99accc5f3f336b63be458cdc9ce6fb49af3deed4e",
            "fb36512092c124f2982792e1f5a7d468de521d6fda4d0ecd4497db5181ca9832",
            "3f62dca0a1fa14648de3aae69b49aba26ccd08d5a3ec5ceb575781f011dc98f2",
        ];
        for i in 0..6 {
            let uri = format!("//LocalValidator{}", i + 1);
            let entry = |scheme: &str| {
                DEVELOPMENT_KEYS
                    .iter()
                    .find(|(s, u, _)| *s == scheme && *u == uri)
                    .unwrap()
                    .2
            };
            assert_eq!(entry("sr25519"), hex32(LOCAL_DEV_SR25519_PUBS[i]), "{uri}");
            assert_eq!(entry("ed25519"), hex32(LOCAL_DEV_ED25519_PUBS[i]), "{uri}");
        }
    }
}
