use crate::{Address, Amount, Commit, Count, Digest, Millis};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

macro_rules! record {
    ($name:ident { $($(#[$meta:meta])* $field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct $name { $($(#[$meta])* pub $field: $ty),* }
    }
}

record!(ContractInput {
    contract_version: String,
    purpose: Purpose,
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
    sudo: Address, validators: Vec<Validator>, admins: Vec<AdminRole>,
    /// These accounts are derived from runtime PalletIds, never chosen by a producer.
    amm_account: Address, mining_pool_account: Address,
    /// Fresh balances require a declared funding transfer in changes, no implicit mint.
    fee_bps: u32, liquidity_tolerance_bps: u32,
    d9_reserve_floor: Amount, usdt_reserve_floor: Amount,
    redemption_price_floor: bool,
});
record!(AdminRole {
    pallet: String,
    account: Address
});
record!(Validator {
    account: Address,
    babe: Digest,
    grandpa: Digest,
    im_online: Digest,
    discovery: Digest,
    liveness: Digest,
});

fn required_nullable<'de, D: serde::Deserializer<'de>, T: serde::Deserialize<'de>>(
    de: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(de)
}
