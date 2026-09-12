use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// This is a review inventory, not the runtime's StorageInfo manifest.
/// Null provenance records a pending decision; it is NOT a fourth variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Item<'a> {
    pub pallet: &'a str,
    pub storage: &'a str,
    pub provenance: Option<&'a str>,
    pub genesis_field: Option<&'a str>,
    pub input_path: &'a str,
    pub owner: &'a str,
    pub decision: &'a str,
    pub implementation: &'a str,
}

pub fn inventory() -> Vec<Item<'static>> {
    serde_json::from_str(include_str!("../inventory.json")).expect("checked-in inventory is tested")
}

/// Validates the portable inventory's wire-level declaration rules. The pallet
/// adapter independently checks these rows with d9-core::validate_manifest and
/// checks source-name coverage against its own source checkout.
pub fn validate_inventory() -> Result<(), String> {
    validate_rows(&inventory())
}

fn validate_rows(rows: &[Item<'_>]) -> Result<(), String> {
    if rows.is_empty() {
        return Err("empty inventory".into());
    }
    let mut keys = BTreeSet::new();
    for row in rows {
        if row.pallet.is_empty() || row.storage.is_empty() || row.genesis_field == Some("") {
            return Err("empty pallet/storage/field identifier".into());
        }
        if !keys.insert((row.pallet, row.storage)) {
            return Err(format!("duplicate owner: {}.{}", row.pallet, row.storage));
        }
        if row.owner.is_empty() || row.decision.is_empty() {
            return Err("missing owner/decision".into());
        }
        match (row.provenance, row.genesis_field) {
            (Some("Genesis"), Some(_)) | (Some("Derived"), None) | (Some("NotMigrated"), _) => (),
            (None, None) if row.implementation == "decision-pending" => (),
            _ => {
                return Err(format!(
                    "invalid provenance: {}.{}",
                    row.pallet, row.storage
                ))
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inventory_records_decisions_and_rejects_conflicting_declarations() {
        validate_inventory().unwrap();
        let mut rows = inventory();
        for storage in [
            "Resolutions",
            "ProposalFeeVolume",
            "UserNonce",
            "CumulativeBridgedOut",
            "PendingOutbound",
        ] {
            let row = rows.iter().find(|r| r.storage == storage).unwrap();
            assert_eq!(row.provenance, Some("NotMigrated"));
            assert!(row.decision.contains("2026-09-12"));
            assert!(row.decision.contains("/docs/decisions/migration/"));
        }
        rows.push(rows[0].clone());
        assert!(validate_rows(&rows)
            .unwrap_err()
            .contains("duplicate owner"));
        rows.pop();
        rows[0].provenance = Some("Genesis");
        rows[0].genesis_field = None;
        assert!(validate_rows(&rows).is_err());
        rows[0].provenance = Some("Ingest");
        assert!(validate_rows(&rows).is_err());
    }
}
