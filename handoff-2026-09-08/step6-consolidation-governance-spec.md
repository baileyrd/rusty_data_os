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
- **D2 — `docs/VISION.md` gets a new closing section, "9. A parallel, disclosed initiative
  (2026-09-09)."** States: the merge plan (`data-os-multimodal-merge-plan-2026-09-08.md`) is a
  separate, owner-authorized initiative to build a general-purpose database intended to replace
  SQLite/PostgreSQL/DuckDB workloads, using this repository as "the eventual home of the combined
  engine" (the plan's own words); it proceeds under `experiments/unified-commitment/` in parallel
  with, not instead of, the EXP-0001 exploratory-history research this document otherwise
  describes; §7's non-goals and §6's scope progression describe the EXP-0001 thread specifically
  and do not constrain the merge initiative, which the owner has separately authorized to skip
  ahead under named, bounded, per-phase exceptions (cites the Phase 7 precedent). Explicitly does
  not change §§1-8's content, does not retract any EXP-0001 non-goal, and does not claim the merge
  initiative has met any of §5's success criteria.
- **D3 — `AGENTS.md` gets a new numbered section (after the existing authority-order list) naming
  `experiments/unified-commitment/` as the merge initiative's home and `rusty_multimodal_db` as the
  active source repository it ports from.** States plainly: `rusty_multimodal_db` is not archived
  and not deprecated — it remains the authoritative running system for its own consumers until a
  real migration (not a proof) actually lands per merge-plan Step 5's remaining, undone final
  sub-step; new development *for the merge's goal specifically* happens in
  `experiments/unified-commitment/`, not by adding capability to `rusty_multimodal_db` or by
  starting a third, competing implementation. Cites `handoff-2026-09-08/BUILD-LOG.md` as the
  detailed evidence trail for every step's review/build/inspection history.
- **D4 — `docs/ARCHITECTURE.md` gets a short pointer, not a duplicate description.** A brief new
  section noting `experiments/unified-commitment/`'s existence and pointing to
  `docs/experiments/EXP-0005-protocol-facade.md` and the `handoff-2026-09-08/` specs for its actual
  architecture, rather than re-describing crate boundaries already documented there (avoiding two
  descriptions of the same system drifting apart).
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
`docs/RESEARCH-ROADMAP.md`'s Phase 7 section, `handoff-2026-09-08/BUILD-LOG.md`,
`docs/experiments/EXP-0005-protocol-facade.md`), and (3) not overstate the merge initiative's
current state beyond what Steps 1-5's actual closing commits established (no "migration complete,"
no "consumer cutover," no claim `rusty_multimodal_db` is being phased out on any timeline).
