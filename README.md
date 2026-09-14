# D9 native-genesis contract

The shared contract for D9 V2 exporters, pallet genesis builders and independent
verifiers. Candidate wire version: **d9-native-genesis/0.1.0-rc.7**.

This repository is the authoritative home for the contract, schemas, provenance
inventory, rules and shared conformance fixtures extracted from D9-380 at pallets
commit `423900b882fbaaabf67f1eab84c3cae5a3a6e710`.

## Contents

| Artifact | Purpose |
|---|---|
| [CONTRACT.md](CONTRACT.md) | Field semantics, units, invariant ownership and encoding |
| [schema.json](schema.json) | Input schema generated from the Rust DTOs |
| [binding.schema.json](binding.schema.json) | Detached final-artifact binding |
| [inventory.json](inventory.json) | 137 provenance entries with explicit disposition references |
| [rules.json](rules.json) | 24 rules and their implementation owners |
| [fixtures/complete.json](fixtures/complete.json) | Complete synthetic input |
| [fixtures/expected.json](fixtures/expected.json) | Independently authored expected values and digests |
| [fixtures/cases.json](fixtures/cases.json) | 172 mutations and expected failures |
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

## RC2 exclusion revision

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

## RC7 multisig custody, chain identity and asset set

[D9-400](https://linear.app/d9-network/issue/D9-400) applies Yvan's 2026-09-14
ruling to enforce DEC-21 k-of-n `pallet_multisig` custody, with audit finding S-6
and review finding R-4. RC7 is not wire-compatible with RC6:

- `bootstrap.sudo`, the new `bootstrap.usdtOwner` and every `bootstrap.admins[].multisig`
  are `{address, threshold, signatories}`. The contract checks structure
  (2 <= threshold <= n <= 20, strictly ascending unique signatories, no self or
  nested-role signatory, no PalletId, validator-account, session-key or development
  key) and binds each address: `address == multisig_account(signatories, threshold)`,
  the pallet-multisig account. Sudo stays distinct from the USDT owner and every
  admin, and shares no signatory with them (yvan 2026-09-14 04:34 UTC); admin-side
  multisigs may still share signatories. Every signatory carries a
  proof-of-possession over `custody_pop_message` (network, chain id, role, multisig,
  signatory, `custody.ceremonyNonce`), as a `signature` or, for enclave-held keys, as
  `enclaveAttested` (a Nitro attestation whose `user_data` is
  `custody_pop_message_sha256`). Keyless accounts cannot be custodians and a proof
  cannot be replayed across networks, roles or ceremonies. The contract checks an
  attestation's structure and message binding only; the producer verifies the
  document with `d9-enclave-common` `attest_verify` against the PCR0 ledger. d9-v2-tools `derive_admins` keeps a second, independent copy of the
  derivation; both are pinned to the same polkadot-js golden vectors, and tools
  `d9-genesis-composition` `custody::cross_check_derivations` (tools PR #72, not yet
  merged) cross-checks them.
- Multisig custody applies to every purpose and ladder rung; there is no
  single-key rehearsal exception.
- Validator accounts and all five session keys also refuse development keys; validator
  accounts refuse PalletId accounts and session keys; session keys are unique across
  every slot. The deny list (183 keys) covers the sp-keyring URIs, the DEV_PHRASE
  roots and d9's committed authority SURIs (`//LocalValidator1..6`, `//Mainnet0..5//*`,
  `//OCWTest//*`), each in sr25519, ed25519 and the ecdsa-derived account.
- The new top-level `chain` declares `network`, `id`, `name` and `chainType`.
  `migration-input` requires `Live`; only `migration-input` may be labelled
  `mainnet`; a testnet-labelled rehearsal with real data remains valid. The producer
  must match its manifest and chain-spec metadata and reject boot nodes and telemetry.
- D9 rehomes may credit only `miningPoolAccount` or `ammAccount`; asset rehomes only
  `ammAccount`. Any other destination needs a new contract RC.
- `bootstrap.assetIds` declares the complete V2 asset ID set. The producer must
  require the asset definition and metadata ID sets to equal it exactly.

`parse` returns `Result<ContractInput, ParseError>`: a readable but different
`contractVersion` is `ParseError::UnsupportedVersion { found }` before typed decoding,
so an RC6 document reports its version rather than an unknown-field error; anything
else is `ParseError::Decode(message)`. `multisig_account`, `custody_pop_message`,
`custody_pop_message_sha256`, `custody_pop_payload`, `MULTISIG_MAX_SIGNATORIES` and `well_known_development_key`
are exported so producers and signers do not retype the derivation, the PoP message,
the bound or the deny list. `examples/custody_pop_fixture.rs` regenerates the
synthetic custody fixture from OS randomness without writing any seed. RC6 acknowledgements do not
cover RC7 bytes.
