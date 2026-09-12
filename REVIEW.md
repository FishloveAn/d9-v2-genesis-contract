# D9-380 review handoff

Candidate: **d9-native-genesis/0.1.0-rc.3**. No downstream acknowledgement yet.
This file records required evidence, not approval on another person's behalf.

| Owner | Scope | Evidence required | Status |
|---|---|---|---|
| Novy / D9-183 | merchant full Accounts + expiry; time/profile semantics | consume complete fixture, preserve nullable/history/scale; refuse corresponding cases | pending |
| Yvan / D9-184 review; Wen Ryu implementation | full LP owner book and proposed genesis fields | all ten synthetic owners survive; no sqrt; expected total is assertion-only | pending |
| Wen Ryu / D9-190 | per-asset units, reserve floor, bps and migration boundary | accept fixed source/target values; reserve-floor and post-migration behavior tests in pallet | pending |
| Yvan / D9-173 | independent verifier | confirm fields/digests and paid-history/exception baseline; independent final-spec decoder planned | pending |
| Yvan | five economic/identity dispositions | five 2026-09-12 migration ADRs, linked in inventory | decided; delivery evidence pending |
| D9-380 ADR (archive owner unassigned) / D9-211 / D9-370 / D9-173 | archive, legacy settlement, fresh initial state | verified archive and funded disposition; empty/zero final-state readback and watermark | pending |
| Wen Ryu / D9-370 | final composition | confirm complete runtime config digest, five key roles across FRAME Session/D9 registry/liveness, derived pallet identities, final detached binding | pending |

Each acknowledgement must cite the same contract version, contractDigest,
fixture/expected/cases hashes, source commit and test evidence. If any field,
policy or expected result changes, re-run the entire shared corpus and update
the evidence; do not carry acknowledgement across different bytes.

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
- [ ] Four downstream compatibility acknowledgements.
- [x] Five Yvan dispositions recorded for the contract.
- [ ] Archive, legacy settlement, clean V2 processing and fresh-state verification.
- [ ] Full final runtime build, provenance gate and independent state readback.

The unchecked delivery/review obligations are not satisfied by this implementation
or its unit tests. D9-380 must not be marked Done or its four interface blocks removed until
the required review evidence exists. D9-307 / D9-173 / D9-370 / D9-315 retain
their separate release work.
