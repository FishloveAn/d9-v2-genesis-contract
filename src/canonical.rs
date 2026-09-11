use crate::Digest;
use serde::Serialize;
use sha2::{Digest as _, Sha256};

fn hash(bytes: &[u8]) -> Digest {
    Digest(format!("{:x}", Sha256::digest(bytes)))
}

// Explicit sorting also works when a downstream workspace enables serde_json's
// preserve_order feature. Cargo feature unification must not change a digest.
fn canonical(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let ordered: std::collections::BTreeMap<_, _> = map
                .into_iter()
                .map(|(key, value)| (key, canonical(value)))
                .collect();
            serde_json::Value::Object(ordered.into_iter().collect())
        }
        serde_json::Value::Array(rows) => {
            serde_json::Value::Array(rows.into_iter().map(canonical).collect())
        }
        other => other,
    }
}

/// Objects use serde_json's sorted-key map, arrays retain order, compact UTF-8,
/// no trailing newline. All financial integers are already decimal strings.
pub fn input_digest(value: &impl Serialize) -> Digest {
    let value = canonical(serde_json::to_value(value).expect("contract types serialize"));
    let mut bytes = b"d9-genesis-input-v1\0".to_vec();
    bytes.extend(serde_json::to_vec(&value).expect("JSON values serialize"));
    hash(&bytes)
}

/// Order-independent dataset commitment. Sort complete compact canonical JSON
/// row bytes lexicographically, then hash domain + dataset UTF-8 + NUL +
/// (u64 big-endian byte length, row bytes)*. Duplicates are retained; validation
/// must reject them. A row's fields, not only its amount, are committed.
pub fn dataset_digest<T: Serialize>(dataset: &str, rows: &[T]) -> Digest {
    let mut rows: Vec<Vec<u8>> = rows
        .iter()
        .map(|r| {
            serde_json::to_vec(&canonical(serde_json::to_value(r).expect("row serializes")))
                .expect("JSON serializes")
        })
        .collect();
    rows.sort();
    let mut bytes = b"d9-genesis-dataset-v1\0".to_vec();
    bytes.extend(dataset.as_bytes());
    bytes.push(0);
    for row in rows {
        bytes.extend((row.len() as u64).to_be_bytes());
        bytes.extend(row);
    }
    hash(&bytes)
}
