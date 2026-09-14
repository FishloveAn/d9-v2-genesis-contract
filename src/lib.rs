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
pub use validation::{
    custody_pop_message, custody_pop_message_sha256, custody_pop_payload, multisig_account,
    validate, well_known_development_key, ContractReport, Violation, CUSTODY_POP_DOMAIN,
    MAX_ATTESTATION_DOCUMENT_BYTES, MULTISIG_MAX_SIGNATORIES,
};

pub const CONTRACT_VERSION: &str = "d9-native-genesis/0.1.0-rc.7";

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

/// Why [`parse`] refused a document. Consumers match on the variant; the
/// `Display` text is for humans only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The top-level `contractVersion` string was readable and differs from
    /// [`CONTRACT_VERSION`]. Reported before typed decoding.
    UnsupportedVersion { found: String },
    /// Strict typed decoding failed (JSON syntax, unknown, duplicate or missing
    /// fields, non-canonical scalars). The string is the decoder's path and message.
    Decode(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedVersion { found } => write!(
                f,
                "contract_version: unsupported contract version {found:?}; expected {CONTRACT_VERSION}"
            ),
            Self::Decode(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for ParseError {}

/// Strictly decode the transport contract. Integers are decimal strings,
/// AccountIds are checksum-verified canonical SS58 prefix 9.
///
/// A document whose top-level `contractVersion` string differs from
/// [`CONTRACT_VERSION`] is refused with [`ParseError::UnsupportedVersion`] before
/// typed decoding, so an older RC reports its version instead of a field error.
pub fn parse(bytes: &[u8]) -> Result<ContractInput, ParseError> {
    // CHOICE: peek with a one-field struct rather than serde_json::Value. It
    // skips the rest of the document without allocating a second copy of
    // million-row ledgers. Anything it cannot read (invalid JSON, a missing or
    // non-string or duplicated version) falls through to the strict decoder.
    #[derive(serde::Deserialize)]
    struct VersionPeek {
        #[serde(rename = "contractVersion")]
        contract_version: String,
    }
    if let Ok(peek) = serde_json::from_slice::<VersionPeek>(bytes) {
        if peek.contract_version != CONTRACT_VERSION {
            return Err(ParseError::UnsupportedVersion {
                found: peek.contract_version,
            });
        }
    }
    let decode = |error: String| ParseError::Decode(error);
    let mut de = serde_json::Deserializer::from_slice(bytes);
    let value = serde_path_to_error::deserialize(&mut de).map_err(|e| decode(e.to_string()))?;
    de.end().map_err(|e| decode(e.to_string()))?;
    Ok(value)
}
