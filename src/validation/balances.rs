use super::*;

pub(super) fn check(i: &ContractInput) -> Check {
    let s = &i.source;
    let mut expected = BTreeMap::new();
    let source = keyed(&s.balances, |r| r.account.clone(), "/source/balances")?;
    let refunds = balances_map(&i.changes.reserve_refunds, "/changes/reserveRefunds")?;
    let mut expected_refunds = BTreeMap::new();
    let mut issuance = 0;
    for (id, row) in &source {
        if row.reserved.0 != 0 {
            expected_refunds.insert(id.clone(), row.reserved.0);
        }
        let amount = add(row.free.0, row.reserved.0, "/source/balances")?;
        issuance = add(issuance, amount, "/source/totalIssuance")?;
        expected.insert(id.clone(), amount);
    }
    same(issuance, s.total_issuance.0, "/source/totalIssuance")?;
    same_map(&expected_refunds, &refunds, "/changes/reserveRefunds")?;
    let tops = balances_map(&i.changes.top_ups, "/changes/topUps")?;
    let mut expected_tops = BTreeMap::new();
    for (id, amount) in &mut expected {
        if *amount < 1_000_000 {
            let delta = 1_000_000 - *amount;
            expected_tops.insert(id.clone(), delta);
            *amount = 1_000_000;
            issuance = add(issuance, delta, "/changes/topUps")?;
        }
    }
    same_map(&expected_tops, &tops, "/changes/topUps")?;
    let rehomes = keyed(&i.changes.rehomes, |r| r.from.clone(), "/changes/rehomes")?;
    // Simultaneous debits before credits: chains/cycles of destinations do not
    // depend on input ordering, and a newly credited amount cannot fund a move.
    let mut credits = Vec::new();
    for (from, row) in rehomes {
        if row.decision.trim().is_empty() || from == row.to {
            return Err(fail(
                "invalid_rehome",
                "/changes/rehomes",
                "distinct source/destination and a decision reference required",
            ));
        }
        let amount = expected
            .remove(&from)
            .ok_or_else(|| fail("missing_account", "/changes/rehomes", from.as_str()))?;
        same(amount, row.amount.0, "/changes/rehomes")?;
        credits.push((row.to.clone(), amount));
    }
    for (to, amount) in credits {
        let old = expected.get(&to).copied().unwrap_or(0);
        expected.insert(to, add(old, amount, "/changes/rehomes")?);
    }
    let rewards = balances_map(&i.changes.reward_credits, "/changes/rewardCredits")?;
    same_map(
        &balances_map(&s.legacy_reward_credits, "/source/legacyRewardCredits")?,
        &rewards,
        "/changes/rewardCredits",
    )?;
    let debit = total(&i.changes.reward_credits, "/changes/rewardCredits")?;
    if debit > 0 {
        let pool = expected
            .get_mut(&i.bootstrap.mining_pool_account)
            .ok_or_else(|| {
                fail(
                    "missing_account",
                    "/changes/rewardCredits",
                    "reward pool must already be funded by the rehome",
                )
            })?;
        *pool = pool.checked_sub(debit).ok_or_else(|| {
            fail(
                "insufficient_reward_pool",
                "/changes/rewardCredits",
                "credits exceed pool",
            )
        })?;
        for (to, amount) in rewards {
            let old = expected.get(&to).copied().unwrap_or(0);
            expected.insert(to, add(old, amount, "/changes/rewardCredits")?);
        }
    }
    let actual = balances_map(&i.state.balances, "/state/balances")?;
    for (id, amount) in &actual {
        if *amount < 1_000_000 {
            return Err(fail("below_ed", "/state/balances", id.as_str()));
        }
    }
    same_map(&expected, &actual, "/state/balances")?;
    same(
        issuance,
        total(&i.state.balances, "/state/totalIssuance")?,
        "/state/totalIssuance",
    )?;
    assets(i)?;
    let src = &s.lp;
    let dst = &i.state.lp;
    let expected_lp = balances_map(&src.positions, "/source/lp/positions")?;
    let actual_lp = balances_map(&dst.positions, "/state/lp/positions")?;
    if src.positions.is_empty() || src.positions.iter().any(|r| r.amount.0 == 0) {
        return Err(fail(
            "lp_book_empty_or_zero",
            "/source/lp/positions",
            "migration profile requires nonempty positive LP book",
        ));
    }
    same(
        src.total.0,
        total(&src.positions, "/source/lp/total")?,
        "/source/lp/total",
    )?;
    same_map(&expected_lp, &actual_lp, "/state/lp/positions")?;
    same(src.total, dst.total, "/state/lp/total")?;
    same(src.d9_reserve, dst.d9_reserve, "/state/lp/d9Reserve")?;
    same(
        scale(src.usdt_reserve.0, 10_000, "/state/lp/usdtReserve")?,
        dst.usdt_reserve.0,
        "/state/lp/usdtReserve",
    )?;
    same(
        Some(dst.d9_reserve.0),
        actual.get(&i.bootstrap.amm_account).copied(),
        "/state/lp/d9Reserve",
    )?;
    let usdt = i
        .state
        .assets
        .iter()
        .find(|r| r.asset_id == 1)
        .ok_or_else(|| fail("missing_usdt", "/state/assets", "asset 1 required"))?;
    let holders = balances_map(&usdt.holders, "/state/assets/1/holders")?;
    same(
        Some(dst.usdt_reserve.0),
        holders.get(&i.bootstrap.amm_account).copied(),
        "/state/lp/usdtReserve",
    )?;
    Ok(())
}

fn assets(i: &ContractInput) -> Check {
    let source = keyed(&i.source.assets, |r| r.asset_id, "/source/assets")?;
    let target = keyed(&i.state.assets, |r| r.asset_id, "/state/assets")?;
    let moves = keyed(
        &i.changes.asset_rehomes,
        |r| (r.asset_id, r.from.clone()),
        "/changes/assetRehomes",
    )?;
    for (id, _) in moves.keys() {
        if !source.contains_key(id) {
            return Err(fail(
                "asset_coverage",
                "/changes/assetRehomes",
                "rehome names unknown asset",
            ));
        }
    }
    if source.len() != target.len() {
        return Err(fail("asset_coverage", "/state/assets", "asset set differs"));
    }
    for (id, src) in source {
        let dst = target
            .get(&id)
            .ok_or_else(|| fail("asset_coverage", "/state/assets", "missing asset"))?;
        let factor = if id == 1 {
            same(2, src.decimals, "/source/assets/1/decimals")?;
            same(6, dst.decimals, "/state/assets/1/decimals")?;
            10_000
        } else {
            same(src.decimals, dst.decimals, "/state/assets/decimals")?;
            1
        };
        let mut src_rows = balances_map(&src.holders, "/source/assets/holders")?;
        let dst_rows = balances_map(&dst.holders, "/state/assets/holders")?;
        if src.holder_count.0 != src.holders.len() as u64
            || dst.holder_count.0 != dst.holders.len() as u64
        {
            return Err(fail(
                "record_count_mismatch",
                "/state/assets/holderCount",
                "holder count must equal the actual unique records",
            ));
        }
        let mut credits = Vec::new();
        for ((asset, from), row) in &moves {
            if *asset != id {
                continue;
            }
            if row.decision.trim().is_empty() || *from == row.to {
                return Err(fail(
                    "invalid_rehome",
                    "/changes/assetRehomes",
                    "distinct identities and decision required",
                ));
            }
            let amount = src_rows
                .remove(from)
                .ok_or_else(|| fail("missing_account", "/changes/assetRehomes", from.as_str()))?;
            same(amount, row.source_amount.0, "/changes/assetRehomes")?;
            credits.push((row.to.clone(), amount));
        }
        for (to, amount) in credits {
            let old = src_rows.get(&to).copied().unwrap_or(0);
            src_rows.insert(to, add(old, amount, "/changes/assetRehomes")?);
        }
        let mut expected = BTreeMap::new();
        for (account, amount) in src_rows {
            if amount == 0 {
                return Err(fail(
                    "zero_asset_holder",
                    "/source/assets/holders",
                    "zero rows need an explicit source disposition, SDK does not create holders",
                ));
            }
            expected.insert(account, scale(amount, factor, "/state/assets/holders")?);
        }
        same_map(&expected, &dst_rows, "/state/assets/holders")?;
        same(
            src.total_supply.0,
            total(&src.holders, "/source/assets/totalSupply")?,
            "/source/assets/totalSupply",
        )?;
        same(
            scale(src.total_supply.0, factor, "/state/assets/totalSupply")?,
            dst.total_supply.0,
            "/state/assets/totalSupply",
        )?;
    }
    Ok(())
}
