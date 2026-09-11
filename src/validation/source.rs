use super::*;

pub(super) fn check(i: &ContractInput) -> Check {
    let s = &i.source;
    if !s.finalized {
        return Err(fail(
            "unfinalized_source",
            "/source/finalized",
            "only finalized snapshots",
        ));
    }
    timestamp(s.timestamp_ms, s.timestamp_ms, "/source/timestampMs")?;
    let evidence = keyed(&s.datasets, |r| r.dataset.clone(), "/source/datasets")?;
    let expected = [
        (
            "balances",
            s.balances.len(),
            dataset_digest("balances", &s.balances),
        ),
        (
            "assets",
            s.assets.len(),
            dataset_digest("assets", &s.assets),
        ),
        (
            "referrals",
            s.referrals.len(),
            dataset_digest("referrals", &s.referrals),
        ),
        (
            "referral-counts",
            s.referral_counts.len(),
            dataset_digest("referral-counts", &s.referral_counts),
        ),
        (
            "burn",
            s.burn_accounts.len(),
            dataset_digest("burn", &s.burn_accounts),
        ),
        (
            "merchant",
            s.merchant_accounts.len(),
            dataset_digest("merchant", &s.merchant_accounts),
        ),
        (
            "merchant-expiries",
            s.merchant_expiries.len(),
            dataset_digest("merchant-expiries", &s.merchant_expiries),
        ),
        (
            "voting",
            s.voting_interests.len(),
            dataset_digest("voting", &s.voting_interests),
        ),
        ("locks", s.locks.len(), dataset_digest("locks", &s.locks)),
        (
            "lp",
            s.lp.positions.len(),
            dataset_digest("lp", &s.lp.positions),
        ),
        (
            "counters",
            1,
            dataset_digest("counters", std::slice::from_ref(&s.counters)),
        ),
        (
            "legacy-rewards",
            s.legacy_reward_credits.len(),
            dataset_digest("legacy-rewards", &s.legacy_reward_credits),
        ),
    ];
    if evidence.len() != expected.len() {
        return Err(fail(
            "dataset_coverage",
            "/source/datasets",
            "expected all twelve declared datasets, including empty datasets",
        ));
    }
    for (name, count, digest) in expected {
        let e = evidence.get(name).ok_or_else(|| {
            fail(
                "dataset_coverage",
                "/source/datasets",
                format!("missing {name}"),
            )
        })?;
        let path = format!("/source/datasets/{name}");
        if e.block_hash != s.block_hash {
            return Err(fail(
                "source_pin_mismatch",
                &path,
                "dataset belongs to another block",
            ));
        }
        if e.source_map.trim().is_empty() {
            return Err(fail("source_map_missing", &path, "source map is required"));
        }
        let expected_map = match name {
            "balances" => Some("System.Account"),
            "referrals" => Some("D9Referral.ReferralRelationships"),
            "referral-counts" => Some("D9Referral.DirectReferralsCount"),
            "voting" => Some("D9NodeVoting.UsersVotingInterests"),
            "locks" => Some("CouncilLock.LockedAccounts"),
            _ => None,
        };
        if expected_map.is_some_and(|map| e.source_map != map) {
            return Err(fail(
                "source_map_mismatch",
                &path,
                format!("expected {}", expected_map.unwrap()),
            ));
        }
        if e.record_count.0 != count as u64 {
            return Err(fail(
                "record_count_mismatch",
                &path,
                format!("records={count}; declared={}", e.record_count.0),
            ));
        }
        if e.payload_digest != digest {
            return Err(fail(
                "dataset_digest_mismatch",
                &path,
                "digest must commit to the actual normalized source records",
            ));
        }
        let is_contract = matches!(
            name,
            "assets"
                | "burn"
                | "merchant"
                | "merchant-expiries"
                | "lp"
                | "counters"
                | "legacy-rewards"
        );
        let complete = e.contract.is_some() && e.code_hash.is_some() && e.abi_digest.is_some();
        let absent = e.contract.is_none() && e.code_hash.is_none() && e.abi_digest.is_none();
        if (is_contract && !complete) || (!is_contract && !absent) {
            return Err(fail("source_abi_binding", &path, "contract datasets require address/codeHash/abiDigest; pallet datasets require explicit nulls"));
        }
        if name == "burn" {
            if e.source_map != "main_pool.portfolios" || e.supplements.len() != 1 {
                return Err(fail(
                    "burn_source_authority",
                    &path,
                    "main_pool.portfolios owns money; one burner supplement owns interaction/boost",
                ));
            }
            let extra = &e.supplements[0];
            if extra.source_map != "burner.accounts"
                || extra.block_hash != s.block_hash
                || extra.fields != ["lastInteraction", "referralBoostCoefficients"]
            {
                return Err(fail(
                    "burn_source_authority",
                    &path,
                    "supplement cannot override monetary fields or use a different block",
                ));
            }
        } else if !e.supplements.is_empty() {
            return Err(fail(
                "undeclared_source",
                &path,
                "supplementary sources require a versioned field ownership contract",
            ));
        }
    }
    // Independent per-parent baseline: both wrong-parent attribution and
    // symmetric omission must disagree with this recorded source dataset.
    let mut counts = BTreeMap::<Address, u64>::new();
    for row in keyed(&s.referrals, |r| r.child.clone(), "/source/referrals")?.values() {
        if row.child == row.parent {
            return Err(fail(
                "self_referral",
                "/source/referrals",
                "child equals parent",
            ));
        }
        let count = counts.entry(row.parent.clone()).or_default();
        *count = count
            .checked_add(1)
            .ok_or_else(|| fail("overflow", "/source/referrals", "count overflow"))?;
    }
    let recorded: BTreeMap<_, _> = keyed(
        &s.referral_counts,
        |r| r.parent.clone(),
        "/source/referralCounts",
    )?
    .into_iter()
    .map(|(k, v)| (k, v.count.0))
    .collect();
    same_map(&counts, &recorded, "/source/referralCounts")?;
    let refs = keyed(&i.state.referrals, |r| r.child.clone(), "/state/referrals")?;
    let source_refs = keyed(&s.referrals, |r| r.child.clone(), "/source/referrals")?;
    // Deliberately no acyclicity check: DEC-16 carries historical cycles.
    same_map(&source_refs, &refs, "/state/referrals")?;
    // Source bindings are evidence references, not independently authenticated
    // by this checker. D9-378/D9-173 must verify the external source baseline.
    Ok(())
}
