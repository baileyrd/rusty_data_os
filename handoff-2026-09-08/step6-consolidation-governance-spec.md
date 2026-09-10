# Step 6 work order — governance consolidation (the honestly-achievable slice)

## Host decision (2026-09-09, autonomous per the owner's `/goal` directive)

The merge plan's Step 6 ("Consolidate the projects") "done when" bar is: "active consumers use one
maintained engine, compatibility obligations are covered, and there is no ambiguity about which
runtime owns new database development." The first clause depends on a real consumer actually
cutting over — Step 5 built and verified the *mechanism* for that (export/import/compare/reopen/
backup-restore, all against synthetic fixtures) but explicitly did not perform a live cutover,
for the reasons recorded in that step's own "Host decision": `rusty_remind_me` has real,
differentiated capabilities (FTS5 search, ACT-R vitality scoring, a wiki, vector search, multi-node
sync) with no equivalent in `uc-*` yet, so an actual cutover today would either silently regress a
live, daily-used personal system or require a large, separately-scoped feature-parity effort. That
constraint is unchanged by reaching Step 6; nothing here proposes to close it.

Similarly, "remove duplicate runtime paths" and "archive the old repository" are explicitly gated
by the plan's own text on consumers "that rely on them hav[ing] migrated" — `rusty_multimodal_db`
remains an independently active repository (recent commits, open PRs, its own roadmap) with no
migrated consumer yet. Archiving or removing anything there now would be exactly the kind of
hard-to-reverse, high-blast-radius action this project's own working agreement treats with
deliberate caution regardless of any standing autonomy directive — and the plan's own gating
condition for it simply is not met yet. **This work order does not touch `rusty_multimodal_db` at
all, and does not remove or deprecate anything in `rusty_data_os`.**

**What is honestly achievable right now, and what this work order does:** the third clause — "no
ambiguity about which runtime owns new database development" — is a governance/documentation
question, not one gated on a real migration. Investigation found this repository's own top-level
governance documents (`docs/VISION.md`, `AGENTS.md`, `docs/ARCHITECTURE.md`) have **never been
updated to reflect the merge plan at all** — zero mentions of `rusty_multimodal_db`, the
replacement-database goal, or `experiments/unified-commitment` anywhere in those three files,
despite the merge plan's own "Working approach" section instructing exactly this: "Briefly align
active repository instructions with that goal in the first relevant implementation change while
preserving existing research evidence." That alignment was never done at Step 2 (or any step
since) — a real, disclosed gap, not something invented for this work order. This work order does
that alignment now: a small, clearly-marked, additive section in each of the three documents
stating the merge goal, `experiments/unified-commitment`'s role as where new database-engine
development for that goal happens, and `rusty_multimodal_db`'s status as the still-active source
repository whose consumers have not yet migrated — **without rewriting or diluting `VISION.md`'s
existing research culture for the separate, prior EXP-0001 exploratory-history thread**, which
remains a real, ongoing, independent research direction in this same repository.

## Target repository

New worktree `C:/dev/rusty_data_os-step6`, branch `codex/merge-step6-governance`, based on Step 5's
closing commit. Pure documentation; no source, test, or manifest changes.

## Repository facts

### The merge plan's own instruction, not yet followed

`data-os-multimodal-merge-plan-2026-09-08.md`, "Working approach": "Do not create a
documentation-only approval ladder or reopen the owner's replacement-database goal. Briefly align
active repository instructions with that goal in the first relevant implementation change while
preserving existing research evidence and useful correctness requirements." Confirmed by direct
search: `rusty_multimodal_db` and `experiments/unified-commitment` appear nowhere in `AGENTS.md`,
`docs/VISION.md`, or `docs/ARCHITECTURE.md` as of Step 5's close.

### The existing, narrower precedent for a disclosed, bounded exception

`docs/RESEARCH-ROADMAP.md`'s "Phase 7 — Server adapter" section already carries exactly this
pattern, added at Step 4b-i: "EXP-0005 (2026-09-09) is a bounded, migration-driven exception to
this phase ordering: the merge plan's Step 4 compatibility facade for existing `rusty_multimodal_db`
clients is authorized now, scoped to protocol-22 wire compatibility for the domains the merge plan
ports. It does not advance this roadmap's Phase 2-6 status and authorizes no other server/
networking work." This work order's additions follow the same shape: named, dated, scoped,
explicit about what it does *not* authorize.

### `docs/VISION.md`'s existing research culture (not being replaced)

`docs/VISION.md` §7 ("Initial non-goals") lists "replacing every existing database workload" and
committing to a server as *not* an initial goal for the EXP-0001 exploratory-history research
thread; §6 places "server adapter" and "distributed capabilities" at the far end of a scope
progression gated on core execution/persistence ideas "earn[ing] confidence independently of
network and service-layer complexity" first. The merge plan's parallel initiative
(`experiments/unified-commitment`, Steps 2-5) already skips ahead of that progression under
disclosed, bounded exceptions (Phase 7's note above) — this work order makes that coexistence
explicit in `VISION.md` itself rather than leaving a reader to reconstruct it from scattered
per-experiment notes.

## Design decisions settled by the host

- **D1 — additive sections only, clearly dated and scoped, no rewrite of existing prose.** Each of
  the three files gets one new, clearly-headed section (not an edit to existing paragraphs)
  explaining the merge initiative's relationship to that document's existing content. This matches
  the project's own established pattern (Phase 7's note in `RESEARCH-ROADMAP.md`) rather than
  inventing a new convention.
- **D2 (revised, review round 1 S6-001) — `docs/VISION.md` gets a new closing section, "9. A
  parallel, disclosed initiative (2026-09-09)," which narrows exactly which §7 non-goals are
  superseded and explicitly re-affirms the rest.** States: the merge plan
  (`data-os-multimodal-merge-plan-2026-09-08.md`) is a separate, owner-authorized initiative to
  build a general-purpose database intended to replace SQLite/PostgreSQL/DuckDB workloads, using
  this repository as "the eventual home of the combined engine" (the plan's own words); it proceeds
  under `experiments/unified-commitment/` in parallel with, not instead of, the EXP-0001
  exploratory-history research this document otherwise describes. **Only the following §7 items are
  superseded, each by a named, already-authorized exception, and only to the stated extent:**
  "full SQL compatibility" and "replacing every existing database workload" (the plan's own stated
  end goal, not yet achieved — Steps 1-5 have ported three domains' basic operations, nothing
  resembling full coverage); "cloud service deployment," "generalized plugin marketplaces,"
  "distributed consensus," "multi-node replication" (still genuinely not goals of the merge
  initiative either — nothing in Steps 1-5 touches any of these; listed only because a future
  reader should not assume otherwise from the "replace SQLite/PostgreSQL/DuckDB" framing alone).
  §6's "server adapter" step is superseded specifically and only to the extent
  `docs/RESEARCH-ROADMAP.md`'s Phase 7 note already authorizes (protocol-22 wire compatibility for
  the ported domains) — cites that note directly, does not restate or broaden it here. **Explicitly
  NOT superseded, restated as still fully in force for the merge initiative too:** §7's "committing
  to a permanent event encoding before measurement" and "hiding durability semantics behind vague
  'successful write' behavior" — these are engineering-rigor principles, not narrow EXP-0001 scope
  boundaries, and the merge initiative has in fact already followed them throughout (the CMM2→CMM3
  magic-byte version bump rather than silent format extension, Step 4c's BUILD-LOG; Step 5's test
  code explicitly labeling its `Durability::D1` choice "synthetic D1 fidelity only," never
  implying a stronger guarantee). §5's success criteria are unmet and not claimed met. No change to
  §§1-6, 8's content.
- **D3 (revised, review round 1 S6-002) — `AGENTS.md` gets a new numbered section (after the
  existing authority-order list) naming `experiments/unified-commitment/` as the merge initiative's
  home and `rusty_multimodal_db` as the active source repository it ports from, with
  revision-pinned pointers to the *actual* authoritative evidence, not a bare directory name.**
  States plainly: `rusty_multimodal_db` is not archived and not deprecated — it remains the
  authoritative running system for its own consumers until a real migration (not a proof) actually
  lands per merge-plan Step 5's remaining, undone final sub-step; new development *for the merge's
  goal specifically* happens in `experiments/unified-commitment/`, not by adding capability to
  `rusty_multimodal_db` or by starting a third, competing implementation. **Round 1 found that this
  worktree's own local `handoff-2026-09-08/` copy is incomplete** (branched at Step 5's close, it
  carries only early specs and this worktree's own append-only "local implementation" notes, not
  the accumulating history every subsequent step adds only to the separate `handoff/merge-2026-09-08`
  branch) — a bare `handoff-2026-09-08/BUILD-LOG.md` reference would resolve to a real but
  misleadingly incomplete file for anyone reading this branch alone, and the markdown-link
  validator cannot detect that (it checks existence, not completeness). Fixed: cite the
  authoritative source explicitly as "the `handoff/merge-2026-09-08` branch's
  `handoff-2026-09-08/BUILD-LOG.md`" (naming the branch, not just the path), and separately note
  that each step's own worktree/branch (e.g. this repository's `codex/merge-step5-migration-proof`)
  carries a local, partial advisory report for that step's own build only
  (`docs/experiments/EXP-0005/STEP<N>-IMPLEMENTATION-REPORT.md`), not the full cross-step closure
  log.
- **D4 (revised, review round 1 S6-002) — `docs/ARCHITECTURE.md` gets a short pointer, not a
  duplicate description, with the same branch-qualified citation as D3.** A brief new section
  noting `experiments/unified-commitment/`'s existence and pointing to
  `docs/experiments/EXP-0005-protocol-facade.md` (present in this checkout) for its architecture,
  and to "the `handoff/merge-2026-09-08` branch's `handoff-2026-09-08/` specs" (named explicitly as
  a different branch, not implied to be in the current checkout) for the full per-step work orders
  — rather than re-describing crate boundaries already documented there (avoiding two descriptions
  of the same system drifting apart).
- **D5 — an explicit "not yet done, and why" list, in `AGENTS.md`'s new section.** Names, plainly:
  no real consumer has migrated (Step 5 built and verified the mechanism only); `rusty_multimodal_db`
  is not archived and has no migrated consumer; no duplicate runtime path has been removed; opening
  the listener beyond loopback and real authentication remain open (carried from Step 4c's own
  handoff). This is what actually satisfies Step 6's "no ambiguity" clause honestly — ambiguity is
  removed by stating the true state precisely, not by overclaiming completion.

## Required changes

**R1 — `docs/VISION.md`:** add D2's new "9. A parallel, disclosed initiative (2026-09-09)" section
at the end of the file.

**R2 — `AGENTS.md`:** add D3/D5's new section (suggested heading: "9. The multimodal-merge
initiative") after the existing authority-order list (section 1), renumbering nothing else.

**R3 — `docs/ARCHITECTURE.md`:** add D4's short pointer section.

**R4 — `handoff-2026-09-08/BUILD-LOG.md` and the handoff prompt (on `handoff/merge-2026-09-08`,
not this worktree):** after this work order closes, record it there too, matching every prior
step's closing pattern — the host does this after independent verification, not the builder.

## Non-goals

No change to `rusty_multimodal_db` (not archived, not deprecated, not modified in any way). No
removal of any code, crate, or runtime path in `rusty_data_os`. No claim that any real consumer has
migrated. No change to `docs/VISION.md` §§1-8, `docs/RESEARCH-ROADMAP.md`'s existing phase
structure, or any prior ADR/decision. No change to `experiments/unified-commitment`'s code, tests,
or manifests — this work order is pure top-level documentation. No opening the listener beyond
loopback, no real authentication (still open, unrelated to this work order).

## Proof

Documentation-only change; the code-focused eleven-command proof chain does not apply (no crate
touched). Required checks: `python tools/validate_markdown_links.py` and `git diff --check` both
exit 0. Required review, named explicitly in the implementation report: each of the three new
sections is read back and confirmed to (1) not alter any pre-existing sentence in its file, (2)
accurately cite the real file paths/section names referenced (`data-os-multimodal-merge-plan-2026-09-08.md`,
`docs/RESEARCH-ROADMAP.md`'s Phase 7 section, `docs/experiments/EXP-0005-protocol-facade.md` — all
present in this checkout), (3) cite the cross-step closure log as "the `handoff/merge-2026-09-08`
branch's `handoff-2026-09-08/BUILD-LOG.md`," explicitly naming the branch rather than a bare path
that would resolve, misleadingly, to this checkout's own incomplete local copy (D3/D4, review round
1 S6-002), and (4) not overstate the merge initiative's current state beyond what Steps 1-5's
actual closing commits established (no "migration complete," no "consumer cutover," no claim
`rusty_multimodal_db` is being phased out on any timeline, and no §7 non-goal marked superseded
beyond the specific, named items D2 lists).
