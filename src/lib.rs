//! Versioned host-side input contract, not a release gate or raw-spec verifier.
//! Pallet builders and the independent D9-173 verifier consume this contract.
//! No storage is written by this crate.

mod binding;
mod canonical;
mod inventory;
mod model;
mod scalar;
mod validation;

/// Shared vectors embedded for downstream tests without a sibling checkout.
pub mod fixtures {
    pub const COMPLETE: &str = include_str!("../fixtures/complete.json");
    pub const EXPECTED: &str = include_str!("../fixtures/expected.json");
    pub const CASES: &str = include_str!("../fixtures/cases.json");
    pub const MANIFEST: &str = include_str!("../fixture-manifest.json");
}

pub use binding::*;
pub use canonical::{dataset_digest, input_digest};
pub use inventory::{inventory, validate_inventory, Item as InventoryItem};
pub use model::*;
pub use scalar::*;
pub use validation::{validate, ContractReport, Violation};

pub const CONTRACT_VERSION: &str = "d9-native-genesis/0.1.0-rc.2";

pub fn schema() -> schemars::schema::RootSchema {
    let mut schema = schemars::schema_for!(ContractInput);
    // Every DTO property is required, including explicit nullable Options.
    // Serde's required_nullable enforces the same distinction at decode time.
    if let Some(object) = &mut schema.schema.object {
        object.required.extend(object.properties.keys().cloned());
    }
    for value in schema.definitions.values_mut() {
        if let schemars::schema::Schema::Object(value) = value {
            if let Some(object) = &mut value.object {
                object.required.extend(object.properties.keys().cloned());
            }
        }
    }
    schema
}

/// Strictly decode the transport contract. Integers are decimal strings,
/// AccountIds are checksum-verified canonical SS58 prefix 9.
pub fn parse(bytes: &[u8]) -> Result<ContractInput, String> {
    let mut de = serde_json::Deserializer::from_slice(bytes);
    let value = serde_path_to_error::deserialize(&mut de).map_err(|e| e.to_string())?;
    de.end().map_err(|e| e.to_string())?;
    Ok(value)
}
