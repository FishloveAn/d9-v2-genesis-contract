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
    phase: Phase,
    code: String,
    path_prefix: String,
}
#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
enum Phase {
    Decode,
    Validation,
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
        if case.expected.phase == Phase::Decode {
            let error = parsed
                .err()
                .unwrap_or_else(|| panic!("{} unexpectedly decoded", case.id));
            match (case.expected.code.as_str(), &error) {
                ("contract_version", ParseError::UnsupportedVersion { .. })
                | ("contract_decode", ParseError::Decode(_)) => {}
                (expected, actual) => {
                    panic!("{}: expected {expected}, got {actual:?}", case.id)
                }
            }
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
    let id = input().bootstrap.sudo.address.account_id();
    value["bootstrap"]["sudo"]["address"] =
        json!(id.to_ss58check_with_version(Ss58AddressFormat::custom(42)));
    assert!(matches!(
        parse(&serde_json::to_vec(&value).unwrap()),
        Err(ParseError::Decode(message)) if message.contains("prefix 9")
    ));
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
    changed.bootstrap.sudo.address = changed.bootstrap.validators[0].account.clone();
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
        "proof-of-possession",
        "projects sudo.key",
        "excludes keyless accounts",
        "attest_verify",
        "assets.metadata ID sets",
        "no bootNodes",
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

#[test]
fn merchant_conversion_must_not_predate_creation() {
    let original = input_value();
    let created = original["source"]["merchantAccounts"][0]["createdAt"]
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    for last in [None, Some(created - 1), Some(created), Some(created + 1)] {
        let mut value = original.clone();
        let timestamp = last.map(|t| json!(t.to_string())).unwrap_or(Value::Null);
        value["source"]["merchantAccounts"][0]["lastConversion"] = timestamp.clone();
        value["state"]["merchantAccounts"][0]["lastConversion"] = timestamp;
        refresh_evidence(&mut value, &["merchant".to_owned()]);
        let parsed = parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        let result = validate(&parsed);
        if last == Some(created - 1) {
            let error = result.unwrap_err();
            assert_eq!(error.code, "merchant_timestamp_order");
            assert!(error.path.ends_with("/lastConversion"));
        } else {
            result.unwrap();
        }
    }
}

#[test]
fn approved_reserve_depth_uses_whole_token_units() {
    let input = input();
    let expected = golden();
    assert_eq!(
        input.bootstrap.d9_reserve_floor.0,
        600_000 * 1_000_000_000_000u128
    );
    assert_eq!(
        input.bootstrap.usdt_reserve_floor.0,
        600_000 * 1_000_000u128
    );
    assert_eq!(
        serde_json::to_value(input.bootstrap.d9_reserve_floor).unwrap(),
        expected["d9ReserveFloor"]
    );
    assert_eq!(
        serde_json::to_value(input.bootstrap.usdt_reserve_floor).unwrap(),
        expected["usdtReserveFloor"]
    );
    validate(&input).unwrap();
}

/// Every literal `fail("code", ...)` in the RC7 validation modules must have a
/// shared negative case. Scans source text so a new rejection cannot be added
/// without a corpus entry.
#[test]
fn rc7_rejection_codes_in_source_each_have_a_shared_case() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("../fixtures/cases.json")).unwrap();
    let mut codes = BTreeSet::new();
    for source in [
        include_str!("../src/validation/custody.rs"),
        include_str!("../src/validation/network.rs"),
        include_str!("../src/validation/composition.rs"),
        include_str!("../src/validation/pop.rs"),
    ] {
        for (offset, _) in source.match_indices("fail(") {
            let rest = source[offset + "fail(".len()..].trim_start();
            if let Some(literal) = rest.strip_prefix('"') {
                codes.insert(literal.split('"').next().unwrap().to_owned());
            }
        }
    }
    for code in [
        "multisig_nested_signatory",
        "sudo_signatory_not_independent",
        "rehome_destination_not_allowed",
        "custody_pop_invalid",
        "custody_pop_scheme",
        "custody_pop_attestation_malformed",
        "custody_pop_attestation_binding",
        "multisig_address_not_derived",
        "authority_role_not_distinct",
        "authority_session_key",
        "authority_pallet_account",
        "authority_validator_account",
        "dev_key_authority",
        "chain_identity_label",
        "asset_id_set",
    ] {
        assert!(codes.contains(code), "scanner missed {code}");
    }
    for code in &codes {
        assert!(
            cases
                .iter()
                .any(|case| case["expected"]["code"] == code.as_str()),
            "{code} has no shared negative case"
        );
    }
}

fn account(address: &Address) -> [u8; 32] {
    *AsRef::<[u8; 32]>::as_ref(&address.account_id())
}

/// Fresh sr25519 test custodians. `Pair::generate` draws the seed from `OsRng`
/// (sp-core 43.0.0 `src/crypto.rs:851-855`); the seed is dropped immediately and
/// the pair lives only in this test's memory. Nothing is written.
fn fresh_keys(n: usize) -> Vec<sp_core::sr25519::Pair> {
    use sp_core::Pair;
    (0..n)
        .map(|_| sp_core::sr25519::Pair::generate().0)
        .collect()
}

fn nonce(input: &ContractInput) -> [u8; 32] {
    let hex = input.custody.ceremony_nonce.as_str();
    std::array::from_fn(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap())
}

/// A PoP-valid `threshold`-of-n multisig for `role` under `input`'s chain and nonce.
fn issue(
    input: &ContractInput,
    role: &str,
    threshold: u16,
    keys: &[sp_core::sr25519::Pair],
    form: PopMessageForm,
) -> MultisigAuthority {
    use sp_core::Pair;
    let mut keys: Vec<&sp_core::sr25519::Pair> = keys.iter().collect();
    keys.sort_by_key(|key| key.public().0);
    let publics: Vec<[u8; 32]> = keys.iter().map(|key| key.public().0).collect();
    let address = multisig_account(&publics, threshold);
    let signatories = keys
        .iter()
        .map(|key| {
            let message = custody_pop_message(
                input.chain.network,
                &input.chain.id,
                role,
                &address,
                &key.public().0,
                &nonce(input),
            );
            let signature = key.sign(&custody_pop_payload(form, &message)).0;
            Signatory {
                address: Address::from_account_id(key.public().0.into()),
                evidence: PossessionEvidence::Signature(SignatureEvidence {
                    scheme: SignatureScheme::Sr25519,
                    message: form,
                    signature: SignatureHex::try_from(
                        signature
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>(),
                    )
                    .unwrap(),
                }),
            }
        })
        .collect();
    MultisigAuthority {
        address: Address::from_account_id(address.into()),
        threshold,
        signatories,
    }
}

/// Replace every custody authority with fresh 2-of-3 keys valid for `input` as it is.
fn reissue_custody(input: &mut ContractInput) {
    input.bootstrap.sudo = issue(input, "sudo", 2, &fresh_keys(3), PopMessageForm::Raw);
    input.bootstrap.usdt_owner = issue(input, "usdtOwner", 2, &fresh_keys(3), PopMessageForm::Raw);
    for index in 0..input.bootstrap.admins.len() {
        let role = format!("admin/{}", input.bootstrap.admins[index].pallet);
        input.bootstrap.admins[index].multisig =
            issue(input, &role, 2, &fresh_keys(3), PopMessageForm::Raw);
    }
}

#[test]
fn multisig_bounds_are_inclusive() {
    // threshold == n is valid (n-of-n).
    let mut input = input();
    input.bootstrap.sudo = issue(&input, "sudo", 3, &fresh_keys(3), PopMessageForm::Raw);
    validate(&input).unwrap();
    // Exactly MULTISIG_MAX_SIGNATORIES is valid; one more is refused.
    let keys = fresh_keys(MULTISIG_MAX_SIGNATORIES + 1);
    input.bootstrap.sudo = issue(
        &input,
        "sudo",
        2,
        &keys[..MULTISIG_MAX_SIGNATORIES],
        PopMessageForm::Raw,
    );
    validate(&input).unwrap();
    input.bootstrap.sudo = issue(&input, "sudo", 2, &keys, PopMessageForm::Raw);
    assert_eq!(
        validate(&input).unwrap_err().code,
        "multisig_signatory_limit"
    );
}

#[test]
fn every_fixture_authority_address_is_its_multisig_account() {
    let input = input();
    let mut authorities = vec![&input.bootstrap.sudo, &input.bootstrap.usdt_owner];
    authorities.extend(input.bootstrap.admins.iter().map(|role| &role.multisig));
    assert_eq!(authorities.len(), 14);
    for authority in authorities {
        let keys: Vec<[u8; 32]> = authority
            .signatories
            .iter()
            .map(|signatory| account(&signatory.address))
            .collect();
        assert_eq!(
            multisig_account(&keys, authority.threshold),
            account(&authority.address),
            "{}",
            authority.address
        );
    }
    // Any change to threshold or signatories without re-deriving is refused.
    let mut changed = self::input();
    changed.bootstrap.usdt_owner.threshold = 3;
    let error = validate(&changed).unwrap_err();
    assert_eq!(error.code, "multisig_address_not_derived");
    assert_eq!(error.path, "/bootstrap/usdtOwner/address");
}

#[test]
fn admins_may_share_one_multisig_and_usdt_owner_may_equal_an_admin() {
    let mut input = input();
    let keys = fresh_keys(3);
    for index in 0..input.bootstrap.admins.len() {
        let role = format!("admin/{}", input.bootstrap.admins[index].pallet);
        input.bootstrap.admins[index].multisig =
            issue(&input, &role, 2, &keys, PopMessageForm::Raw);
    }
    let shared = &input.bootstrap.admins[0].multisig.address;
    assert!(input
        .bootstrap
        .admins
        .iter()
        .all(|role| &role.multisig.address == shared));
    validate(&input).unwrap();
    input.bootstrap.usdt_owner = issue(&input, "usdtOwner", 2, &keys, PopMessageForm::Raw);
    validate(&input).unwrap();
    // The same multisig needs a proof per role: admin 0's proofs do not cover admin 1.
    input.bootstrap.admins[1].multisig = input.bootstrap.admins[0].multisig.clone();
    let error = validate(&input).unwrap_err();
    assert_eq!(error.code, "custody_pop_invalid");
    assert!(error
        .path
        .starts_with("/bootstrap/admins/1/multisig/signatories/"));
}

#[test]
fn sudo_signatories_must_be_disjoint_from_usdt_owner_and_every_admin() {
    let shared = fresh_keys(1).remove(0);
    let others = fresh_keys(4);
    let mut input = input();
    let sudo_keys = [shared.clone(), others[0].clone(), others[1].clone()];
    input.bootstrap.sudo = issue(&input, "sudo", 2, &sudo_keys, PopMessageForm::Raw);
    validate(&input).unwrap();
    let admin_keys = [shared, others[2].clone(), others[3].clone()];
    input.bootstrap.usdt_owner = issue(&input, "usdtOwner", 2, &admin_keys, PopMessageForm::Raw);
    let error = validate(&input).unwrap_err();
    assert_eq!(error.code, "sudo_signatory_not_independent");
    assert!(error.path.starts_with("/bootstrap/sudo/signatories/"));
    assert!(error
        .related_path
        .as_deref()
        .unwrap()
        .starts_with("/bootstrap/usdtOwner/signatories/"));
    let mut input = self::input();
    input.bootstrap.sudo = issue(&input, "sudo", 2, &sudo_keys, PopMessageForm::Raw);
    input.bootstrap.admins[3].multisig = issue(
        &input,
        "admin/d9-governance",
        2,
        &admin_keys,
        PopMessageForm::Raw,
    );
    let error = validate(&input).unwrap_err();
    assert_eq!(error.code, "sudo_signatory_not_independent");
    assert!(error
        .related_path
        .as_deref()
        .unwrap()
        .starts_with("/bootstrap/admins/3/multisig/signatories/"));
}

#[test]
fn a_signatory_may_sit_in_several_admin_side_multisigs() {
    let shared = fresh_keys(1).remove(0);
    let mut input = input();
    let with_shared = |extra: Vec<sp_core::sr25519::Pair>| {
        let mut keys = extra;
        keys.push(shared.clone());
        keys
    };
    let usdt = with_shared(fresh_keys(2));
    let admin0 = with_shared(fresh_keys(2));
    let admin5 = with_shared(fresh_keys(2));
    input.bootstrap.usdt_owner = issue(&input, "usdtOwner", 2, &usdt, PopMessageForm::Raw);
    input.bootstrap.admins[0].multisig =
        issue(&input, "admin/d9-amm", 2, &admin0, PopMessageForm::Raw);
    input.bootstrap.admins[5].multisig =
        issue(&input, "admin/d9-merchant", 2, &admin5, PopMessageForm::Raw);
    validate(&input).unwrap();
}

#[test]
fn proofs_are_accepted_in_raw_and_bytes_wrapped_form_for_both_schemes() {
    let input = input();
    let mut forms = BTreeSet::new();
    for authority in std::iter::once(&input.bootstrap.sudo)
        .chain(std::iter::once(&input.bootstrap.usdt_owner))
        .chain(input.bootstrap.admins.iter().map(|role| &role.multisig))
    {
        for signatory in &authority.signatories {
            if let PossessionEvidence::Signature(evidence) = &signatory.evidence {
                forms.insert(format!("{:?}/{:?}", evidence.scheme, evidence.message));
            }
        }
    }
    for form in ["Sr25519/Raw", "Sr25519/BytesWrapped", "Ed25519/Raw"] {
        assert!(forms.contains(form), "fixture lacks {form}");
    }
    let mut input = input;
    input.bootstrap.sudo = issue(
        &input,
        "sudo",
        2,
        &fresh_keys(3),
        PopMessageForm::BytesWrapped,
    );
    validate(&input).unwrap();
}

#[test]
fn rehome_destinations_allow_only_the_mining_pool_and_the_amm() {
    let input = input();
    let destinations: BTreeSet<_> = input.changes.rehomes.iter().map(|r| r.to.clone()).collect();
    let allowed: BTreeSet<_> = [
        input.bootstrap.mining_pool_account.clone(),
        input.bootstrap.amm_account.clone(),
    ]
    .into_iter()
    .collect();
    assert_eq!(
        destinations, allowed,
        "fixture exercises both D9 destinations"
    );
    assert!(input
        .changes
        .asset_rehomes
        .iter()
        .all(|r| r.to == input.bootstrap.amm_account));
    validate(&input).unwrap();
}

#[test]
fn every_well_known_development_key_is_refused_as_signatory() {
    let alice = sp_core::sr25519::Pair::from_string("//Alice", None).unwrap();
    let ferdie_stash = sp_core::ed25519::Pair::from_string("//Ferdie//stash", None).unwrap();
    use sp_core::Pair;
    for key in [alice.public().0, ferdie_stash.public().0] {
        assert!(well_known_development_key(&key).is_some());
        let mut input = input();
        let signatories = &mut input.bootstrap.usdt_owner.signatories;
        signatories[2].address = Address::from_account_id(key.into());
        signatories.sort_by_key(|signatory| account(&signatory.address));
        assert_eq!(validate(&input).unwrap_err().code, "dev_key_authority");
    }
    assert!(well_known_development_key(&[0xa0; 32]).is_none());
}

#[test]
fn chain_identity_binds_purpose_without_blocking_testnet_rehearsal() {
    let mut input = input();
    assert_eq!(input.chain.network, Network::Testnet);
    // Χ rehearsal: real-data purpose on a testnet-labelled Live chain.
    input.purpose = Purpose::MigrationInput;
    validate(&input).unwrap();
    input.chain = ChainIdentity {
        network: Network::Mainnet,
        id: "d9_mainnet".into(),
        name: "D9 Mainnet".into(),
        chain_type: ChainType::Live,
    };
    // Testnet proofs do not carry over to mainnet; a mainnet ceremony does.
    assert_eq!(validate(&input).unwrap_err().code, "custody_pop_invalid");
    reissue_custody(&mut input);
    validate(&input).unwrap();
    input.purpose = Purpose::SyntheticFixture;
    assert_eq!(validate(&input).unwrap_err().code, "chain_purpose_binding");
}

#[test]
fn declared_asset_set_may_include_fresh_assets_but_not_omit_migrated_ones() {
    let input = input();
    let migrated: BTreeSet<u32> = input.state.assets.iter().map(|a| a.asset_id).collect();
    let declared: BTreeSet<u32> = input.bootstrap.asset_ids.iter().copied().collect();
    assert!(migrated.is_subset(&declared));
    assert!(declared.difference(&migrated).next().is_some());
    validate(&input).unwrap();
}

#[test]
fn version_is_reported_before_typed_decode_and_still_checked_by_validate() {
    let mut value = input_value();
    value["contractVersion"] = json!("d9-native-genesis/0.1.0-rc.6");
    value.as_object_mut().unwrap().remove("chain");
    let error = parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
    assert_eq!(
        error,
        ParseError::UnsupportedVersion {
            found: "d9-native-genesis/0.1.0-rc.6".into()
        }
    );
    assert!(
        error.to_string().starts_with("contract_version: "),
        "{error}"
    );
    let mut input = input();
    input.contract_version = "d9-native-genesis/0.1.0-rc.6".into();
    assert_eq!(validate(&input).unwrap_err().code, "contract_version");
}

#[test]
fn validator_accounts_and_every_session_key_refuse_development_keys() {
    use sp_core::Pair;
    let alice_sr = sp_core::sr25519::Pair::from_string("//Alice", None)
        .unwrap()
        .public()
        .0;
    let alice_ed = sp_core::ed25519::Pair::from_string("//Alice", None)
        .unwrap()
        .public()
        .0;
    let hex = |key: [u8; 32]| key.iter().map(|b| format!("{b:02x}")).collect::<String>();
    for (field, key) in [
        ("babe", alice_sr),
        ("grandpa", alice_ed),
        ("imOnline", alice_sr),
        ("discovery", alice_sr),
        ("liveness", alice_ed),
    ] {
        let mut value = input_value();
        value["bootstrap"]["validators"][1][field] = json!(hex(key));
        let error = validate(&parse(&serde_json::to_vec(&value).unwrap()).unwrap()).unwrap_err();
        assert_eq!(error.code, "dev_key_authority", "{field}");
        assert_eq!(error.path, format!("/bootstrap/validators/1/{field}"));
    }
}

/// A synthetic document stands in for a Nitro attestation. It passes only the
/// contract-side checks (structure and message binding); it is not a real
/// attestation and the producer's `attest_verify` would refuse it.
fn synthetic_enclave_sudo_signatory(input: &ContractInput, index: usize) -> Signatory {
    let authority = &input.bootstrap.sudo;
    let signatory = &authority.signatories[index];
    let digest = custody_pop_message_sha256(
        input.chain.network,
        &input.chain.id,
        "sudo",
        &account(&authority.address),
        &account(&signatory.address),
        &nonce(input),
    );
    Signatory {
        address: signatory.address.clone(),
        evidence: PossessionEvidence::EnclaveAttested(EnclaveAttestation {
            // base64("synthetic-not-a-real-attestation")
            attestation_document: "c3ludGhldGljLW5vdC1hLXJlYWwtYXR0ZXN0YXRpb24=".into(),
            expected_pcr0: Pcr0Hex::try_from("ab".repeat(48)).unwrap(),
            pop_message_sha256: Digest::try_from(
                digest
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>(),
            )
            .unwrap(),
        }),
    }
}

#[test]
fn synthetic_enclave_attestation_passes_only_the_contract_side_checks() {
    let mut input = input();
    input.bootstrap.sudo.signatories[0] = synthetic_enclave_sudo_signatory(&input, 0);
    let report = validate(&input).unwrap();
    assert!(!report.release_gate_evaluated);
    assert!(report
        .independent_evidence_required
        .iter()
        .any(|line| line.contains("attest_verify") && line.contains("pcr0-ledger")));
    // The binding covers role, chain and nonce like a signature does.
    let mut moved = input.clone();
    moved.custody.ceremony_nonce = CeremonyNonce::try_from("44".repeat(32)).unwrap();
    let error = validate(&moved).unwrap_err();
    assert_eq!(error.code, "custody_pop_attestation_binding");
    assert_eq!(
        error.path,
        "/bootstrap/sudo/signatories/0/evidence/enclaveAttested/popMessageSha256"
    );
    // Documents above MAX_ATTESTATION_DOCUMENT_BYTES are malformed; the bound itself is not.
    for (decoded, accepted) in [
        (MAX_ATTESTATION_DOCUMENT_BYTES / 3 * 3, true),
        ((MAX_ATTESTATION_DOCUMENT_BYTES / 3 + 1) * 3, false),
    ] {
        let mut sized = input.clone();
        let PossessionEvidence::EnclaveAttested(evidence) =
            &mut sized.bootstrap.sudo.signatories[0].evidence
        else {
            unreachable!()
        };
        evidence.attestation_document = "A".repeat(decoded / 3 * 4);
        match validate(&sized) {
            Ok(_) => assert!(accepted, "{decoded} bytes accepted"),
            Err(error) => {
                assert!(!accepted, "{decoded} bytes refused: {error:?}");
                assert_eq!(error.code, "custody_pop_attestation_malformed");
            }
        }
    }
}
