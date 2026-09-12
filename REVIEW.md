# D9-380 review handoff

Candidate: **d9-native-genesis/0.1.0-rc.2**. No downstream acknowledgement yet.
This file records required evidence, not approval on another person's behalf.

| Owner | Scope | Evidence required | Status |
|---|---|---|---|
| Novy / D9-183 | merchant full Accounts + expiry; time/profile semantics | consume complete fixture, preserve nullable/history/scale; refuse corresponding cases | pending |
| Wen Ryu / D9-184 | full LP owner book and proposed genesis fields | all ten synthetic owners survive; no sqrt; expected total is assertion-only | pending |
| Wen Ryu / D9-190 | per-asset units, reserve floor, bps and migration boundary | accept fixed source/target values; reserve-floor and post-migration behavior tests in pallet | pending |
| Yvan / D9-173 | independent verifier | confirm fields/digests and paid-history/exception baseline; independent final-spec decoder planned | pending |
| Yvan | economic/identity dispositions | resolve each null inventory item; authenticate real exception/disposition lists (X1 policy already settled) | pending |
| D9-370 owner | final composition | confirm complete runtime config digest, five key roles across FRAME Session/D9 registry/liveness, derived pallet identities, final detached binding | pending |

Each acknowledgement must cite the same contract version, contractDigest,
fixture/expected/cases hashes, source commit and test evidence. If any field,
policy or expected result changes, re-run the entire shared corpus and update
the evidence; do not carry acknowledgement across different bytes.

## RC2 X1 reconciliation

The September 3 D9-195 / DEC-20 exclusion ruling is applied; no new X1 policy
decision is requested. All prior RC1 compatibility evidence must be rerun against
RC2 hashes. Five unrelated null provenance entries remain unresolved. JurorBallots
is classified as fresh V2 operational state consistently with VotingBodies.

## Local implementation acceptance

- [x] Strict transport DTOs + generated schema; no implicit nullable defaults.
- [x] Recovered D9-307 static core reused without changing its API.
- [x] Per-item candidate inventory + named unresolved dispositions.
- [x] Source/target reference checks and typed runtime domain conversions.
- [x] Complete synthetic input + independent golden values/digests.
- [x] Negative corpus including paid-history loss and equal-sum wrong owners.
- [x] Detached final artifact binding + tamper cases.
- [ ] Four downstream compatibility acknowledgements.
- [ ] Yvan dispositions required for real migration input.
- [ ] Full final runtime build, provenance gate and independent state readback.

The last three checkboxes are not satisfied by this implementation or its unit
tests. D9-380 must not be marked Done or its four interface blocks removed until
the required review evidence exists. D9-307 / D9-173 / D9-370 / D9-315 retain
their separate release work.
