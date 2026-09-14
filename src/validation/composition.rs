use super::custody::{
    bytes, hex32, not_development, not_ed25519_prime_order, not_pallet_account, not_session_key,
    not_weak, session_key_slots,
};
use super::*;
use sp_runtime::traits::AccountIdConversion;
use std::collections::BTreeSet;

/// CR4-02 (yvan 2026-09-14): an sr25519 session key (babe, liveness, discovery)
/// must be a canonical Ristretto encoding. No secret key corresponds to any other 32
/// bytes, so such a validator could never author, heartbeat or be discovered.
// VERIFIED: curve25519-dalek 4.1.3 `CompressedRistretto::decompress` returns `None` for
// non-canonical or negative encodings and for points off the curve (`src/ristretto.rs:255-270`).
// CHOICE: validator accounts are not checked: an AccountId32 may be an ed25519 key or an
// ecdsa blake2 hash, which are legitimately not Ristretto encodings.
// CHOICE: reuses invalid_session_key, which already names an unusable session key.
fn sr25519_session_key_decodes(key: &[u8; 32], path: String) -> Check {
    use curve25519_dalek::ristretto::CompressedRistretto;
    if CompressedRistretto(*key).decompress().is_none() {
        return Err(fail(
            "invalid_session_key",
            path,
            "sr25519 session key is not a canonical Ristretto point; no secret key can use it",
        ));
    }
    Ok(())
}

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
    // CHOICE: session-key bytes are unique across every slot of every validator,
    // not only within one role. A key reused in two slots of one validator, or in
    // any slot of two validators, is refused with the existing invalid_session_key.
    let mut keys = BTreeMap::<[u8; 32], String>::new();
    for (index, v) in b.validators.iter().enumerate() {
        let account = bytes(&v.account);
        let account_path = format!("/bootstrap/validators/{index}/account");
        not_weak(&account, account_path.clone())?;
        not_development(&account, account_path.clone())?;
        not_pallet_account(&account, b, account_path)?;
        for (role, key) in session_key_slots(v) {
            let path = format!("/bootstrap/validators/{index}/{role}");
            // Digest decoding guarantees 64 lowercase hex characters, so hex32
            // cannot panic. babe/liveness/discovery are sr25519 and
            // grandpa is ed25519; every scheme's deny list is checked for each.
            let key = hex32(key.as_str());
            if key == [0; 32] {
                return Err(fail("invalid_session_key", path, "zero session key"));
            }
            if let Some(first) = keys.insert(key, path.clone()) {
                let mut error = fail(
                    "invalid_session_key",
                    path,
                    "session key reused in another slot of this or another validator",
                );
                error.related_path = Some(first);
                return Err(error);
            }
            not_weak(&key, path.clone())?;
            if role == "grandpa" {
                not_ed25519_prime_order(&key, path.clone())?;
            }
            not_development(&key, path.clone())?;
            if role != "grandpa" {
                // CHOICE: after the deny list, so a well-known development key reports as
                // dev_key_authority even when it is not a valid sr25519 encoding.
                sr25519_session_key_decodes(&key, path)?;
            }
        }
    }
    for (index, v) in b.validators.iter().enumerate() {
        not_session_key(
            &bytes(&v.account),
            &keys,
            format!("/bootstrap/validators/{index}/account"),
        )?;
    }
    keyed(&b.admins, |r| r.pallet, "/bootstrap/admins")?;
    let actual: BTreeSet<AdminPallet> = b.admins.iter().map(|r| r.pallet).collect();
    if actual != AdminPallet::ALL.into_iter().collect() {
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
