# Host disposition on Step 4b-ii inspection 1 (fresh Claude CLI, snapshot `78e7839e…8d3fdb6`)

**Verdict: REVISE, 1 medium finding, accepted.**

F1-DESCRIBE-RELATIONS-WILDCARD-DROPPED (medium): confirmed by direct source read.
`MemoryStore::describe_relations()` (`uc-facade/src/memory.rs:316-322`) and
`EntityStore::describe_relations()` (`uc-facade/src/entity.rs:285-294`) both hand-write a custom
override that lists only the named-label relation descriptors (`mentions` for Memory; each label
in `known_labels` for Entity), omitting the generic `RelationDescriptor { kind:
JoinRelation::Neighbors(None), .. }` wildcard entry that `uc-protocol::query::
default_relation_descriptors` (the trait's own default, `query.rs:89-107`) already supplies for
free whenever `schema.relations.neighbors` is true — which both `describe()` implementations
already declare. Nothing in the work order asked for a custom override; the only stated
requirement was `target_table: None` everywhere, which the trait default already produces
unconditionally. The custom overrides therefore silently drop working capability: a
`Request::Join` naming the unlabeled `Neighbors(None)` relation against either table is rejected
`Malformed` by `validate_join`'s exact-match lookup, even though `Store::neighbors()` itself is
fully implemented and correct for both. This was untested (none of the eleven new facade tests
call `Request::Join`) and undisclosed in the implementation report.

**Fix:** delete both custom `describe_relations()` overrides entirely and rely on the trait
default (`default_relation_descriptors(&self.describe(), self.list_relation_kinds())`), which
already produces `target_table: None` for every entry and is simpler than the code it replaces.
Add one test per domain exercising `Request::Join` with an unlabeled `Neighbors(None)` relation
spec against Memory and against Entity, to lock in the restored behavior.
