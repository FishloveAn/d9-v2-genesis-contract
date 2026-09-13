# Historical pallet-workspace verification — 2026-09-10

## D9-204 SessionRanking inventory extension — 2026-09-13

The inventory now contains 137 unique pallet-storage rows. The added
`d9-node-rewards::SessionRanking` row is `NotMigrated`, has no genesis field,
and is marked `proposed` pending the pallets implementation merge. The contract
digest is
`9b41f0cda8ad25b2b07db60dd0053078d88ab44cc5d4c717a87fcada7602ab72`;
the fixture binding and manifest hashes were regenerated from the changed
inventory bytes.

`cargo test --locked --workspace` passed all 17 tests, including the 80-case
rejection corpus, inventory validation, exact artifact hashes and detached
binding checks. `cargo run --locked -q -- check fixtures/complete.json` returned
the same contract digest and retained input digest
`5eb35bf564b4982e5e4243ab95f300315b9590c2fc795221d2799ec72b328a55`.
Formatting passed. No schema, rule, source fixture, source acceptance, runtime
composition or release gate changed.

Current local RC6 correction (2026-09-13): all 17 standalone tests passed,
including 80 negative cases and exact schema/fixture/binding digest checks.
Before the fix, the new one-millisecond-before-creation regression failed
because RC5 accepted it. None, equality and later conversion timestamps pass.
The paired merchant genesis change passes all 103 merchant tests, including
rejection before storage writes. No real export, integrated runtime or remote
CI result is claimed by these checks.

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

## Mainnet extraction evidence

`evidence-amm-mainnet-23802000.json` records a read-only extraction from
`wss://archiver.d9network.com:40300` at finalized V1 block `23,802,000`. The
receipt binds the block hash, state root, AMM contract address and code hash to
the child trie. Each LP row retains its raw child-trie key and `StorageData`
bytes so an independent reader can reproduce the decoding: remove the SCALE
`Bytes` length prefix (`0x40` for a 16-byte payload), then decode the remaining
little-endian `u128`.

The receipt contains ten LP positions and their checked sum `7,332,501`. This
is a source evidence artifact for D9-184, separate from the synthetic RC1
conformance fixtures. It does not claim to be the complete migration input or
to represent the eventual cutover pin.

## RC2 X1 alignment — 2026-09-12

The settled D9-195 / DEC-20 rule is implemented with explicit same-pin absence
declarations and exact source-minus-exclusion reconciliation. The synthetic
fixture contains two raw locks, one declared approved exclusion and one seeded
freeze; it is not raw V1 proof. A funded future-pin instance is retained without
an exclusion. The corpus now contains 68 negative cases. All 13 standalone tests
passed, including every corpus case, schema/bundle binding and historical AMM
receipt checks. Clippy all-targets -D warnings, formatting and diff checks passed.

JurorBallots is a V2 open-ballot reference count, updated by snapshot/take of
VotingBodies and backfilled from that map during runtime upgrade (pallets main
24a2f97, judicial lib.rs). Its NotMigrated classification follows the existing
VotingBodies/Referendums fresh operational-state rule. Inventory count is 136;
the five null provenance decisions are unchanged. No downstream acknowledgement,
source authentication, nativegen run, reproduction or release gate is claimed.

## RC3 five accepted dispositions — 2026-09-12

All five September 12 ADR classifications are now NotMigrated with full decision
links. The 136-entry inventory has no pending classifications. Future null
inventory classifications and explicit unresolved input still refuse; the old RC2
version is rejected. Migration-input conformance can now succeed, while reports
explicitly retain releaseGateEvaluated=false and require independent archive,
legacy settlement, clean processing-boundary and fresh-state/watermark evidence.
No runtime config, final spec, archive or settlement evidence was authenticated.

15 standalone tests passed, including all 68 negative corpus entries, manifest
and binding hash checks, future-null-classification refusal and conformance-not-release
coverage. Clippy all-targets -D warnings, formatting and diff checks passed.
The input schema is byte-identical to RC2 because the DTO fields are unchanged;
version, inventory, rules, input and binding digests were regenerated coherently.

The earlier phrase "unknown future inventory decisions" overstated the test:
`any_future_unresolved_inventory_still_blocks_migration_input` supplies null
provenance. It does not test or enforce an approval allowlist for non-empty
decision text in a modified build. Inventory is compiled from repository source;
source review, revision pinning and comparison with an accepted contract digest
remain necessary. See [the review dispositions](review/rc3-comments.md).

## RC4 reserve-depth consistency — 2026-09-12

Standalone cargo test: 16 passed, including the 75-case rejection corpus.
Clippy all-targets with -D warnings, formatting and diff checks passed. Added
independent whole-token golden assertions and rejections for RC3, each retired
parity floor, and one raw unit above/below each approved policy value. Source,
target state, changes/exception declarations, inventory and historical AMM
evidence remain unchanged. Only launch-policy floor literals and contract version
change the complete fixture. Schema regeneration produces identical DTO schemas.

This record does not claim downstream acceptance, runtime floor enforcement,
oracle security, performance weights or release-gate completion.

## RC5 fixed 600k reserve minima — 2026-09-12

The user superseded RC4's 1M-token minima with fixed 600,000-token minima:
600000000000000000 raw D9 and 600000000000 raw USDT. Standalone cargo test:
16 passed, including 78 rejection cases. RC4 version and both retired 1M policy
values are rejected; boundary cases now target the accepted 600k values.
Clippy all-targets -D warnings, formatting and diff checks passed. Source,
state, changes/exception declarations and inventory are unchanged. Historical
RC4 artifacts and verification remain identified separately above. No runtime
integration, downstream acknowledgement or release approval is claimed here.
