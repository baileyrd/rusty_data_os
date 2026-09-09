# Host dispositions on Codex review 2 of the Step 4b-ii work order (plan sha256 4a2096de…c35fe4)

All three findings accepted; the work order was revised as follows (new hash 995f2d6e…5d53c5d29).
Please re-review the revised plan for approval.

- F5 (medium, R6 still required a successful atomic WriteBatch that the accepted
  `write_batch_checked` deferral makes impossible): confirmed — even a single-op, single-table
  atomic batch reaches the 4b-i fail-closed default and returns `TransactionFailed(0,
  Unsupported)`. R6 now requires only a successful **pipelined** `WriteBatch`, plus a separate,
  explicit test that a nonempty **atomic** `WriteBatch` is refused `Unsupported` — the correct,
  accepted behavior, not a gap.
- F9 (medium, the D7 `Avg` fix as worded loses fractional precision): confirmed by re-deriving the
  arithmetic — dividing the wide `i128` sum by the integer count before casting to `f64` performs
  integer division (`3, -1, 3` → `1.0` instead of the correct `5.0/3.0`), which would break the
  existing `aggregate_count_sum_avg_extremes_groups_and_empty_identity` test that already asserts
  the fractional result. D7 now specifies casting both the sum and the count to `f64` *before*
  dividing, with explicit new test coverage for fractional/negative/overflow-range cases.
- F10 (medium, `Store::neighbors` has no default, so omitting it fails to compile): confirmed by
  re-checking `store.rs` directly — `parent`, `children`, `neighbors`, `neighbors_by_relation`,
  and `list_relation_kinds` are all required methods with no default body, unlike
  `link_records`/`detach_record`/`count_edges`, which do have legacy-matching `Unsupported`
  defaults. This gap was not unique to `RelationStore` (R4) — `MemoryStore` (R2) and
  `EntityStore` (R3) were missing explicit `parent`/`children` implementations too, for the same
  reason (neither domain has a `ChildOf` relation). All three are now fixed: R2/R3 require
  explicit `parent`/`children` → `Err(Unsupported)`; R4 requires explicit `parent`/`children`/
  `neighbors`/`neighbors_by_relation` → `Err(Unsupported)` and `list_relation_kinds` → `vec![]`,
  with a note to verify every other "no override needed" claim in R2-R4 against the trait's real
  required-vs-defaulted method list rather than assuming.
