# D9-380 review handoff

## RC7 multisig custody, chain identity and asset set — 2026-09-14

[D9-400](https://linear.app/d9-network/issue/D9-400) records Yvan's 02:28 UTC
ruling: enforce DEC-21 k-of-n `pallet_multisig` custody through a new contract RC,
initially validating structure only; Yvan's 03:33 UTC rulings then bound each
address to its signatories in the contract (CS-1 = A), required multisig custody on
every purpose and ladder rung, and made sudo distinct from other roles (CS-4). S-6 (chain identity not bound to purpose) and R-4
(extra asset IDs pass) are included.

Contract digest: `4e611d12efaae24dd5bf4e1bcf87ececd081c07e2dd707a67b725a7c04e85f60`. Input digest: `f21a9e88d27aa7d7e9b31aee002d1e854949f91cf00159a8946feb947cb75f87`.
Rejection corpus: 134 cases. Exact SHA-256 pins:
- complete fixture: `e7a2a7e7d88b0d2d9caddead7ac3e3d6e4c963e31d088d753f463270e368f7aa`
- expected: `b6b951556f652191f8abaea43735a8963a9eb654b765ca7fc0929ca224a2ade3`
- cases: `ee129c5d596be84b31ac888d7bf0d464427f017ae158fc55d420c2b48e55121b`
- rules: `7ffd689c9c6f6c6ccbf0a23966b230dc9821419a357c3d2673e4cd31c57541df`
- schema: `c29a9f5e9c2865bf4d7f278822722992babc7ec79d2239de8c1413450ffb7968`
- inventory: `20f1bfd0323ecd4c8209fc7ea029bea8e8dc4273721a48a63df46ba46f47da9a`

Review points:
- The synthetic fixture's 14 multisig addresses are true `pallet_multisig`
  derivations of deterministic 2-of-3 filled-byte signatories (0xa0..0xc9),
  computed with d9-v2-tools `d9-bootstrap derive-admins` at tools `a5938f7` and
  independently recomputed in Python. The contract now re-derives and requires
  them, and a test asserts all 14.
- The development-key deny list applies to all purposes; its constants are
  re-derived from seed URIs with sp-core in a unit test.
- Chain label checks are consistency checks, not proof of deployment network.
  No mainnet chain id is pinned because none has been ruled.
- Asset definitions and metadata remain outside the DTO; exact ID-set equality is
  a producer obligation (rule ASSET_SET).

Revision after PR #10 code review and security audit (same day):
- CS-2: validator accounts and all five session keys refuse development keys.
- CS-3: no signatory may be another role's address; no address or signatory may be
  ammAccount, miningPoolAccount, any `b"modl"` PalletId account or a validator
  account. Multisigs nested outside the document are ceremony evidence.
- CS-5/CR-1: four producer obligations added to `independentEvidenceRequired`;
  `MultisigAuthority` documents that structure checks do not bind its address.
- CS-6: chain names are printable ASCII; tokens stay substring matches.
- CS-7: the deny list adds the bare DEV_PHRASE roots and the ecdsa-derived accounts
  (45 keys, each re-derived with sp-core/sp-runtime in a unit test).
- CR-2: every `chain_identity_label` branch has a case.
- CR-3: the `Assets.Asset` inventory decision now names the contract-bound owners,
  which changes the contract digest; CONTRACT.md corrects the owner binding and
  documents `threshold` as u16.
- CR-4: `parse` reports an unsupported `contractVersion` before typed decoding;
  main's RC6 `complete.json` reproduces exactly as case
  `rc6-complete-document-reports-version`.

Rulings applied (Yvan 2026-09-14 03:33 UTC):
- CS-1 = A: public `multisig_account(signatories, threshold)` (pallet-multisig
  48.0.0 `multi_account_id`), enforced as `multisig_address_not_derived`. It is
  intentionally the second implementation: d9-bootstrap keeps its own copy so the
  key-generation binary does not depend on sp-core; both are pinned to the same
  `@polkadot/util-crypto` vectors, and composition cross-checks them.
- Multisig custody applies to every purpose and ladder rung (Keel, Χ, Genie, Ψ, Ω);
  there is no single-key rehearsal exception.
- CS-4: sudo differs from the USDT owner and every admin
  (`authority_role_not_distinct`); admins may share one multisig, the USDT owner may
  equal an admin, and signatories may repeat across multisigs (positive tests).

No downstream acknowledgement exists for RC7. Tools, node and the pallets adapter
must repin and adopt the new fields before integrated enforcement is claimed.
RC6 acknowledgements below do not cover RC7 bytes.

## D9-204 SessionRanking inventory extension — 2026-09-13

Yvan's approved D9-204 behavior records the eligible operator ranking for each
planned V2 session so election and reward settlement use the same session-bound
ordering. `d9-node-registry::SessionRanking` is therefore classified as
`NotMigrated`, with no V1 import or genesis field; the runtime derives it from
eligible operators during session planning. The inventory marks the item
`proposed` until the corresponding pallets change is merged.

Contract digest:
`710f5670d22276fd65c1e6e012b174dd0790e356e4a2e08eac1d1efb107bf297`.
The 137-entry inventory and detached binding were regenerated coherently. All
17 shared tests passed, including the 80-case rejection corpus and exact
manifest/binding digest checks. The RC6 wire schema, complete fixture and input
digest are unchanged.

The previously merged D9-204 digest
`9b41f0cda8ad25b2b07db60dd0053078d88ab44cc5d4c717a87fcada7602ab72`
misclassified `SessionRanking` under `d9-node-rewards` and is superseded by
this corrected pallet ownership. The earlier RC6 contract digest
`94b42ed2efd9915b6f476ffb7da1c3ffb2d0e57e63183e6d360f4b3540841981`
does not cover this inventory extension. This record establishes the proposed
fresh-state classification; it does not merge the pallet, verify final runtime
composition, or satisfy the separate release gates.

## RC6 chronology correction — 2026-09-13

Yvan requires inverted merchant dates to be rejected, including authenticated
legacy rows: [decision](https://linear.app/d9-network/issue/D9-183#comment-a1f9a8dc-a8eb-4931-b049-26f29886e40d).
RC6 requires Some(lastConversion) >= createdAt, retaining the existing nonzero
source-time bounds. Equality and None remain valid. No clamp, dropped account or
legacy exception is permitted. Expiry policy is unchanged.

Chronology-only RC6 bundle digest, historical after the D9-204 inventory
extension: `94b42ed2efd9915b6f476ffb7da1c3ffb2d0e57e63183e6d360f4b3540841981`.
Shared tests: 17 passed, including 80 rejection cases. The chronology regression
failed before the fix. The paired local merchant genesis correction passed all
103 merchant tests, including refusal before storage writes and equality/None
preservation. These are synthetic/local checks, not real source acceptance.

RC6 is a local candidate. Existing RC5/RC3 consumer pins do not enforce this new
shared rule. Publish and repin the adapter/composer/exporter consumers coherently
before declaring integrated enforcement. Historical compatibility below applies
only to its stated revisions; it is not automatically RC6 acceptance.

## Historical RC5 compatibility disposition — 2026-09-12

All four interface scopes have now been technically reviewed against
`d9-native-genesis/0.1.0-rc.5`, contract source
`57b270d4922c2bab7aae33ce575a930ca08d893b`, contract digest
`65af6b84e0f940cb87f73a5bc851a4f8ffa22ff22fe77fda8c461504d6397f9c`.
Codex performed the current reconciliation under Yvan's instruction. This is
new technical review evidence, not a relabeling of historical RC3 approvals
or a signature on another person's behalf.

| Scope | Current RC5 evidence | Disposition |
|---|---|---|
| D9-183 merchant | 103 merchant tests and shared adapter storage tests; full history, nullable timestamps and expiry semantics unchanged in RC3-to-RC5 source diff | compatible |
| D9-184 LP | 58 AMM tests (one existing ignored) and shared adapter storage tests; complete ownership/amount map and reserve equality unchanged | compatible |
| D9-190 reserve policy | Yvan's 600k policy recorded on the issue; RC5 adapter pin and current runtime at pallets `a91d9cc8dabe43d324912dd39182b5799c54cee3`; boundary tests pass | compatible |
| D9-173 independent verifier | exact input/dataset/contract hashes independently reproduced, all seven gate interfaces reviewed; [full evidence](https://linear.app/d9-network/document/d9-173-rc5-compatibility-review-and-independent-fixture-evidence-f4cec6cfe9e1) | compatible for implementation |

### Reproducible consumer check

In a disposable worktree at pallets `a91d9cc8dabe43d324912dd39182b5799c54cee3`,
apply merchant commit `11f8f98e8552d905dc863c7b7b4c2117c52b48ad` using
`git cherry-pick --no-commit`. It applies cleanly and retains the exact RC5 pin.
Run `cargo test --locked --offline -p d9-genesis-adapter -p pallet-d9-merchant -p pallet-d9-amm`.
Result: 9 adapter + 103 merchant + 58 AMM tests passed; one existing AMM test
ignored, zero failures. This integration worktree is review evidence, not an
additional published runtime branch. Shared `cargo test --locked --offline`
also passed: 16 tests, including the 78-case rejection corpus.

Exact SHA-256 pins:
- complete fixture: `4ab1df08174dc553d425614c32cc8b739d470ab0721dc59f069d7e8f7baad2bc`
- expected: `8c2ff0ff1ccf0101c85a4f870fa6ebd68df1b502b00ada0d06f4fd932ccb6b4f`
- cases: `16a6ae2d985b79b0e273950f8c70cf45d0333399b031ca508bc4588456579ea7`

`RESERVE_DEPTH` explicitly supersedes the older `RESERVE` numeric clause:
minimum remaining reserves are 600000000000000000 raw D9 and
600000000000 raw USDT. Source balances and LP ownership remain exact. This
documentation correction changes no contract, schema, rule or fixture bytes.

The compatibility gap is resolved. Runtime PR integration, authenticated
frozen-source delivery, final composition/decoding, archive and legacy
settlement evidence, reproduction and release remain separate gates.
The dated sections below are historical and their pending-status statements
are superseded by this section.

## Historical review handoff


Candidate: **d9-native-genesis/0.1.0-rc.5**. Yvan approved the new economic reserve policy on 2026-09-12; exact RC5 downstream compatibility verification remains separate.

The table below preserves the three recorded **RC3** interface acknowledgements. D9-173 remains pending. These historical acknowledgements are not automatically relabeled as RC4 verification.
This file records existing decisions and their evidence, not new approval on another person's behalf.


| Owner | Scope | Evidence required | Status |
|---|---|---|---|
| Yvan review; Novy / D9-183 implementation | merchant full Accounts + expiry; time/profile semantics | consume complete fixture, preserve nullable/history/scale; refuse corresponding cases | interface acknowledged; implementation pending |
| Yvan / D9-184 review; Wen Ryu implementation | full LP owner book and proposed genesis fields | all ten synthetic owners survive; no sqrt; expected total is assertion-only | interface acknowledged; final integration pending |
| Yvan review; Wen Ryu / D9-190 implementation | per-asset units, reserve floor, bps and migration boundary | accept fixed source/target values; reserve-floor and post-migration behavior tests in pallet | interface acknowledged; implementation pending |
| Yvan / D9-173 | independent verifier | confirm fields/digests and paid-history/exception baseline; independent final-spec decoder planned | pending |
| Yvan | five economic/identity dispositions | five 2026-09-12 migration ADRs, linked in inventory | decided; delivery evidence pending |
| D9-380 ADR (archive owner unassigned) / D9-211 / D9-370 / D9-173 | archive, legacy settlement, fresh initial state | verified archive and funded disposition; empty/zero final-state readback and watermark | pending |
| Wen Ryu / D9-370 | final composition | confirm complete runtime config digest, five key roles across FRAME Session/D9 registry/liveness, derived pallet identities, final detached binding | pending |

Each acknowledgement must cite the same contract version, contractDigest,
fixture/expected/cases hashes, source commit and test evidence. If any field,
policy or expected result changes, re-run the entire shared corpus and update
the evidence; do not carry acknowledgement across different bytes.

## Recorded RC3 acknowledgements — 2026-09-12

All three decisions below refer to source
`d629d028dab2a8f154c57eda197ed7e1cf9d56bf`, contract digest
`d0f06baa97856920c43ac871736e5cfc4b675af82a84f46103d78db718098c6a`,
and the exact fixture hashes in [fixture-manifest.json](fixture-manifest.json).
This record-only update changes neither that contract nor its fixture bytes.

| Interface | Yvan's recorded decision | Implementation evidence cited in the decision |
|---|---|---|
| [D9-183](https://linear.app/d9-network/issue/D9-183) | 02:27:32 UTC; comment `56261cbd-667b-4789-8556-b96282fa35d1` | adapter `f4194533a23ab2f2c828895724164bae88342abf`: 7 adapter checks and 96 merchant tests; native merchant genesis and redemption still owed |
| [D9-184](https://linear.app/d9-network/issue/D9-184) | 02:38:56 UTC; comment `44480ce1-664c-4361-8317-06cf44abb498` | pallets `cdb272be0e8922542903702bf0d7d52d96156c2a`: 53 AMM and 7 adapter tests; complete mock owner map and reserves; multiple bootstrap providers rejected |
| [D9-190](https://linear.app/d9-network/issue/D9-190) | 02:41:06 UTC; comment `e8babab0-82bf-430b-8819-ffd57083144a` | RC3 conformance covers scaling, reserve floor and policy refusals; runtime reserve-floor and migration-document fixes remain implementation work |

[D9-173](https://linear.app/d9-network/issue/D9-173) has a review request but
no compatibility acknowledgement. Its independent decoder, final source/spec
comparison and the separate release gates are not completed by these decisions.

## RC2 X1 reconciliation

The September 3 D9-195 / DEC-20 exclusion ruling is applied; no new X1 policy
decision is requested. All prior RC1 compatibility evidence must be rerun against
RC3 hashes. The five previously null entries are now NotMigrated under the
September 12 ADRs; their operational evidence remains pending. JurorBallots
is classified as fresh V2 operational state consistently with VotingBodies.

## Local implementation acceptance

- [x] Strict transport DTOs + generated schema; no implicit nullable defaults.
- [x] Recovered D9-307 static core reused without changing its API.
- [x] Per-item inventory + five accepted ADR dispositions; future unknowns fail closed.
- [x] Source/target reference checks and typed runtime domain conversions.
- [x] Complete synthetic input + independent golden values/digests.
- [x] Negative corpus including paid-history loss and equal-sum wrong owners.
- [x] Detached final artifact binding + tamper cases.
- [x] D9-183 merchant interface acknowledgement.
- [x] D9-184 LP interface acknowledgement.
- [x] D9-190 reserve/units interface acknowledgement.
- [ ] D9-173 independent-verifier interface acknowledgement.
- [x] Five Yvan dispositions recorded for the contract.
- [ ] Archive, legacy settlement, clean V2 processing and fresh-state verification.
- [ ] Full final runtime build, provenance gate and independent state readback.

D9-380 is the bounded interface milestone: its four compatibility acknowledgements
must exist against this contract and fixture revision before marking it Done.
An individual interface block can be released by its recorded acknowledgement;
the remaining D9-173 block stays in place. That does not require the separate cutover/release
deliveries to be finished in this issue. Archive verification, legacy settlement,
clean processing, final-state checks and reproduction remain mandatory in their
own cutover/release gates under the ADRs and D9-307 / D9-173 / D9-370 / D9-315.
Neither contract tests nor D9-380 closure mark those gates passed.

## RC4 reserve-depth review (historical; superseded by RC5)

D9-190 policy is 1M whole tokens on each side: D9 10^18 raw, USDT 10^12 raw.
The source/target reserve amounts, LP owner map and five RC3 dispositions stay
unchanged. Verify quote/swap/withdrawal exact-boundary and one-below rejection
against the composed runtime. No prior compatibility acknowledgement or release
approval is implied by this candidate.

## RC5 reserve-depth review

Review fixed 600,000-token minima: D9 600000000000000000 raw and USDT
600000000000 raw, with no change to source/target reserves, LP ownership or
prior migration dispositions. Verify the new version, digest, fixture hashes
and exact-boundary runtime evidence. No prior acknowledgement is carried over.
