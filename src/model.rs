use crate::{Address, Amount, CeremonyNonce, Commit, Count, Digest, Millis, Pcr0Hex, SignatureHex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

macro_rules! record {
    ($(#[$outer:meta])* $name:ident { $($(#[$meta:meta])* $field:ident : $ty:ty),* $(,)? }) => {
        $(#[$outer])*
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct $name { $($(#[$meta])* pub $field: $ty),* }
    }
}

record!(ContractInput {
    contract_version: String,
    purpose: Purpose,
    /// RC7 (S-6): the chain identity the producer's chain metadata must equal.
    chain: ChainIdentity,
    /// RC7 (D9-400 ruling 4): the proof-of-possession ceremony every custody
    /// signatory's signature binds to.
    custody: CustodyCeremony,
    source: SourceSnapshot,
    build: BuildIdentity,
    state: MigratedState,
    changes: Changes,
    bootstrap: Bootstrap,
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Purpose {
    SyntheticFixture,
    MigrationInput,
}

/// Network label shared with the d9-v2-tools bootstrap manifest `network`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Network {
    Mainnet,
    Testnet,
}

impl Network {
    /// The wire label, as serialized and as used in the custody PoP message.
    pub fn label(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
        }
    }
}

/// Mirrors `sc_chain_spec::ChainType`'s unit variants and their serde names.
/// `Custom(String)` is deliberately not representable.
// VERIFIED: ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sc-chain-spec-51.0.0/src/lib.rs
// (`pub enum ChainType { Development, Local, Live, Custom(String) }`, default serde derive),
// the version pinned by d9-v2-node nativegen/Cargo.lock at af1621b.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ChainType {
    Development,
    Local,
    Live,
}

record!(CustodyCeremony {
    /// 32 bytes chosen for this ceremony; every PoP message includes it.
    ceremony_nonce: CeremonyNonce,
});

/// Signature scheme of a custody signatory's proof-of-possession.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
// CHOICE: no ecdsa variant, so an unsupported scheme is unrepresentable and refused at
// decode. An ecdsa AccountId32 is blake2_256 of the public key, so no public key exists
// to verify a signature against.
pub enum SignatureScheme {
    Sr25519,
    Ed25519,
}

record!(ChainIdentity {
    network: Network,
    /// Chain-spec `id`.
    id: String,
    /// Chain-spec `name`.
    name: String,
    chain_type: ChainType,
});

record!(BuildIdentity {
    node_commit: Commit,
    pallets_commit: Commit,
    cargo_lock_digest: Digest,
    runtime_config_digest: Digest,
    wasm_digest: Digest,
    nativegen_commit: Commit,
});

record!(SourceSnapshot {
    genesis_hash: Digest, block_number: Count, block_hash: Digest,
    state_root: Digest, runtime_spec_version: u32,
    timestamp_ms: Millis, runtime_metadata_digest: Digest, finalized: bool,
    /// Dataset references must identify code/ABI as well as chain state.
    datasets: Vec<DatasetEvidence>,
    total_issuance: Amount,
    balances: Vec<BalanceSource>,
    assets: Vec<AssetBook>,
    referrals: Vec<Referral>,
    referral_counts: Vec<ReferralCount>,
    burn_accounts: Vec<BurnSource>,
    merchant_accounts: Vec<MerchantSource>,
    merchant_expiries: Vec<Expiry>,
    voting_interests: Vec<Voting>,
    locks: Vec<JudicialLock>,
    lp: LpBook,
    counters: Counters,
    legacy_reward_credits: Vec<Balance>,
});
record!(DatasetEvidence {
    dataset: String, source_map: String, block_hash: Digest,
    payload_digest: Digest, record_count: Count,
    /// Contract datasets require all three; pallet datasets require none.
    #[serde(deserialize_with = "required_nullable")] contract: Option<Address>, #[serde(deserialize_with = "required_nullable")] code_hash: Option<Digest>, #[serde(deserialize_with = "required_nullable")] abi_digest: Option<Digest>,
    supplements: Vec<SupplementEvidence>,
});
record!(SupplementEvidence {
    source_map: String, contract: Address, code_hash: Digest, abi_digest: Digest,
    block_hash: Digest, fields: Vec<String>,
});

record!(BalanceSource {
    account: Address,
    free: Amount,
    reserved: Amount
});
record!(Balance {
    account: Address,
    amount: Amount
});
record!(AssetBook {
    asset_id: u32, decimals: u8, total_supply: Amount, holders: Vec<Balance>,
    holder_count: Count,
});
record!(Referral {
    child: Address,
    parent: Address
});
record!(ReferralCount {
    parent: Address,
    count: Count
});
record!(Voting {
    account: Address,
    total: Count,
    delegated: Count
});
record!(Expiry {
    account: Address,
    expiry_ms: Millis
});
record!(JudicialLock {
    account: Address,
    frozen_by: Address,
    session: u32
});
record!(LpBook { positions: Vec<Balance>, total: Amount, d9_reserve: Amount, usdt_reserve: Amount });
record!(Counters {
    burn_volume: Amount,
    merchant_volume: Amount,
    accumulative_reward_pool: Amount,
    last_session: u32,
});

record!(BurnSource {
    account: Address, amount_burned: Amount, remaining_due: Amount, balance_paid: Amount,
    last_burn: Millis, #[serde(deserialize_with = "required_nullable")] last_withdrawal: Option<Millis>, last_interaction: Millis,
    referral_boost_coefficients: [Amount; 2],
});
record!(BurnAccount {
    account: Address, amount_burned: Amount, balance_due: Amount, balance_paid: Amount,
    last_burn: Millis, #[serde(deserialize_with = "required_nullable")] last_withdrawal: Option<Millis>, last_interaction: Millis,
    referral_boost_coefficients: [Amount; 2],
});
record!(MerchantSource {
    account: Address, green_points: Amount, relationship_factors: [Amount; 2],
    #[serde(deserialize_with = "required_nullable")] last_conversion: Option<Millis>, redeemed_d9: Amount, created_at: Millis,
});
record!(MerchantAccount {
    account: Address, merit: Amount, relationship_factors: [Amount; 2],
    #[serde(deserialize_with = "required_nullable")] last_conversion: Option<Millis>, redeemed_d9: Amount, created_at: Millis,
});

record!(MigratedState {
    balances: Vec<Balance>, assets: Vec<AssetBook>, referrals: Vec<Referral>,
    burn_accounts: Vec<BurnAccount>, merchant_accounts: Vec<MerchantAccount>,
    merchant_expiries: Vec<Expiry>, voting_interests: Vec<Voting>,
    locks: Vec<JudicialLock>, lp: LpBook, counters: Counters,
});

record!(Changes {
    top_ups: Vec<Balance>, reserve_refunds: Vec<Balance>,
    rehomes: Vec<Rehome>, reward_credits: Vec<Balance>,
    asset_rehomes: Vec<AssetRehome>,
    excluded_merchant_expiries: Vec<Expiry>,
    excluded_judicial_locks: Vec<JudicialLockExclusion>,
    /// Unknown/economic dispositions never become implicit zero/empty defaults.
    unresolved: Vec<PendingDecision>,
});
record!(Rehome {
    from: Address,
    to: Address,
    amount: Amount,
    decision: String
});
record!(AssetRehome {
    asset_id: u32,
    from: Address,
    to: Address,
    source_amount: Amount,
    decision: String
});
record!(PendingDecision {
    item: String,
    issue: String
});

record!(Bootstrap {
    /// DEC-21 k-of-n pallet_multisig sudo account.
    sudo: MultisigAuthority,
    /// DEC-21 k-of-n pallet_multisig owner of USDT asset 1.
    usdt_owner: MultisigAuthority,
    validators: Vec<Validator>, admins: Vec<AdminRole>,
    /// These accounts are derived from runtime PalletIds, never chosen by a producer.
    amm_account: Address, mining_pool_account: Address,
    /// Fresh balances require a declared funding transfer in changes, no implicit mint.
    fee_bps: u32, liquidity_tolerance_bps: u32,
    d9_reserve_floor: Amount, usdt_reserve_floor: Amount,
    redemption_price_floor: bool,
    /// Complete V2 pallet-assets ID set, strictly ascending. Includes fresh
    /// assets with no migrated book; the producer's definitions and metadata
    /// must carry exactly these IDs.
    asset_ids: Vec<u32>,
});
record!(AdminRole {
    pallet: AdminPallet,
    multisig: MultisigAuthority,
});

// CHOICE: a closed enum instead of free slugs, so an unknown admin pallet is
// unrepresentable (refused at decode) and the slug table exists once. Nothing pins
// RC7 yet, so the wire change is free.
macro_rules! admin_pallets {
    ($($variant:ident => $slug:literal),* $(,)?) => {
        /// The twelve admin-bearing D9 pallets, spelled as d9-v2-tools
        /// `derive_admins::PALLET_ADMINS` and the genesis admin roles.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
        pub enum AdminPallet { $(#[serde(rename = $slug)] $variant),* }
        impl AdminPallet {
            /// Every admin pallet, in slug order.
            pub const ALL: [AdminPallet; 12] = [$(AdminPallet::$variant),*];
            /// The wire slug, e.g. `"d9-amm"`.
            pub fn slug(self) -> &'static str {
                match self { $(AdminPallet::$variant => $slug),* }
            }
        }
    };
}
admin_pallets! {
    D9Amm => "d9-amm",
    D9BurnMining => "d9-burn-mining",
    D9CrossChain => "d9-cross-chain",
    D9Governance => "d9-governance",
    D9JudicialPenalty => "d9-judicial-penalty",
    D9Merchant => "d9-merchant",
    D9MiningPool => "d9-mining-pool",
    D9NodeRegistry => "d9-node-registry",
    D9NodeRewards => "d9-node-rewards",
    D9Referrals => "d9-referrals",
    D9UpgradeCoordinator => "d9-upgrade-coordinator",
    D9Voting => "d9-voting",
}

/// A custody role, as bound into the proof-of-possession message.
// CHOICE: typed role instead of a free label string, so a message can only be built
// for a role that exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CustodyRole {
    Sudo,
    UsdtOwner,
    Admin(AdminPallet),
}

impl CustodyRole {
    /// `"sudo"`, `"usdtOwner"` or `"admin/<pallet slug>"`.
    pub fn label(self) -> String {
        match self {
            Self::Sudo => "sudo".to_owned(),
            Self::UsdtOwner => "usdtOwner".to_owned(),
            Self::Admin(pallet) => format!("admin/{}", pallet.slug()),
        }
    }
}

impl Serialize for CustodyRole {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.label())
    }
}
record!(
    /// DEC-21 k-of-n pallet_multisig authority. Validation requires
    /// `address == multisig_account(signatories, threshold)`. Signature evidence is
    /// verified here; enclave-attested evidence is checked here only for structure and
    /// message binding, and its attestation is listed in the report's
    /// `pendingAttestationVerifications` for the producer. Neither proves the
    /// signatories are the intended custodians, which remains ceremony evidence.
    MultisigAuthority {
    address: Address,
    threshold: u16,
    /// Strictly ascending by AccountId32 bytes, as pallet_multisig requires.
    signatories: Vec<Signatory>,
});
record!(
    /// One custody signatory and its proof-of-possession over
    /// `custody_pop_message` for this role, multisig and ceremony.
    Signatory {
    /// The signatory's AccountId32, which is its sr25519 or ed25519 public key.
    address: Address,
    evidence: PossessionEvidence,
});

/// How a custody signatory proves possession of its key. Externally tagged:
/// `{"signature": {..}}` or `{"enclaveAttested": {..}}`, exactly one key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PossessionEvidence {
    /// A signature by the signatory key, verified by this contract.
    Signature(SignatureEvidence),
    /// A Nitro attestation from an enclave that holds the key. This contract checks
    /// structure and the message binding only; the producer must verify the
    /// document itself (see `independentEvidenceRequired`).
    EnclaveAttested(EnclaveAttestation),
}

// CHOICE: no message-form field. Decision D (yvan 2026-09-14 05:46 UTC) accepts raw
// proofs only, so the signature always covers `custody_pop_message` itself and a
// `message` field is an unknown field refused at decode.
record!(SignatureEvidence {
    scheme: SignatureScheme,
    /// 64-byte signature over the raw `custody_pop_message`, lowercase hex.
    signature: SignatureHex,
});

record!(EnclaveAttestation {
    /// Standard base64 (with padding) of the COSE_Sign1 attestation document bytes,
    /// at most 16 KiB decoded. Not verified by this contract.
    attestation_document: String,
    /// The signer enclave measurement the producer must require, 48-byte SHA-384 hex.
    expected_pcr0: Pcr0Hex,
    /// SHA-256 of `custody_pop_message` for this signatory, role and ceremony; the
    /// attestation's `user_data` must equal it.
    pop_message_sha256: Digest,
});
record!(Validator {
    account: Address,
    babe: Digest,
    grandpa: Digest,
    discovery: Digest,
    liveness: Digest,
});

fn required_nullable<'de, D: serde::Deserializer<'de>, T: serde::Deserialize<'de>>(
    de: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(de)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JudicialLockExclusionReason {
    MissingSystemAccount,
}
record!(JudicialLockExclusion {
    account: Address,
    reason: JudicialLockExclusionReason,
    v1_lock_amount: Amount,
    block_hash: Digest,
    /// Declared pinned RPC observation; independently authenticated by D9-378/173.
    system_account_exists: bool,
});
