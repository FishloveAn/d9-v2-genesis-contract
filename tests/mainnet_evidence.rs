use d9_genesis_contract::Address;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Evidence {
    source: Source,
    positions: Vec<Position>,
    total_lp_tokens: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Source {
    endpoint: String,
    block_number: u64,
    block_hash: String,
    state_root: String,
    contract_code_hash: String,
    finalized: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Position {
    account: String,
    amount: String,
    storage_key: String,
    raw_value: String,
}

fn hex_bytes(value: &str) -> Vec<u8> {
    let value = value
        .strip_prefix("0x")
        .expect("evidence hex has 0x prefix");
    assert_eq!(value.len() % 2, 0, "hex must contain whole bytes");
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(text, 16).unwrap()
        })
        .collect()
}

fn little_u128(bytes: &[u8]) -> u128 {
    assert_eq!(bytes.len(), 16);
    let mut value = [0u8; 16];
    value.copy_from_slice(bytes);
    u128::from_le_bytes(value)
}

#[test]
fn mainnet_amm_receipt_preserves_all_lp_owners_and_values() {
    let evidence: Evidence =
        serde_json::from_str(include_str!("../evidence-amm-mainnet-23802000.json")).unwrap();

    assert_eq!(
        evidence.source.endpoint,
        "wss://archiver.d9network.com:40300"
    );
    assert_eq!(evidence.source.block_number, 23_802_000);
    assert_eq!(evidence.source.block_hash.len(), 64);
    assert_eq!(evidence.source.state_root.len(), 64);
    assert_eq!(evidence.source.contract_code_hash.len(), 64);
    assert!(evidence.source.finalized);
    assert_eq!(evidence.positions.len(), 10);

    let mut total = 0u128;
    for position in &evidence.positions {
        let address = Address::try_from(position.account.clone()).unwrap();
        let key = hex_bytes(&position.storage_key);
        assert_eq!(
            key.len(),
            52,
            "Mapping key must include hash, root and AccountId"
        );
        let account_id = address.account_id();
        let account_bytes: &[u8] = account_id.as_ref();
        assert_eq!(&key[20..], account_bytes);

        let raw = hex_bytes(&position.raw_value);
        assert_eq!(raw.len(), 17, "StorageData must be a SCALE Bytes wrapper");
        assert_eq!(raw[0], 0x40, "16-byte SCALE Bytes length prefix expected");
        let amount = little_u128(&raw[1..]);
        assert_eq!(amount.to_string(), position.amount);
        total = total.checked_add(amount).unwrap();
    }

    assert_eq!(total.to_string(), evidence.total_lp_tokens);
    assert_eq!(evidence.total_lp_tokens, "7332501");
}
