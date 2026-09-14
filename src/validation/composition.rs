use super::custody::{hex32, not_development};
use super::*;
use sp_runtime::traits::AccountIdConversion;
use std::collections::BTreeSet;

fn pallet_account(pallet: &[u8; 8]) -> Address {
    Address::from_account_id(frame_support::PalletId(*pallet).into_account_truncating())
}
pub(super) fn check(i: &ContractInput) -> Check {
    let b = &i.bootstrap;
    same(
        pallet_account(b"d9/d9amm"),
        b.amm_account.clone(),
        "/bootstrap/ammAccount",
    )?;
    same(
        pallet_account(b"d9/mnpol"),
        b.mining_pool_account.clone(),
        "/bootstrap/miningPoolAccount",
    )?;
    if b.redemption_price_floor {
        return Err(fail(
            "retired_price_floor",
            "/bootstrap/redemptionPriceFloor",
            "DEC-17: no price floor at launch",
        ));
    }
    if b.fee_bps != 30 || b.liquidity_tolerance_bps != 500 {
        return Err(fail(
            "amm_policy",
            "/bootstrap",
            "recorded launch policy: 30 bps fee, 500 bps post-migration liquidity tolerance",
        ));
    }
    if b.d9_reserve_floor.0 != 600_000_000_000_000_000 || b.usdt_reserve_floor.0 != 600_000_000_000
    {
        return Err(fail(
            "reserve_floor_units",
            "/bootstrap",
            "D9-190: 600k D9 = 600000000000000000 raw; 600k USDT = 600000000000 raw; LP minimum is separate",
        ));
    }
    if b.validators.is_empty() {
        return Err(fail(
            "empty_authorities",
            "/bootstrap/validators",
            "authority set cannot be empty",
        ));
    }
    keyed(
        &b.validators,
        |r| r.account.clone(),
        "/bootstrap/validators",
    )?;
    let mut keys = BTreeSet::new();
    for (index, v) in b.validators.iter().enumerate() {
        not_development(
            v.account.account_id().as_ref(),
            format!("/bootstrap/validators/{index}/account"),
        )?;
        for (role, key) in [
            ("babe", &v.babe),
            ("grandpa", &v.grandpa),
            ("imOnline", &v.im_online),
            ("discovery", &v.discovery),
            ("liveness", &v.liveness),
        ] {
            if key.0.bytes().all(|c| c == b'0') || !keys.insert((role, &key.0)) {
                return Err(fail(
                    "invalid_session_key",
                    format!("/bootstrap/validators/{index}/{role}"),
                    "zero/duplicate key in role",
                ));
            }
            // Digest decoding guarantees 64 lowercase hex characters, so hex32
            // cannot panic. babe/imOnline/discovery/liveness are sr25519 and
            // grandpa is ed25519; every scheme's deny list is checked for each.
            not_development(
                &hex32(key.as_str()),
                format!("/bootstrap/validators/{index}/{role}"),
            )?;
        }
    }
    keyed(&b.admins, |r| r.pallet.clone(), "/bootstrap/admins")?;
    let required = [
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
    let actual: BTreeSet<_> = b.admins.iter().map(|r| r.pallet.as_str()).collect();
    if actual != required.into_iter().collect() {
        return Err(fail(
            "admin_coverage",
            "/bootstrap/admins",
            "explicit admin required for each admin-bearing D9 pallet",
        ));
    }
    super::custody::check(i)?;
    // R-4: the declared set is the complete V2 pallet-assets ID set. The contract
    // does not model definitions/metadata (owner, sufficiency, minimum balance,
    // name, symbol); the producer requires both of those ID sets to equal this.
    if b.asset_ids.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(fail(
            "asset_id_set",
            "/bootstrap/assetIds",
            "declared asset IDs must be unique and strictly ascending",
        ));
    }
    if let Some(book) = i
        .state
        .assets
        .iter()
        .find(|book| b.asset_ids.binary_search(&book.asset_id).is_err())
    {
        return Err(fail(
            "asset_id_set",
            "/bootstrap/assetIds",
            format!("migrated asset {} is not declared", book.asset_id),
        ));
    }
    Ok(())
}
