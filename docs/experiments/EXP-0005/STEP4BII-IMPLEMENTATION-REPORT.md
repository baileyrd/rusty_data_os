# Step 4b-ii implementation report (advisory)

Latest correction: [inspection 1 F1](#inspection-1-f1-correction). The original implementation
and proof below are retained as history; corrective evidence is recorded separately.

## Authority and scope

Implemented in `C:/dev/rusty_data_os-step4bii`, branch
`codex/merge-step4bii-memory-domain`, starting clean at
`5af480971b006b3f04ac14fc30127a0361f371ae`. The supplied plan file was read only for
SHA-256 verification; it matched
`995f2d6ea29af8547985673e71efd90cdf08f90e844db876f9a49f75d53c5d29`.
All source paths resolved in this checkout; no original checkout was edited.
No commit, push, publication, external-provider review or CI change was performed.
Independent review by another provider remains pending; these results are advisory.

## Implementation and disclosed R0 exceptions

- uc-core: added Send to writer/hook trait objects, the set_hook/wrap_writer call
  boundaries and two existing injected-writer declarations. Existing test assertions
  and behavior were unchanged.
- uc-memory: named MAX_OPERATIONS and raised the decode cap from 1024 to 4096.
- uc-entity/uc-relation: raised their existing MAX_OPERATIONS constants to 4096.
- uc-protocol: Sum/Avg use i128 accumulation. Sum outside i64 returns Malformed through
  fallible reduction/group evaluation and dispatch. Avg casts sum and count before
  division, preserving fractional/negative results and empty-input 0.0. The existing
  connection loop, wire types and 66 fixtures are unchanged. This is a 4b-i residual
  closed under D7/R0, also recorded in BUILD-LOG.md.

The new uc-facade crate has exactly the six authorized path dependencies. It contains
three explicit Store implementations, each owning its engine behind one Mutex, complete
tag/type decoding, literal field capabilities, raw-byte IDs and internally selected
incarnations. Whole-record guards reuse uc-protocol's predicate functions. Read-set
conflicts precede indexed update preconditions; commits submit one engine transaction.
Unsupported operations retain the specified defaults/explicit implementations.

## Correctness evidence

The R6 test is named
`real_tcp_three_domains_crud_entity_links_batches_and_all_flags_sessions`.
Each domain's own same-table insert/get/replace/delete and separate all-flags session,
including Entity's open-label links, ran over a real TcpListener-accepted socket. A
single TcpStream switches all three tables and sends both a non-atomic WriteBatch and
the following Get before reading either response. These are real engine Stores and
real TCP, using uc-protocol framing/codec as client, not in-process dispatch calls.

Four further real-socket tests cover nonempty atomic batch refusal on all three tables,
two-connection session conflicts, 4096 staged operations plus the 4097th refusal and
reopen, and an aggregate overflow response followed by another successful request.
Six direct adapter tests cover exact fields/schema/capabilities, malformed and semantic
refusals, all six whole-record guard operators, concurrent guarded replacements with
one winner, shared StrList predicate refusal, read-set precedence/indexed refusal without partial apply, one physical
transaction for 4096 changes, replay, same-table symmetric links, known-label retention,
delete/reinsert and edge cleanup. UUID fixtures use distinct bytes, including non-v4
record IDs, to detect byte-order/version assumptions. All new facade tests use D1.

No cross-table mentions scenario and no cross-domain session were attempted or claimed.
Memory's mentions descriptor has target_table None. Real foreign-edge durability remains
deferred; existing synthetic 4b-i cross-table tests remain their own historical evidence.
No legacy client binary, cross-repo integration, benchmark, production or new durability
claim is made. HYP-0005 stays Open; EXP-0005 stays Ready and ADR-0005 stays Proposed.

## Proof

The five affected crates' existing suites passed 94 tests after R0, before the integration proof:
[R0 output](step4bii-r0-proof-output.txt). The additional aggregate regression and all
existing protocol tests passed (39 tests): [aggregate output](step4bii-aggregate-proof-output.txt).
The ten new facade tests and targeted warnings-denied Clippy passed during development:
[facade test output](step4bii-facade-development-output.txt).

The [initial eleven-command proof](step4bii-initial-proof-output.txt) exited 0 with
108 unified-commitment, 17 convergence-memory and 95 portable exp-0001 tests passing.
The StrList admission test was added after that source run. The
[final eleven-command proof](step4bii-proof-output.txt) also exited **0**:
**109 unified-commitment + 17 convergence-memory + 95 portable exp-0001 = 221 tests
passed, 0 failed**. All three formatting and warnings-denied Clippy checks, Markdown
links and git diff --check passed. All eleven facade tests, including the named R6 real
TCP test and the StrList refusal test, are included in that final output. Only result
documentation was updated afterward; final Markdown/whitespace checks were repeated.
The exact owner command chain ran
from checkout root with Windows GNU Rust 1.89.0, formatting, locked/offline Clippy and
all-target tests for unified-commitment, convergence-memory and portable exp-0001
(exp1-descriptive-d1-harness excluded), then Markdown links and git diff --check.

## Deviations and operational dispositions

One wording conflict was identified: the work order describes ReplaceIf guards against
"any field", but the required unchanged shared validator in uc-protocol/src/query.rs
admits only U32/I64/Bool/Str predicates. Memory tags and Entity aliases are StrList, so
even Eq/Ne guards on them return Malformed. Supporting those fields would require a
forbidden protocol change or a second evaluator. **Proposed deviation:** interpret "any
field" as any field admitted by that frozen validator. The implementation preserves the
explicit reuse/no-change requirements and adds
shared_predicate_admission_refuses_string_list_guards to prove refusal without mutation.
This limitation is reported for independent review; no new predicate semantics were added.

Two implementation details refine the illustrative R5 function signature: a
LoopbackListener wrapper enforces the sole bind address, and serve accepts an AtomicBool
stop signal. The nonblocking accept loop checks it at 5 ms intervals, spawns one thread
per accepted socket and joins all workers on exit. Callers close clients before joining;
the signal is not cancellation of idle clients. uc-harness is a runner binary, so this
test-driven listener remains a library function with no new server executable.

The Sum overflow disposition is explicitly Malformed, as permitted by D7. Static schema
methods need no engine access; all state reads/checks/mutations retain the table lock.
Infallible Store reads panic on mutex poison; error-returning operations return Storage.
Storage also represents core errors/indeterminate outcomes because the wire has no
request-ID resolution mechanism; no automatic retry or false atomic refusal is invented.

Denied/blocked actions: the initial PowerShell profile attempted denied Terminal-Icons
configuration exports and a user PATH registry write outside the checkout. Subsequent
commands used login=false (no profile). rg was unavailable, so searches used native
PowerShell. No implementation or proof action was denied; no escalation was requested.
One documentation patch failed exact-context validation and applied nothing; it was
corrected. The preliminary Markdown check found two new README links one parent level
too high; they were corrected before full proof. Development Clippy and facade tests
passed on their first completed run.

## Files changed

Paths below are relative to this checkout.

- New crate: experiments/unified-commitment/crates/uc-facade/Cargo.toml, README.md,
  src/lib.rs, src/memory.rs, src/entity.rs, src/relation.rs, src/listener.rs,
  tests/adapters.rs, tests/tcp.rs and tests/support/mod.rs.
- Workspace registration: experiments/unified-commitment/Cargo.toml and Cargo.lock;
  README.md documents the facade and new operation cap.
- R0: experiments/unified-commitment/crates/uc-core/src/lib.rs and tests/recovery.rs;
  uc-memory/src/lib.rs, uc-entity/src/lib.rs, uc-relation/src/lib.rs;
  uc-protocol/src/query.rs, src/dispatch.rs and tests/dispatch.rs.
- Synchronized documentation: AGENTS.md, docs/PROJECT-STATUS.md,
  docs/experiments/EXP-0004-entity-relation-domains.md,
  docs/experiments/EXP-0005-protocol-facade.md,
  docs/hypotheses/HYP-0005-protocol-facade.md, docs/adr/ADR-0005-protocol-facade.md,
  handoff-2026-09-08/BUILD-LOG.md, this report and the accompanying step4bii proof outputs.

Prior evidence files, CI, uc-harness source and all other crates remain unchanged.

## Inspection 1 F1 correction

The host's accepted F1-DESCRIBE-RELATIONS-WILDCARD-DROPPED applies to inspection snapshot
`78e7839e…8d3fdb6`. Direct source verification and two new real-TCP tests confirmed the
custom MemoryStore/EntityStore describe_relations overrides omitted Neighbors(None).
The initial report did not disclose this capability loss, and the original eleven facade
tests did not exercise Join. The correction deletes both overrides exactly as requested;
Store's existing default supplies wildcard and named descriptors with target_table None.

Tests `real_tcp_unlabeled_memory_join` and `real_tcp_unlabeled_entity_join` each send
Request::Join with Neighbors(None) and right_table None through a real TcpListener-accepted
socket. They assert exact rows in both edge directions, exclude an unlinked fourth record,
cover two distinct Entity labels, and check wildcard/named descriptor retention and absence
of foreign targets. [Before-fix output](step4bii-f1-before-fix-output.txt): both fail with
Malformed, exit 101 as expected. [After-fix output](step4bii-f1-after-fix-output.txt): both
pass, exit 0. The [full owner proof chain](step4bii-f1-proof-output.txt) exited **0**:
**111 unified-commitment + 17 convergence-memory + 95 portable exp-0001 = 223 tests
passed, 0 failed**. All formatting, warnings-denied locked/offline Clippy, Markdown links
and git diff --check passed. The full run includes all thirteen facade tests, including
both F1 regressions and the original R6 test. Only result documentation changed afterward;
the [final documentation check](step4bii-f1-final-documentation-check.txt) rechecks links
and whitespace.

Files changed in this correction, relative to checkout:

- experiments/unified-commitment/crates/uc-facade/src/memory.rs and src/entity.rs:
  removed only the custom descriptor overrides.
- experiments/unified-commitment/crates/uc-facade/tests/tcp.rs: two regression tests and
  their shared same-table scenario helper.
- experiments/unified-commitment/crates/uc-facade/README.md; docs/PROJECT-STATUS.md;
  docs/hypotheses/HYP-0005-protocol-facade.md; docs/adr/ADR-0005-protocol-facade.md;
  docs/experiments/EXP-0005-protocol-facade.md; this report;
  handoff-2026-09-08/BUILD-LOG.md: synchronized correction and evidence documentation.
- New step4bii-f1 before/after/full-proof/final-documentation output files alongside this report.

No denied or blocked action, impossible requirement or new deviation in this correction.
Prior dispositions are preserved without reopening them. All source changes stay in this
checkout; the plan hash was reverified, and HEAD remains the original uncommitted base.
No commit, push or publication was performed. No cross-table mentions scenario or
cross-domain session was attempted or claimed. The original R6 test remains
real_tcp_three_domains_crud_entity_links_batches_and_all_flags_sessions: each domain's own
same-table operations, including Entity's open-label relation, run over a real socket.
Results remain advisory pending independent re-review.
