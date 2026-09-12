use d9_genesis_contract::*;
use serde::Deserialize;
use serde_json::{json, Value};
use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};
use std::collections::BTreeSet;

fn input_value() -> Value {
    serde_json::from_str(include_str!("../fixtures/complete.json")).unwrap()
}
fn input() -> ContractInput {
    parse(include_bytes!("../fixtures/complete.json")).unwrap()
}
fn golden() -> Value {
    serde_json::from_str(include_str!("../fixtures/expected.json")).unwrap()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    patches: Vec<Patch>,
    refresh_source: Vec<String>,
    expected: Expected,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Patch {
    op: String,
    path: String,
    value: Option<Value>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Expected {
    phase: String,
    code: String,
    path_prefix: String,
}

fn apply(root: &mut Value, patch: Patch) {
    let (parent, key) = patch.path.rsplit_once('/').unwrap();
    let parent = root
        .pointer_mut(parent)
        .expect("fixture patch must address existing parent");
    match parent {
        Value::Object(map) => match patch.op.as_str() {
            "remove" => {
                assert!(map.remove(key).is_some());
            }
            "replace" => {
                assert!(map.contains_key(key));
                map.insert(key.into(), patch.value.unwrap_or(Value::Null));
            }
            "add" => {
                assert!(!map.contains_key(key));
                map.insert(key.into(), patch.value.unwrap_or(Value::Null));
            }
            _ => panic!("unsupported fixture patch"),
        },
        Value::Array(rows) => {
            assert_eq!(patch.op, "replace");
            rows[key.parse::<usize>().unwrap()] = patch.value.unwrap_or(Value::Null);
        }
        _ => panic!("invalid fixture patch parent"),
    }
}

fn refresh_evidence(value: &mut Value, datasets: &[String]) {
    let fields = [
        ("balances", "/source/balances"),
        ("assets", "/source/assets"),
        ("referrals", "/source/referrals"),
        ("referral-counts", "/source/referralCounts"),
        ("burn", "/source/burnAccounts"),
        ("merchant", "/source/merchantAccounts"),
        ("merchant-expiries", "/source/merchantExpiries"),
        ("voting", "/source/votingInterests"),
        ("locks", "/source/locks"),
        ("lp", "/source/lp/positions"),
        ("legacy-rewards", "/source/legacyRewardCredits"),
    ];
    for name in datasets {
        let path = fields.iter().find(|(k, _)| k == name).unwrap().1;
        let rows = value.pointer(path).unwrap().as_array().unwrap();
        let digest = dataset_digest(name, rows).as_str().to_owned();
        let count = rows.len().to_string();
        let e = value["source"]["datasets"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["dataset"] == name.as_str())
            .unwrap();
        e["payloadDigest"] = json!(digest);
        e["recordCount"] = json!(count);
    }
}

#[test]
fn shared_negative_cases_refuse_at_the_declared_boundary() {
    let cases: Vec<Case> = serde_json::from_str(include_str!("../fixtures/cases.json")).unwrap();
    let mut ids = BTreeSet::new();
    assert!(cases.len() >= 54);
    for case in cases {
        assert!(ids.insert(case.id.clone()), "duplicate fixture id");
        let mut value = input_value();
        for patch in case.patches {
            apply(&mut value, patch);
        }
        refresh_evidence(&mut value, &case.refresh_source);
        let parsed = parse(&serde_json::to_vec(&value).unwrap());
        if case.expected.phase == "decode" {
            assert!(parsed.is_err(), "{} unexpectedly decoded", case.id);
            assert_eq!(case.expected.code, "contract_decode");
        } else {
            let parsed = parsed.unwrap_or_else(|e| panic!("{} failed decoding: {e}", case.id));
            let error = validate(&parsed).unwrap_err();
            assert_eq!(error.code, case.expected.code, "{}", case.id);
            assert!(
                error.path.starts_with(&case.expected.path_prefix),
                "{}: {:?}",
                case.id,
                error
            );
        }
    }
}

#[test]
fn complete_fixture_matches_independently_authored_values_and_hashes() {
    let input = input();
    let result = validate(&input).unwrap();
    let golden = golden();
    assert!(!result.release_gate_evaluated);
    assert_eq!(result.input_digest.as_str(), golden["inputDigest"]);
    assert_eq!(input.state.lp.total.0.to_string(), golden["totalLpTokens"]);
    assert_eq!(
        serde_json::to_value(&input.state.lp.positions).unwrap(),
        golden["lpPositions"]
    );
    assert_eq!(input.source.locks.len().to_string(), golden["rawLockCount"]);
    assert_eq!(
        input.state.locks.len().to_string(),
        golden["seededLockCount"]
    );
    assert_eq!(
        serde_json::to_value(&input.changes.excluded_judicial_locks).unwrap(),
        golden["excludedJudicialLocks"]
    );
    assert_eq!(
        input.source.locks.len(),
        input.state.locks.len() + input.changes.excluded_judicial_locks.len()
    );
    let burn = &input.state.burn_accounts[0];
    assert_eq!(
        (burn.balance_due.0 - burn.balance_paid.0).to_string(),
        golden["burnRemaining"]
    );
    assert_eq!(
        input
            .state
            .balances
            .iter()
            .map(|r| r.amount.0)
            .sum::<u128>()
            .to_string(),
        golden["totalIssuance"]
    );
    for e in input.source.datasets {
        assert_eq!(
            e.payload_digest.as_str(),
            golden["sourceDatasetDigests"][&e.dataset]
        );
    }
}

#[test]
fn prefix_alias_is_rejected_and_duplicate_reports_both_rows() {
    let mut value = input_value();
    let id = input().bootstrap.sudo.account_id();
    value["bootstrap"]["sudo"] = json!(id.to_ss58check_with_version(Ss58AddressFormat::custom(42)));
    assert!(parse(&serde_json::to_vec(&value).unwrap())
        .unwrap_err()
        .contains("prefix 9"));
    let mut input = input();
    input
        .state
        .lp
        .positions
        .push(input.state.lp.positions[0].clone());
    let e = validate(&input).unwrap_err();
    assert_eq!(e.code, "duplicate_key");
    assert_eq!(e.related_path.as_deref(), Some("/state/lp/positions/0"));
}

#[test]
fn digests_commit_to_ownership_and_domain_but_not_dataset_order() {
    let input = input();
    let rows = &input.state.lp.positions;
    let digest = dataset_digest("lp", rows);
    let mut reversed = rows.clone();
    reversed.reverse();
    assert_eq!(digest, dataset_digest("lp", &reversed));
    reversed[0].amount = Amount(reversed[0].amount.0 + 1);
    assert_ne!(digest, dataset_digest("lp", &reversed));
    assert_ne!(digest, dataset_digest("assets", rows));
    let mut changed = input.clone();
    changed.bootstrap.sudo = changed.bootstrap.validators[0].account.clone();
    assert_ne!(input_digest(&input), input_digest(&changed));
    changed = input.clone();
    changed.build.wasm_digest = Digest::try_from("f".repeat(64)).unwrap();
    assert_ne!(input_digest(&input), input_digest(&changed));
}

#[test]
fn retired_ingest_and_implicit_nullable_defaults_are_not_wire_compatible() {
    let mut value = input_value();
    value["changes"]["ingest"] = json!([]);
    assert!(parse(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = input_value();
    value["source"]["datasets"][0]
        .as_object_mut()
        .unwrap()
        .remove("contract");
    assert!(parse(&serde_json::to_vec(&value).unwrap()).is_err());
    let bytes = include_str!("../fixtures/complete.json").replacen(
        "\"contractVersion\":",
        "\"contractVersion\": \"duplicate\", \"contractVersion\":",
        1,
    );
    assert!(parse(bytes.as_bytes()).is_err());
}

#[test]
fn schema_is_generated_from_the_same_rust_contract() {
    let actual = serde_json::to_value(schema()).unwrap();
    let committed: Value = serde_json::from_str(include_str!("../schema.json")).unwrap();
    assert_eq!(actual, committed);
    assert!(actual["definitions"]["MerchantAccount"]["required"]
        .as_array()
        .unwrap()
        .contains(&json!("lastConversion")));
    assert_eq!(
        actual["definitions"]["MerchantAccount"]["additionalProperties"],
        false
    );
}

#[test]
fn funded_approved_account_is_retained_without_an_exclusion() {
    let mut value = input_value();
    let raw = value["source"]["locks"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    let account = raw["account"].clone();
    value["changes"]["excludedJudicialLocks"] = json!([]);
    value["source"]["balances"]
        .as_array_mut()
        .unwrap()
        .push(json!({"account": account, "free": "1000000", "reserved": "0"}));
    let issuance = value["source"]["totalIssuance"]
        .as_str()
        .unwrap()
        .parse::<u128>()
        .unwrap();
    value["source"]["totalIssuance"] = json!((issuance + 1000000).to_string());
    value["state"]["balances"]
        .as_array_mut()
        .unwrap()
        .push(json!({"account": account, "amount": "1000000"}));
    value["state"]["locks"].as_array_mut().unwrap().push(raw);
    refresh_evidence(&mut value, &["balances".to_owned()]);
    let input = parse(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(!validate(&input).unwrap().release_gate_evaluated);
}

#[test]
fn migration_input_conformance_is_not_archive_settlement_or_release_approval() {
    let mut input = input();
    input.purpose = Purpose::MigrationInput;
    let report = validate(&input).unwrap();
    assert_eq!(report.check, "input-contract-conformance");
    assert!(!report.release_gate_evaluated);
    for required in [
        "resolution archive",
        "legacy settlement",
        "clean V2 processing",
        "watermark readback",
    ] {
        assert!(report
            .independent_evidence_required
            .iter()
            .any(|line| line.contains(required)));
    }
    input.changes.unresolved.push(PendingDecision {
        item: "future.Unclassified".into(),
        issue: "future-decision-required".into(),
    });
    assert_eq!(validate(&input).unwrap_err().code, "unresolved_disposition");
}
