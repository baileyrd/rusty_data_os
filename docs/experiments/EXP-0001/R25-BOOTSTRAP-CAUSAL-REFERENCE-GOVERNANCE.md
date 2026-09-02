# R25 Bootstrap Causal-Reference Governance Correction

**Profile:** `EXP-0001-BOOTSTRAP-CAUSAL-REFERENCE-v2`
**Status:** frozen prospective v2 documentation design; no code or execution authorization
**Evidence classification:** failed-governance-assumption record and corrective governance only;
not implementation, correctness, workload, benchmark, durability, recovery, or performance evidence
**Decision date:** 2026-09-02

## 1. Failed assumption and preserved evidence

R24 assumed that the unchanged R12/R14/R16 v1 authorities could produce an authority-valid stream
that begins at an accepted-prefix bootstrap operation and later exercises a valid causal reference.
PR #91 tested that assumption and falsified it. The contradiction is exact:

1. R16 sections 2.3–2.4 declare one `profiles.envelope` value for the manifest-bound stream and
   require causal manifests to have one positive scalar `reference_cardinality`.
2. R12 section 5.2 requires every v1 causal envelope to carry at least one target, and section 5.3
   requires `k>0`, `k<=i`, and a prior ordinary EventId.
3. R22 makes `[0,i)` strictly same-stream and same-segment, so segment ordinal 0 has no eligible
   target; in particular, measured ordinal 0 cannot target warm-up.
4. R21 sections 4–5 initialize accepted-prefix mapping at the selected stream's first WS1
   operation and advance it transactionally. It cannot skip an invalid bootstrap to reach a later valid operation.
5. R23 sections 3–4 require exact, fully R14/R16-valid manifest-bound streams; they cannot repair,
   mix, omit, or substitute operations to manufacture a valid context.
6. R24 sections 2 and 6 require a valid bootstrap-to-prior-reference test while forbidding changes to
   the authority crates and manifests. Its implementation authorization was therefore unsatisfiable.

The owner closed PR #91 without merge after the valid multi-operation reference fixture gate proved
impossible under those frozen rules. Its branch and review history are preserved as negative evidence;
none of its code or tests is merged evidence. The failed assumption is that R21–R23 governance alone
made the R24 implementation boundary executable. R25 corrects that governance assumption rather than
concealing the failure.

## 2. Immutable v1 boundary

R25 does **not** amend or reinterpret `EXP-0001-ENVELOPE-INPUT-v1`,
`EXP-0001-PRIOR-EVENTS-v1`, `EXP-0001-SEMANTIC-OP-v1`,
`EXP-0001-WORKLOAD-STREAM-v1`, `EXP-0001-WORKLOAD-MANIFEST-JCS-v1`, or any associated
R12/R14/R16 documentation vector, manifest, stream, digest, fixture, or byte. Every v1 success and
failure remains governed by its original authority. Existing artifacts remain immutable historical
evidence and are never accepted as v2 by alias, fallback, negotiation, or silent reinterpretation.

V2 requires new workload, envelope-generator, reference-generator, semantic-operation, stream,
manifest, and digest profile identifiers and new vectors. Their exact byte encodings and digests are
deliberately deferred to a separately authorized conformance/validator documentation and
implementation increment.

## 3. Prospective v2 semantic rule

The prospective uniform causal profile is `envelope-causal-reference-v2`. For each
`(stream_namespace, segment)` independently:

- segment ordinal 0 is the **only bootstrap position** and is valid with causal semantics when it
  has exactly zero targets;
- every operation at segment ordinal greater than 0 MUST contain one or more ordered targets selected
  from prior ordinary EventIds in that same stream and segment;
- warm-up ordinal 0 and measured ordinal 0 therefore each bootstrap independently;
- measured bootstrap MUST NOT refer to warm-up, because R22's segment boundary remains absolute;
- no cross-stream or cross-segment bootstrap exception exists;
- bootstrap is not permission to target self, future, missing, wrong-kind, wrong-fact, duplicate,
  cross-stream, or cross-segment identities; and
- all non-bootstrap causal target ordering and prior-reference rules remain those of R12 as narrowed
  by R21–R23.

`E-REFERENCE-CARDINALITY` applies only when a non-bootstrap causal operation has zero targets. A
bootstrap with zero targets is valid. At any position, malformed target encoding or duplicate target
bytes retain the unchanged semantic-validation precedence from R21. After that validation, a
syntactically valid target-bearing bootstrap is classified by applying the unchanged R21–R23 ordered
target rules; its mere presence at the bootstrap position does not produce an earlier cardinality
error. Thus cross-stream and cross-segment targets at bootstrap produce their respective ordered
errors, as V25-06a and V25-06b require. Validation never filters a bad target to turn the operation
into a valid zero-target bootstrap.

## 4. Required v2 manifest and generator representation

The v1 scalar `generator_inputs.reference_cardinality` cannot represent v2 and MUST NOT appear in a
v2 manifest. The future closed `EXP-0001-WORKLOAD-MANIFEST-JCS-v2` ledger must instead contain the
following required object (member names and semantic types are frozen here; canonical bytes await the
separate v2 conformance freeze):

```json
"reference_cardinality_policy": {
  "kind": "segment_bootstrap_then_prior_v2",
  "warm_up": {"bootstrap":"0","subsequent":"1"},
  "measured": {"bootstrap":"0","subsequent":"1"}
}
```

`bootstrap` is exactly the canonical u64 text `0`. `subsequent` is canonical u64 text in
`1..18446744073709551615` and is the fixed `k` used for every non-bootstrap operation in that
segment; generation fails before output where `k` exceeds the available same-segment ordinary
prefix. Both segment objects are mandatory even when that segment's operation count is zero, which
prevents an implicit warm-up-to-measured inheritance. A later profile may vary `k` further only via
another versioned governance decision; it may not restore one scalar for bootstrap and subsequent
positions.

The v2 generator computes `k=0` iff segment ordinal is 0 and otherwise uses that segment's positive
`subsequent` value. It emits causal semantics at both positions and binds the resulting operation to
the v2 envelope/reference/semantic-operation/stream profiles. The v2 manifest validator must check
the policy against every decoded operation, segment count, segment ordinal, target count, R22 domain,
and R21–R23 disposition. Mixed v1/v2 profile combinations fail closed.

## 5. Documentation vectors

These semantic vectors freeze required outcomes, not bytes. The later v2 conformance freeze must give
each a literal operation/stream/manifest vector and digest without altering v1 vectors.

| Vector | Segment and operation | Targets | Expected disposition |
|---|---|---|---|
| V25-01 | warm-up ordinal 0 | none | valid causal warm-up bootstrap |
| V25-02 | warm-up ordinal 1 | warm-up ordinary EventId at ordinal 0 | valid ordered prior reference |
| V25-03 | measured ordinal 0 | none | valid causal measured bootstrap, independent of warm-up |
| V25-04 | measured ordinal 1 | measured ordinary EventId at ordinal 0 | valid ordered prior reference |
| V25-05 | either segment ordinal 1 | none | `E-REFERENCE-CARDINALITY`; invalid non-bootstrap causal operation |
| V25-06a | measured ordinal 0 | warm-up ordinary EventId | `E-REFERENCE-CROSS-SEGMENT`; no bootstrap exception |
| V25-06b | either segment ordinal 0 | ordinary EventId from another stream | `E-REFERENCE-CROSS-STREAM`; no bootstrap exception |

V25-06a and V25-06b are ordered-target classifications, not cardinality failures. The later vectors
must additionally retain R21–R23 tests for malformed and duplicate target semantic-validation
precedence and for self, future, wrong-kind, wrong-fact, missing, ordered-target precedence, exact
closed scope, and transactional accepted-prefix behavior.

## 6. Authorization and gates

R25 supersedes R24 **only as prospective implementation authorization**. R24 is not complete, was
not implemented on `main`, and has no merged correctness evidence. Its text, PR #91 history, and
failed gate remain unchanged historical records. The complete R20 gate remains open.

R25 authorizes no Rust, Cargo, lockfile, workflow, authority-crate, test, fixture, generated workload,
harness, capture, execution, benchmark, durability, recovery, append/reopen, fault, adapter,
production, server, networking, query, or distributed change. It also authorizes no v2 bytes.

Work may proceed only through two later, separately reviewed authorizations in this order:

1. freeze and authorize the v2 conformance/validator changes and literal vectors, including all new
   profile identifiers, exact encodings, canonical manifest bytes, domains, digests, immutable
   supersession/bindings, and negative gates; then
2. after that conformance increment is merged and reviewed, authorize a new bounded
   reference-context implementation PR against the v2 authority and its valid bootstrap-to-reference
   fixtures.

Neither later authorization is implied by R25. Live Linux capture and the descriptive D1 harness
remain independently blocked, and all workload and benchmark execution remains unauthorized.
