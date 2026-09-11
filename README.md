# D9 native-genesis contract

The shared contract for D9 V2 exporters, pallet genesis builders and independent
verifiers. Candidate wire version: **d9-native-genesis/0.1.0-rc.1**.

This repository is the authoritative home for the contract, schemas, provenance
inventory, rules and shared conformance fixtures extracted from D9-380 at pallets
commit `423900b882fbaaabf67f1eab84c3cae5a3a6e710`.

## Contents

| Artifact | Purpose |
|---|---|
| [CONTRACT.md](CONTRACT.md) | Field semantics, units, invariant ownership and encoding |
| [schema.json](schema.json) | Input schema generated from the Rust DTOs |
| [binding.schema.json](binding.schema.json) | Detached final-artifact binding |
| [inventory.json](inventory.json) | 135 candidate provenance entries with explicit pending decisions |
| [rules.json](rules.json) | 18 rules and their implementation owners |
| [fixtures/complete.json](fixtures/complete.json) | Complete synthetic input |
| [fixtures/expected.json](fixtures/expected.json) | Independently authored expected values and digests |
| [fixtures/cases.json](fixtures/cases.json) | 56 mutations and expected failures |
| [fixture-manifest.json](fixture-manifest.json) | Exact hashes of the shared artifacts |
| [REVIEW.md](REVIEW.md) | Pending downstream compatibility acknowledgements |
| [VERIFICATION.md](VERIFICATION.md) | Validation scope and historical verification |

The Rust crate contains strict DTOs, scalar/address validation, input conformance,
canonical digests and final-byte binding checks. It has **no dependency on
`d9-core`, business pallets, or a sibling checkout**. Standard Substrate SDK
primitives remain dependencies for SS58 and PalletId derivation.

## Run locally

```sh
cargo run --locked -- check fixtures/complete.json
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --all -- --check
```

CI runs these checks on pushes and pull requests. Success means input-contract
conformance, and reports explicitly state `releaseGateEvaluated: false`.

Regenerate schemas after an intentional contract change:

```sh
cargo run --locked -q -- schema > schema.json
cargo run --locked -q -- binding-schema > binding.schema.json
```

## Consume from another repository

Pin a reviewed commit rather than following `main`:

```toml
[dependencies]
d9-genesis-contract = { git = "https://github.com/D-Nine-Chain/d9-v2-genesis-contract", rev = "<reviewed-full-commit>" }
```

For shared tests, use `d9_genesis_contract::fixtures::{COMPLETE, EXPECTED, CASES,
MANIFEST}`. Do not copy these files into each consumer repository. Schemas and
fixture files can also be consumed without using the Rust crate.

Pallet-specific `MerchantMiningAccount`/`BurnPortfolio` conversion, the original
`d9-core::validate_manifest` bridge and source-name coverage tests belong to the
`d9-genesis-adapter` crate in **d9-v2-pallets**. The shared crate does not implement
missing pallet genesis fields or write runtime storage. D9-370 composes all input
slices into the actual runtime configuration.

## Scope and review

All chain-like fixtures are synthetic. Historical LP values have synthetic
owners; they are not an authenticated V1 snapshot. Mock binding bytes are not a
usable chain spec or a provenance report. The extraction preserves the RC1 wire
schema, fixture bytes and contract digest; moving runtime adapters does not
constitute downstream approval.

The inventory validator here checks portable declaration shape. The pallet
adapter independently uses the accepted D9-307 API and checks its own source
coverage. Null provenance remains an unresolved decision, not a fourth variant.
Real migration inputs are refused while pending classifications remain.

D9-380 still requires downstream acknowledgement. D9-307 owns complete provenance
checks, D9-173 owns independent final-state reconciliation, D9-370 owns composition,
and D9-315 owns reproducible WASM. See [D9-380](https://linear.app/d9-network/issue/D9-380).
