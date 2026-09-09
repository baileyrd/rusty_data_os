# HYP-0004 — Entity and Relation domains

Status: Open.

The unified core's recovery, incarnation and checkpoint guarantees generalize to Entity
and Relation without any core change. A mismatch against either domain's independent
oracle or a lost/duplicated commit under the existing injected-fault model falsifies this
for that domain. [EXP-0004](../experiments/EXP-0004-entity-relation-domains.md) fixes the
correctness method before implementation. No cross-domain atomicity or performance claim.
