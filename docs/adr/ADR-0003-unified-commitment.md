# ADR-0003 — Bounded unified commitment

**Status:** Proposed
**Date:** 2026-09-09

## Context and authority

The owner’s [merge plan, step 3](../plans/data-os-multimodal-merge-plan-2026-09-08.md)
requires one ordered recovery authority for the reproduced journal/lifecycle failures.
This records the frozen work-order choices, not an evidence-backed production graduation.

The host accepted the payload-magic rejection in disposition P1 of the
[EXP-0003 host dispositions](../experiments/EXP-0003/HOST-DISPOSITIONS.md) on 2026-09-09,
under the owner's merge-plan step 3 authority and approved work order; the same record
carries disposition Q3 (explicit `Log::create` versus `Log::open`) and the residuals carried
at close.
This is the recorded bounded deviation from the payload-agnostic contract; the UUID/envelope
residual and codec follow-on below remain unchanged.

## Decision

One transaction is one RF1 event. Reuse the frozen R5 codec and lifecycle unchanged,
CRC-32C on every eligible frame, and its four D2 synchronization placements with one
realtime sample after the pre-finalization sync. Offer D1 separately; D3 is deferred.
Validation and payload application are caller supplied inside a single-writer core.
Publish one immutable state snapshot only after the requested boundary succeeds.
Memory CMM2 changes carry incarnation; deletion/recreation cannot inherit stale changes.
Retain every binding for the life of its history; normalized request bytes govern retry,
including requested durability. The `UCR1` header is the five bytes `55 43 52 31 00` and the
`UCE1` envelope header is the five bytes `55 43 45 31 00` (the four ASCII letters followed by
NUL); these bytes are frozen here so no rendering of `\0` is ambiguous. uc1 checkpoints cache state/retry resolution at an exact
commit position and SHA-256 prefix, with a CRC-32C and file/parent synchronization.
History is synchronized before checkpoint creation at D1 and D2.
Checkpoints do not authorize history truncation. Use an OS-released advisory owner lock.

New payloads containing the literal byte sequence `RDE1` are rejected before append with
`Invalid("payload contains the RF1 magic")`; old normalized requests containing it fail
recovery as corrupt. The frozen scanner searches the incomplete frame body for this magic
and otherwise can misclassify a torn payload as interior damage. Literal payload magic is
unreachable for CMM2's hex-encoded values and fixed operation syntax.
This restriction does not guarantee recovery of every possible torn RF1 frame: a valid
caller request UUID can contain `RDE1` at frame offset 32 (body offset zero) (tested with a legitimately encoded
Binding truncated one byte before its end). That tail remains corrupt or undecidable.
Binary envelope fields can also contain the sequence. The scanner searches after its
32-byte header, so the header CRC alone does not trigger this heuristic. No additional
identity restrictions or frozen-codec changes are silently imposed.

Auto request IDs use domain byte `0x41`, separate from the explicit test/caller convention
`0x52` and event domain `0x45`; callers must reserve `0x41` for the adapter. They are local
conveniences, not stable request identities across a torn-tail reopen: an ordinal removed
with a partial binding can be reused. Retry-sensitive callers must supply stable IDs.

## Evidence and consequences

[EXP-0003](../experiments/EXP-0003-unified-commitment.md) defines correctness and measurement
before implementation. Three payload copies plus framing are an explicit measured cost.
Platform durability evidence, architecture promotion, retention and production remain pending.

## Alternatives considered

The host excludes a separate application journal, a new RF1 record kind, D3 atomic groups,
expiry and compaction from this increment. Payload references require a later contract change.

## Follow-up

Independent review, the host's descriptive series, and separate platform durability research.
Arbitrary payload support and unambiguous torn-frame classification need a codec-level follow-on.
