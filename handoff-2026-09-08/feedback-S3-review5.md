# Host dispositions on Codex review 5 of the Step 3 work order (plan sha256 4ed583e1…9aa4)

Both findings accepted; both were introduced by the round-4 envelope text and are corrected.

- UCR-012 (high): the recovery check now validates each `Final`'s envelope against its outer
  fields and its `Binding` only; `Commit` agreement is checked only when a `Commit` selects
  that `Final`, using the frozen `validate_lifecycle` rule. A valid `Final` with no complete
  adjacent `Commit` (absent or truncated as a torn tail) is uncommitted residue as R5
  prescribes: excluded from published state, reported, never an open failure. R7 now covers an
  absent `Commit` and a truncated `Commit` at both `D1` and `D2`.
- UCR-013 (medium): the `UCE1` requested-durability byte must be a valid value equal to the
  durability byte inside the embedded `UCR1` bytes; `achieved` durability for a recovered
  commit is derived from the validated bound request; the envelope-mismatch unit tests must
  keep valid RF1 CRCs by re-encoding altered frames.

Please re-review the revised plan (new hash) for approval.
