use crate::{input_digest, schema, BuildIdentity, ContractInput, Digest};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

/// Detached receipt: raw_spec_digest must not be embedded inside the bytes it
/// hashes. D9-370 records this AFTER the final authority/state composition.
/// This binds bytes and inputs, not their authenticity or storage correctness.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactBinding {
    pub contract_version: String,
    pub contract_digest: Digest,
    pub input_digest: Digest,
    pub build: BuildIdentity,
    pub raw_spec_digest: Digest,
    pub checker_report_digest: Digest,
}

pub fn bytes_digest(bytes: &[u8]) -> Digest {
    Digest(format!("{:x}", Sha256::digest(bytes)))
}

/// Commits to the generated schema and the reviewed rule/inventory payloads.
/// Actual Rust and dependency revisions must also be pinned in review evidence.
pub fn contract_digest() -> Digest {
    let inventory: serde_json::Value =
        serde_json::from_str(include_str!("../inventory.json")).expect("inventory is tested");
    let rules: serde_json::Value =
        serde_json::from_str(include_str!("../rules.json")).expect("rules are tested");
    input_digest(&serde_json::json!({
        "domain": "d9-genesis-contract-bundle-v1",
        "version": crate::CONTRACT_VERSION, "schema": schema(), "inventory": inventory, "rules": rules,
    }))
}

/// Refuses reuse after an input, schema/rule, build identity, raw-spec byte or
/// report byte change. D9-307/173 still validate actual runtime content.
pub fn verify_binding(
    input: &ContractInput,
    binding: &ArtifactBinding,
    raw_spec: &[u8],
    report: &[u8],
) -> Result<(), &'static str> {
    if binding.contract_version != crate::CONTRACT_VERSION {
        return Err("contract_version");
    }
    if binding.contract_digest != contract_digest() {
        return Err("contract_digest_mismatch");
    }
    if binding.input_digest != input_digest(input) || binding.build != input.build {
        return Err("input_binding_mismatch");
    }
    if binding.raw_spec_digest != bytes_digest(raw_spec) {
        return Err("raw_spec_digest_mismatch");
    }
    if binding.checker_report_digest != bytes_digest(report) {
        return Err("report_digest_mismatch");
    }
    Ok(())
}
