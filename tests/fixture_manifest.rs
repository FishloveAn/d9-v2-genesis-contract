use d9_genesis_contract::{bytes_digest, contract_digest, CONTRACT_VERSION};

#[test]
fn published_bundle_hashes_cover_the_exact_shared_artifacts() {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../fixture-manifest.json")).unwrap();
    assert_eq!(manifest["contractVersion"], CONTRACT_VERSION);
    assert_eq!(manifest["contractDigest"], contract_digest().as_str());
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../fixtures/cases.json")).unwrap();
    assert_eq!(manifest["negativeCases"], cases.len());
    for row in manifest["files"].as_array().unwrap() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(row["path"].as_str().unwrap());
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(row["size"], bytes.len());
        assert_eq!(
            row["sha256"],
            bytes_digest(&bytes).as_str(),
            "{}",
            row["path"]
        );
    }
}
