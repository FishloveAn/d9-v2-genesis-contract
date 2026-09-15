use super::*;

pub(super) fn check(i: &ContractInput) -> Check {
    let s = &i.source;
    let burn = keyed(
        &i.state.burn_accounts,
        |r| r.account.clone(),
        "/state/burnAccounts",
    )?;
    let source = keyed(
        &s.burn_accounts,
        |r| r.account.clone(),
        "/source/burnAccounts",
    )?;
    if burn.len() != source.len() {
        return Err(fail(
            "record_count_mismatch",
            "/state/burnAccounts",
            "burn account set differs",
        ));
    }
    for (id, src) in source {
        let dst = burn
            .get(&id)
            .ok_or_else(|| fail("missing_account", "/state/burnAccounts", id.as_str()))?;
        let path = format!("/state/burnAccounts/{}", id.as_str());
        let gross = scale(src.amount_burned.0, 3, &path)?;
        same(
            gross,
            add(src.remaining_due.0, src.balance_paid.0, &path)?,
            &path,
        )?;
        let expected = BurnAccount {
            account: id,
            amount_burned: src.amount_burned,
            balance_due: Amount(gross),
            balance_paid: src.balance_paid,
            last_burn: src.last_burn,
            last_withdrawal: src.last_withdrawal,
            last_interaction: src.last_interaction,
            referral_boost_coefficients: src.referral_boost_coefficients,
        };
        if dst.balance_paid != expected.balance_paid {
            return Err(fail(
                "burn_paid_history",
                &path,
                "paid history must equal the authoritative main-pool source",
            ));
        }
        same(&expected, *dst, &path)?;
        timestamp(dst.last_burn, s.timestamp_ms, &path)?;
        timestamp(dst.last_interaction, s.timestamp_ms, &path)?;
        if let Some(t) = dst.last_withdrawal {
            timestamp(t, s.timestamp_ms, &path)?;
        }
    }
    let merchants = keyed(
        &i.state.merchant_accounts,
        |r| r.account.clone(),
        "/state/merchantAccounts",
    )?;
    let source = keyed(
        &s.merchant_accounts,
        |r| r.account.clone(),
        "/source/merchantAccounts",
    )?;
    if merchants.len() != source.len() {
        return Err(fail(
            "record_count_mismatch",
            "/state/merchantAccounts",
            "merchant account set differs",
        ));
    }
    for (id, src) in source {
        let dst = merchants
            .get(&id)
            .ok_or_else(|| fail("missing_account", "/state/merchantAccounts", id.as_str()))?;
        let path = format!("/state/merchantAccounts/{}", id.as_str());
        timestamp(src.created_at, s.timestamp_ms, &path)?;
        if let Some(t) = src.last_conversion {
            timestamp(t, s.timestamp_ms, &path)?;
            if t < src.created_at {
                return Err(fail(
                    "merchant_timestamp_order",
                    format!("{path}/lastConversion"),
                    "lastConversion predates createdAt",
                ));
            }
        }
        let expected = MerchantAccount {
            account: id,
            merit: Amount(scale(src.green_points.0, 10_000, &path)?),
            relationship_factors: [
                Amount(scale(src.relationship_factors[0].0, 10_000, &path)?),
                Amount(scale(src.relationship_factors[1].0, 10_000, &path)?),
            ],
            last_conversion: src.last_conversion,
            redeemed_d9: src.redeemed_d9,
            created_at: src.created_at,
        };
        same(&expected, *dst, &path)?;
    }
    let exclusions = keyed(
        &i.changes.excluded_merchant_expiries,
        |r| r.account.clone(),
        "/changes/excludedMerchantExpiries",
    )?;
    let mut expected = keyed(
        &s.merchant_expiries,
        |r| r.account.clone(),
        "/source/merchantExpiries",
    )?;
    for (id, row) in exclusions {
        if merchants.contains_key(&id) || expected.get(&id) != Some(&row) {
            return Err(fail(
                "invalid_expiry_exclusion",
                "/changes/excludedMerchantExpiries",
                "must identify the exact source expiry of an absent account",
            ));
        }
        expected.remove(&id);
    }
    for id in expected.keys() {
        if !merchants.contains_key(id) {
            return Err(fail(
                "orphan_expiry",
                "/state/merchantExpiries",
                "orphan requires an explicit approved exclusion",
            ));
        }
    }
    // No age/expiry<created_at rejection: DEC-9 grandfathering is explicit.
    same_map(
        &expected,
        &keyed(
            &i.state.merchant_expiries,
            |r| r.account.clone(),
            "/state/merchantExpiries",
        )?,
        "/state/merchantExpiries",
    )?;
    let voting = keyed(
        &i.state.voting_interests,
        |r| r.account.clone(),
        "/state/votingInterests",
    )?;
    let source = keyed(
        &s.voting_interests,
        |r| r.account.clone(),
        "/source/votingInterests",
    )?;
    if voting.len() != source.len() {
        return Err(fail(
            "record_count_mismatch",
            "/state/votingInterests",
            "voting set differs",
        ));
    }
    for (id, src) in source {
        let dst = voting
            .get(&id)
            .ok_or_else(|| fail("missing_account", "/state/votingInterests", id.as_str()))?;
        if dst.delegated.0 != 0 {
            return Err(fail(
                "delegated_nonzero",
                "/state/votingInterests",
                "DEC-20 genesis requires delegated=0",
            ));
        }
        same(src.total, dst.total, "/state/votingInterests")?;
    }
    Ok(())
}

pub(super) fn check_locks(i: &ContractInput) -> Check {
    let s = &i.source;
    let locks = keyed(&s.locks, |r| r.account.clone(), "/source/locks")?;
    if !i.changes.excluded_judicial_locks.is_empty() {
        return Err(fail("invalid_lock_exclusion", "/changes/excludedJudicialLocks",
            "D9-195 2026-09-14: all source locks must be retained; Funded/RecordOnly is determined from final composition"));
    }
    same_map(
        &locks,
        &keyed(&i.state.locks, |r| r.account.clone(), "/state/locks")?,
        "/state/locks",
    )?;
    Ok(())
}
