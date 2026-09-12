# RC3 review dispositions — 2026-09-12

This records the assessment of the remaining comments on
[contract PR #4](https://github.com/D-Nine-Chain/d9-v2-genesis-contract/pull/4).
It is not a downstream compatibility acknowledgement or release approval.
RC3 source `d629d028dab2a8f154c57eda197ed7e1cf9d56bf`, its Rust behavior,
rules, inventory, schema and fixture bytes are preserved by this documentation
follow-up. The contract digest remains
`d0f06baa97856920c43ac871736e5cfc4b675af82a84f46103d78db718098c6a`.

## Lock condition wording

[Comment 3994642250](https://github.com/D-Nine-Chain/d9-v2-genesis-contract/pull/4#discussion_r3994642250)
correctly identifies shorthand that deserves clarification. In the input
validator, `v1LockAmount` must equal the decimal string `"1"`, decoded as integer
1. The approved account, raw source-lock row, matching block, declared account
absence, absent source/target balance records and unique exclusion are also
required. The validator does not read a `Balances.Locks` marker from the chain.
The separate source verification must authenticate the unique `council/` marker
and its amount at the same block.

CONTRACT.md now explains this distinction beside the existing field definition.
The suggested substitution inside `rules.json` and the serialized diagnostic is
not adopted for RC3: it would change already acknowledged contract bytes for a
wording clarification. This is an alternative documentation fix, not a claim
that the serialized wording changed or that the raw marker was authenticated.

## Unknown non-null inventory decisions

[Comment 3994751938](https://github.com/D-Nine-Chain/d9-v2-genesis-contract/pull/4#discussion_r3994751938)
is correct about the limit of the shape check: a non-empty decision in a modified
source inventory is not authenticated by the checker. The former verification
claim that all unknown future decisions are refused was too broad and is
corrected in VERIFICATION.md and README.md.

The proposed five-decision runtime allowlist is not adopted:

- `inventory()` reads `include_str!("../inventory.json")`. `validate()` obtains
  this compiled inventory; a producer cannot supply an alternative inventory
  in `ContractInput`. Unknown JSON fields are refused during decode.
- The five September 12 rulings are only part of the existing NotMigrated
  entries (118 of the 136 inventory rows are NotMigrated). Other entries
  include established authority and operational-state
  classifications. Restricting every NotMigrated row to those five would
  reject the accepted inventory.
- The private `require_resolved_inventory` helper refuses null provenance;
  non-empty `changes.unresolved` is also refused. Those are the executable
  refusal guarantees. A new non-null classification is a source/contract
  change requiring review, not a runtime proof of approval.
- `contract_digest()` commits to the complete inventory and rules.
  `verify_binding()` rejects a mismatched contract digest. Consumers must use
  an independently accepted revision/digest; a producer's self-generated
  digest or receipt does not establish authenticity. Source-code changes
  require revision review even when schema and fixture bytes are unchanged.

[Comment 3995638592](https://github.com/D-Nine-Chain/d9-v2-genesis-contract/pull/4#discussion_r3995638592)
asks the README to assert the broader guarantee again. That wording is not
adopted because it would describe behavior the checker does not implement.
The documentation now explicitly names this limit. If a separately approved
requirement introduces runtime-supplied inventories, that would need a
versioned interface and validation design rather than a README assertion.

## Scope of verification

The existing suite covers null-classification refusal, explicit unresolved
input, generated schemas, the shared rejection corpus and detached binding
tampering. No new runtime acceptance rule or fixture is introduced by this
documentation change. All files listed in fixture-manifest.json must still
match their recorded hashes and sizes. D9-173's compatibility acknowledgement,
source authentication, final composition and release verification remain open.

Local verification for this documentation follow-up: `cargo test --locked`
passed all 15 tests, including the 68 negative corpus cases. All ten manifest
entries matched their recorded file sizes and SHA-256 hashes. An exact Git
comparison with the acknowledged RC3 source confirmed that Rust code/tests,
Cargo files, schemas, rules, inventory and fixtures were unchanged.

Four CLI probes used temporary copies of the shared synthetic fixture with
`purpose = "migration-input"` (no live chain access):

| Input | Result |
|---|---|
| No other modification | Exit 0; `releaseGateEvaluated = false` |
| Add a top-level `inventory` with an unapproved NotMigrated row | Exit 1; decode rejects unknown field `inventory` |
| Add an entry to `changes.unresolved` | Exit 1; `unresolved_disposition` |
| Set the approved exclusion's `v1LockAmount` to `"2"` | Exit 1; `invalid_lock_exclusion` |

These probes confirm input refusals; they do not prove an approval check for
arbitrary non-null text in a future modified build.
