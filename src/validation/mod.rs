use crate::*;
use serde::Serialize;
use std::collections::BTreeMap;

mod balances;
mod composition;
mod custody;
mod network;
mod people;
mod pop;
mod source;

pub use custody::{
    multisig_account, weak_public_key, well_known_development_key, MULTISIG_MAX_SIGNATORIES,
};
pub use pop::{
    custody_pop_message, custody_pop_message_sha256, PendingAttestationVerification,
    BLESSED_SIGNER_PCR0, CUSTODY_POP_DOMAIN, MAX_ATTESTATION_DOCUMENT_BYTES,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Violation {
    pub code: &'static str,
    pub path: String,
    pub related_path: Option<String>,
    pub message: String,
}
pub(super) type Check = Result<(), Violation>;
pub(super) fn fail(
    code: &'static str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> Violation {
    Violation {
        code,
        path: path.into(),
        related_path: None,
        message: message.into(),
    }
}
pub(super) fn same<T: PartialEq + std::fmt::Debug>(a: T, b: T, path: &str) -> Check {
    if a == b {
        Ok(())
    } else {
        Err(fail(
            "source_target_mismatch",
            path,
            format!("expected {a:?}; actual {b:?}"),
        ))
    }
}
pub(super) fn add(a: u128, b: u128, path: &str) -> Result<u128, Violation> {
    a.checked_add(b)
        .ok_or_else(|| fail("overflow", path, "u128 addition overflow"))
}

// Report the first offending key, never format an entire million-row ledger.
pub(super) fn same_map<K: Ord + std::fmt::Display, V: PartialEq + std::fmt::Debug>(
    expected: &BTreeMap<K, V>,
    actual: &BTreeMap<K, V>,
    path: &str,
) -> Check {
    for (key, value) in expected {
        if actual.get(key) != Some(value) {
            return Err(fail(
                "source_target_mismatch",
                format!("{path}/{key}"),
                format!("expected {value:?}; actual {:?}", actual.get(key)),
            ));
        }
    }
    if let Some((key, value)) = actual.iter().find(|(key, _)| !expected.contains_key(key)) {
        return Err(fail(
            "source_target_mismatch",
            format!("{path}/{key}"),
            format!("unexpected record {value:?}"),
        ));
    }
    Ok(())
}
pub(super) fn scale(a: u128, factor: u128, path: &str) -> Result<u128, Violation> {
    a.checked_mul(factor)
        .ok_or_else(|| fail("overflow", path, "u128 multiplication overflow"))
}
pub(super) fn keyed<'a, T, K: Ord + Clone>(
    rows: &'a [T],
    key: impl Fn(&T) -> K,
    path: &str,
) -> Result<BTreeMap<K, &'a T>, Violation> {
    let mut result = BTreeMap::new();
    let mut indices = BTreeMap::new();
    for (i, row) in rows.iter().enumerate() {
        let k = key(row);
        if let Some(first) = indices.insert(k.clone(), i) {
            let mut e = fail(
                "duplicate_key",
                format!("{path}/{i}"),
                "duplicate normalized storage key",
            );
            e.related_path = Some(format!("{path}/{first}"));
            return Err(e);
        }
        result.insert(k, row);
    }
    Ok(result)
}
pub(super) fn total(rows: &[Balance], path: &str) -> Result<u128, Violation> {
    rows.iter()
        .try_fold(0, |sum, row| add(sum, row.amount.0, path))
}
pub(super) fn balances_map(
    rows: &[Balance],
    path: &str,
) -> Result<BTreeMap<Address, u128>, Violation> {
    Ok(keyed(rows, |r| r.account.clone(), path)?
        .into_iter()
        .map(|(k, v)| (k, v.amount.0))
        .collect())
}
pub(super) fn timestamp(value: Millis, upper: Millis, path: &str) -> Check {
    if value.0 == 0 || value > upper {
        Err(fail(
            "timestamp_bounds",
            path,
            "expected 0 < timestamp <= source timestamp (milliseconds)",
        ))
    } else {
        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractReport {
    pub contract_version: &'static str,
    pub input_digest: Digest,
    pub contract_digest: Digest,
    /// `input-contract-conformance`, or `CHECK_ATTESTATION_PENDING` when any custody
    /// signatory's enclave attestation still needs producer verification.
    pub check: &'static str,
    pub release_gate_evaluated: bool,
    pub independent_evidence_required: Vec<&'static str>,
    /// Enclave-attested signatories whose attestation documents this contract did not
    /// verify; empty when every signatory proved possession with a verified signature.
    pub pending_attestation_verifications: Vec<PendingAttestationVerification>,
}

/// `ContractReport::check` for a conforming input with no enclave-attested signatory.
pub const CHECK_CONFORMANCE: &str = "input-contract-conformance";
/// `ContractReport::check` when enclave attestations are pending producer verification.
pub const CHECK_ATTESTATION_PENDING: &str =
    "input-contract-conformance;attestation-verification-pending";

/// Input conformance only. Does not authenticate V1/ABI, inspect raw storage,
/// verify WASM, or acknowledge downstream review. Those remain release gates.
pub fn validate(input: &ContractInput) -> Result<ContractReport, Violation> {
    if input.contract_version != CONTRACT_VERSION {
        return Err(fail(
            "contract_version",
            "/contractVersion",
            "unsupported contract version",
        ));
    }
    network::check(input)?;
    if !input.changes.unresolved.is_empty() {
        return Err(fail(
            "unresolved_disposition",
            "/changes/unresolved/0",
            "requires a recorded Yvan disposition",
        ));
    }
    // Null inventory items must never silently become absent runtime storage.
    // Future unresolved classifications continue to block real input conformance.
    if input.purpose == Purpose::MigrationInput {
        require_resolved_inventory(&inventory())?;
    }
    source::check(input)?;
    people::check_locks(input)?;
    composition::check(input)?;
    balances::check(input)?;
    people::check(input)?;
    same(
        &input.source.counters,
        &input.state.counters,
        "/state/counters",
    )?;
    let pending_attestation_verifications = pop::pending_attestations(input);
    let mut independent_evidence_required = vec![
        "D9-380 downstream reviews",
        "D9-307 complete storage/field coverage and artifact provenance",
        "D9-173 independent raw-spec/chain reconciliation",
        "D9-370 final composition",
        "D9-315 reproducible WASM",
        "D9-380 ADR verified complete V1 resolution archive before cutover; delivery owner unassigned",
        "D9-211 legacy settlement completion/refund or approved funded arrangement before irreversible cutover",
        "D9-250 clean V2 processing boundary before bridge activation",
        "D9-370/173 fresh zero/empty dispositions and opening reward watermark readback",
        "D9-400 CUSTODY: producer projects sudo.key, every <pallet>.admin and the asset owners (asset 1 = usdtOwner, others = sudo) from these contract addresses",
        "D9-400 CUSTODY ceremony: each signatory key belongs to its intended custodian (identity and custody records, raw proofs signed with an offline tool that displays the decoded message). The contract itself verifies signature proof-of-possession and refuses the all-zero sr25519 identity, every ed25519 small-order encoding and every ed25519 key that is not a canonical torsion-free point; it cannot tell whose key a valid proof comes from",
        "D9-400 ASSET_SET: producer proves assets.assets and assets.metadata ID sets each equal bootstrap.assetIds",
        "D9-400 NETWORK: producer proves chain-spec id/name/chainType and manifest network equal chain, with no bootNodes and null telemetryEndpoints",
    ];
    if !pending_attestation_verifications.is_empty() {
        independent_evidence_required.push(
            "D9-400 CUSTODY: enclave-attested custody signatories (listed in pendingAttestationVerifications): the producer verifies the attestation document with d9-enclave-common attest_verify (COSE signature, certificate chain to the pinned AWS Nitro root validated at the document's own timestamp, PCR0 == expectedPcr0, user_data == popMessageSha256), and expectedPcr0 equals the blessed signer measurement recorded in the PCR0 ledger (d9-v2-docs pcr0-ledger)",
        );
    }
    Ok(ContractReport {
        contract_version: CONTRACT_VERSION,
        input_digest: input_digest(input),
        contract_digest: contract_digest(),
        check: if pending_attestation_verifications.is_empty() {
            CHECK_CONFORMANCE
        } else {
            CHECK_ATTESTATION_PENDING
        },
        release_gate_evaluated: false,
        independent_evidence_required,
        pending_attestation_verifications,
    })
}

fn require_resolved_inventory(rows: &[InventoryItem<'_>]) -> Check {
    let pending: Vec<_> = rows
        .iter()
        .filter(|r| r.provenance.is_none())
        .map(|r| format!("{}.{} ({})", r.pallet, r.storage, r.decision))
        .collect();
    if !pending.is_empty() {
        return Err(fail(
            "inventory_disposition_pending",
            "/inventory",
            pending.join("; "),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn any_future_unresolved_inventory_still_blocks_migration_input() {
        let mut rows = inventory();
        require_resolved_inventory(&rows).unwrap();
        rows.push(InventoryItem {
            pallet: "future",
            storage: "Unclassified",
            provenance: None,
            genesis_field: None,
            input_path: "pending",
            owner: "Yvan",
            decision: "future decision",
            implementation: "decision-pending",
        });
        let error = require_resolved_inventory(&rows).unwrap_err();
        assert_eq!(error.code, "inventory_disposition_pending");
        assert!(error.message.contains("future.Unclassified"));
    }
}
