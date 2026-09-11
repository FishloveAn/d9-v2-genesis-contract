use d9_genesis_contract::*;

fn fixture() -> (ContractInput, ArtifactBinding) {
    (
        parse(include_bytes!("../fixtures/complete.json")).unwrap(),
        serde_json::from_str(include_str!("../fixtures/binding.json")).unwrap(),
    )
}
const SPEC: &[u8] = include_bytes!("../fixtures/mock-spec.bytes");
const REPORT: &[u8] = include_bytes!("../fixtures/mock-report.bytes");

#[test]
fn detached_binding_matches_independently_hashed_fixture_bytes() {
    let (input, binding) = fixture();
    verify_binding(&input, &binding, SPEC, REPORT).unwrap();
    assert_eq!(binding.contract_digest, contract_digest());
    let committed: serde_json::Value =
        serde_json::from_str(include_str!("../binding.schema.json")).unwrap();
    assert_eq!(
        committed,
        serde_json::to_value(schemars::schema_for!(ArtifactBinding)).unwrap()
    );
}

#[test]
fn final_spec_and_report_cannot_be_changed_after_binding() {
    let (input, binding) = fixture();
    assert_eq!(
        verify_binding(&input, &binding, b"authority patched after build", REPORT),
        Err("raw_spec_digest_mismatch")
    );
    assert_eq!(
        verify_binding(&input, &binding, SPEC, b"another report"),
        Err("report_digest_mismatch")
    );
}

#[test]
fn changed_authority_wasm_and_contract_cannot_reuse_old_binding() {
    let (input, binding) = fixture();
    let mut changed = input.clone();
    changed.bootstrap.sudo = changed.bootstrap.validators[0].account.clone();
    assert_eq!(
        verify_binding(&changed, &binding, SPEC, REPORT),
        Err("input_binding_mismatch")
    );
    changed = input.clone();
    changed.build.wasm_digest = Digest::try_from("f".repeat(64)).unwrap();
    assert_eq!(
        verify_binding(&changed, &binding, SPEC, REPORT),
        Err("input_binding_mismatch")
    );
    let mut stale = binding.clone();
    stale.contract_digest = Digest::try_from("0".repeat(64)).unwrap();
    assert_eq!(
        verify_binding(&input, &stale, SPEC, REPORT),
        Err("contract_digest_mismatch")
    );
}
