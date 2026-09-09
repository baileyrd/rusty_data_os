# Host dispositions on Codex review 2 of the Step 3 work order (plan sha256 7411e120…b385)

Both findings accepted; the work order was revised. Please re-review the revised plan (new
hash) for approval.

- UCR-008 (medium, untracked sources unrecoverable): R8b now requires `source.patch` to be
  recoverable: `git diff HEAD --binary` plus one `git diff --no-index --binary NUL <file>` hunk
  per untracked, non-ignored file under the prefixes, so `revision` + `source.patch`
  reconstructs every manifest entry; a test applies the retained patch to a fresh worktree of
  `revision` and checks every manifest hash; the results index accepts a series as evidence
  only with clean porcelain at `revision` or a patch that reconstructs every manifest entry,
  verified by the host before cataloguing.
- UCR-009 (medium, digest-only normalized request conflicts with R5): R5 now stores the
  complete versioned serialization in `Binding.normalized_request` (`UCR1\0`, durability
  byte, u64 LE payload length, payload bytes); the SHA-256 is an auxiliary in-memory and
  checkpoint-cache lookup key only; retry equality is byte-identical normalized request; the
  doubled payload write is recorded as a measured cost with payload-by-reference as a §18
  follow-on.
