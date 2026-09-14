# D9 native-genesis contract

The shared contract for D9 V2 exporters, pallet genesis builders and independent
verifiers. Candidate wire version: **d9-native-genesis/0.1.0-rc.6**.

This repository is the authoritative home for the contract, schemas, provenance
inventory, rules and shared conformance fixtures extracted from D9-380 at pallets
commit `423900b882fbaaabf67f1eab84c3cae5a3a6e710`.

## Contents

| Artifact | Purpose |
|---|---|
| [CONTRACT.md](CONTRACT.md) | Field semantics, units, invariant ownership and encoding |
| [schema.json](schema.json) | Input schema generated from the Rust DTOs |
| [binding.schema.json](binding.schema.json) | Detached final-artifact binding |
| [inventory.json](inventory.json) | 140 provenance entries with explicit disposition references |
| [rules.json](rules.json) | 19 rules and their implementation owners |
| [fixtures/complete.json](fixtures/complete.json) | Complete synthetic input |
| [fixtures/expected.json](fixtures/expected.json) | Independently authored expected values and digests |
| [fixtures/cases.json](fixtures/cases.json) | 80 mutations and expected failures |
| [fixture-manifest.json](fixture-manifest.json) | Exact hashes of the shared artifacts |
| [evidence-amm-mainnet-23802000.json](evidence-amm-mainnet-23802000.json) | Read-only V1 mainnet AMM LP extraction receipt at block 23,802,000 |
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

The conformance fixtures remain synthetic. `evidence-amm-mainnet-23802000.json`
is a separate, read-only V1 mainnet extraction receipt and is not part of the RC1
golden fixture manifest. It records the pinned block, contract code hash,
child-trie identifiers, raw entries, decoded LP positions and the checked source
total. It is evidence for the D9-184 downstream export; it is not a complete
migration input or a release approval. Mock binding bytes are not a usable chain
spec or a provenance report. The extraction preserves the RC1 wire schema,
fixture bytes and contract digest; moving runtime adapters does not constitute
downstream approval.

The inventory validator here checks portable declaration shape. The pallet
adapter independently uses the accepted D9-307 API and checks its own source
coverage. Null provenance remains an unresolved decision, not a fourth variant.
Real migration inputs are refused while pending classifications remain.

D9-380 still requires downstream acknowledgement. D9-307 owns complete provenance
checks, D9-173 owns independent final-state reconciliation, D9-370 owns composition,
and D9-315 owns reproducible WASM. See [D9-380](https://linear.app/d9-network/issue/D9-380).

## Historical RC2 exclusion revision (superseded by D9-195)

RC2 applies the settled D9-195 / DEC-20 dangling-lock rule with a required
`changes.excludedJudicialLocks` array, pinned declared absence evidence and
source-minus-exclusion equality. Raw source rows remain inclusive. The complete
synthetic fixture now exercises the approved address; it does not authenticate
historical absence. RC3 subsequently records the five accepted dispositions
described below. JurorBallots adds current pallet inventory
coverage as fresh V2 operational state. For the historical RC2 revision, RC1
consumer acknowledgements required reruns against RC2 artifacts. Current
compatibility evidence must instead refer to RC3, as recorded in REVIEW.md.

## RC3 accepted dispositions

Yvan's five September 12 migration ADRs classify Resolutions, ProposalFeeVolume,
UserNonce, CumulativeBridgedOut and PendingOutbound as NotMigrated. V2 starts
those maps/counters empty/zero. Resolution history requires a verified external
archive; V1 obligations require separate verified settlement or an approved funded
arrangement; live V2 processing stays separate from archived V1 work. Equal
archived transfer-ID bytes do not impose a mandatory hash change. The proposal-fee
watermark and initial redemption band must match the accepted fresh values.

`migration-input` may pass input conformance now that the five classifications
are settled. Reports still say `releaseGateEvaluated: false` and name all remaining
archive, settlement, processing-boundary and final-state checks. A future null
inventory classification or an entry in `changes.unresolved` fails closed.
The inventory is compiled from this repository, not supplied by an input
producer; its decision text is part of the reviewed contract digest. The shape
validator does not authenticate decisions or approve source changes. Any new
classification requires contract review and a new digest. The contract does not read the external runtime
config or authenticate its archive/settlement evidence. RC1/RC2 acknowledgements
cannot be carried across the new contract and fixture digests.

The checker does **not** reject arbitrary non-empty decision text added by a
future source-code change merely because that text has no approval. Such a
change modifies the compiled contract and its digest, not a caller-controlled
input field. Downstream consumers must pin the reviewed revision and verify
the accepted digest; the digest itself does not authenticate approval.
See [RC3 review dispositions](review/rc3-comments.md) for the comment-by-comment
assessment and the distinction between input validation and source review.

## RC4 reserve-depth policy (historical; superseded by RC5)

Yvan approved [D9-190](https://linear.app/d9-network/issue/D9-190) minimum reserves
of 1M whole D9 and 1M whole USDT on
2026-09-12: 10^18 D9 raw and 10^12 USDT raw. This deliberately replaces V1
parity minima; it does not rebase source reserves or LP positions. The retired
redemption price floor remains disabled. Liquidity-depth protection is not an
oracle-security guarantee. RC3 dispositions remain unchanged; RC3 acceptance
does not automatically cover RC4 bytes. Review the new policy and artifact hashes.

## RC5 fixed reserve minima

Yvan superseded RC4 on 2026-09-12 with fixed minima of 600,000 whole D9
(600000000000000000 raw) and 600,000 whole USDT (600000000000 raw).
No dynamic floor formula is introduced. Source balances and every LP position
remain unchanged; the same quote/swap/withdrawal boundary semantics apply.
Prior versions and the retired 1M-token policy are rejected. RC4 remains a
published historical artifact; its acknowledgements do not imply RC5 approval.

## D9-204 session-ranking inventory extension

[D9-204](https://linear.app/d9-network/issue/D9-204/align-v2-validator-election-and-reward-eligibility-zero-vote-operators)
adds `d9-node-registry::SessionRanking` as fresh V2 session-planning state. The
inventory classifies it as `NotMigrated`: it has no V1 value or genesis field to
import, and the runtime derives it from eligible operators when planning a
session. The item remains marked `proposed` until the corresponding pallets
change is merged.

This classification changes the reviewed inventory and contract digest without
changing the RC6 wire schema, complete fixture or input digest. Prior RC6 bundle
acknowledgements do not automatically cover the extended inventory.

## D9-195 amendment, 2026-09-14

The declared judicial exclusion array must now be empty, and state locks must
match every source lock, including accounts without native balance. Final
composition classifies the genesis rows as Funded or RecordOnly; the pallet
checks actual provider state. No OnNewAccount hook is required. This retains
the RC6 source DTO shape but changes the rule-bundle digest, so old bindings
must be regenerated. The complete synthetic fixture contains both records.

Source authenticity and runtime debit enforcement remain separate acceptance
gates; this conformance checker does not approve a cutover.

## D9-383 governance voter-snapshot inventory extension

[D9-383](https://linear.app/d9-network/issue/D9-383/governance-cast-vote-has-no-voter-set-snapshot-a-proposal-spanning-a-session-rotation-mixes-validator-generations-and-re-bases-its-own-threshold)
adds `d9-governance::{VotingBodies, SnapshotVotes, LegacyProposals}`. All three
are `NotMigrated`: a V2 genesis has no V1 proposal history, no reconstructable
V1 voter body, and no V1 ballot data that can be safely reused as V2 votes.
`LegacyProposals` is populated only by the pallet's lazy post-upgrade handling;
V1 ballots remain audit-only. The entries remain `proposed` until the
corresponding pallets change is merged.

This changes the reviewed inventory and contract digest without changing the
RC6 wire schema, fixture payloads (`complete`, `expected`, and `cases`), or
input digest. `fixtures/binding.json` and its manifest hash are regenerated to
bind the new contract digest. Prior RC6 bundle acknowledgements do not
automatically cover the extended inventory.
