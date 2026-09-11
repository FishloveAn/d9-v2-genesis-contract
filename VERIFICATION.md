# Historical pallet-workspace verification — 2026-09-10

Toolchain: rustc 1.98.1 (48a229cea, 2026-09-01), local macOS host.
Baseline: pallets aec116756eaf21fb455c7da068a91615b0ae700b.
The recovered D9-307 patch was verified byte-for-byte against its published
SHA-256 before application; it is a separate prerequisite commit.

| Command | Result |
|---|---|
| cargo fmt --all -- --check | exit 0 |
| cargo check --locked --workspace --all-features | exit 0 |
| cargo clippy --locked --workspace --all-targets -- -D warnings | exit 0 |
| cargo test --locked --workspace | exit 0; 757 passed |
| cargo test --locked -p pallet-d9-merchant --features runtime-benchmarks -- benchmarks | exit 0; 5 passed |
| cargo deny check | exit 0 |
| git diff --check | exit 0 |

The workspace count includes 77 d9-core tests and 13 contract tests. Contract
tests execute all 56 negative corpus entries, native domain-type roundtrips,
source-name inventory coverage, generated schema/manifest drift checks, full
u128 roundtrips, canonical digests and final-byte binding tamper cases.

Original command output and actual process exit codes are retained in the
handoff archive under verification/. Output was captured without a pipeline
that could hide Cargo's exit code. A pre-existing trie-db future-incompatibility
notice is printed by Cargo; it did not fail these checks.

Documentation and bundle-digest finalization are followed by another focused
contract test run. No remote CI result, nativegen build, runtime WASM build,
two-machine reproduction, live-chain readback, no_std qualification or downstream
review is claimed by this record. The host-side contract is not a release gate.

## Standalone extraction — 2026-09-11

The 757/5 counts above describe the original pallet workspace, not this smaller
repository. Runtime type roundtrips and live pallet source-name coverage now
belong to d9-genesis-adapter in d9-v2-pallets. Standalone tests retain all 56
negative cases, golden amounts/digests, schema/manifest drift, address checks
and artifact binding tests. No runtime or business-pallet dependency is needed.
See CI for verification of the standalone commit.
