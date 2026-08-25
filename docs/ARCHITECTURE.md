# Conceptual Architecture

**Status:** exploratory. This document describes the architecture currently being investigated; it is not a final specification.

## 1. Architectural premise

Rusty Data OS separates four ideas that traditional database implementations often combine:

1. **mutation intent** — what a caller wants to change;
2. **canonical history** — the ordered immutable record of accepted changes;
3. **execution state** — the active in-memory structures used to answer and process work;
4. **materialized representations** — optimized physical projections derived from canonical history.

## 2. Conceptual flow

```text
+-------------------+
| Caller / Command  |
+---------+---------+
          |
          v
+-------------------+
| Validation /      |
| Command Handling  |
+---------+---------+
          |
          v
+-------------------+
| Event Construction|
+---------+---------+
          |
          v
+-----------------------------+
| Canonical Event Sequencing  |
+----------+------------------+
           |
     +-----+----------------------------+
     |                                  |
     v                                  v
+----------------------+      +-----------------------+
| Active In-Memory     |      | Durable Event History |
| Materialization      |      | / Persistence Path    |
+----------+-----------+      +-----------+-----------+
           |                              |
           |                              |
           +--------------+---------------+
                          |
                          v
                +---------------------+
                | Materialization Bus |
                +----+----+----+------+ 
                     |    |    |
                     |    |    +------> Vector / graph / future
                     |    +-----------> Columnar / analytic
                     +----------------> Row / index / snapshot
```

The exact ordering between in-memory application and durable acknowledgement is deliberately unresolved. Multiple durability modes may eventually exist. Early experiments must measure and define the consequences.

## 3. Candidate components

### 3.1 Command interface

Accepts mutation intent. The command representation should not be mistaken for the canonical event representation.

Responsibilities may include:

- validation;
- authorization in later layers;
- precondition checks;
- deterministic conversion to one or more events.

### 3.2 Event constructor

Creates immutable events containing enough information to preserve meaning and support deterministic replay.

Event identity, ordering, schema/versioning, checksums, timestamps, metadata, and encoding remain research topics.

### 3.3 Sequencer / append path

Provides the ordering and append semantics required by the selected correctness model.

This is the focus of the first experiment because it establishes the basic ingest cost before additional database features distort measurement.

### 3.4 In-memory execution state

Maintains active state used by commands and reads.

Open research questions include:

- structure-of-arrays versus array-of-structures layouts;
- hash/index strategies;
- copy-on-write versus in-place mutation;
- lock-based versus lock-free or sharded concurrency;
- snapshot interaction;
- memory reclamation;
- replay/rebuild speed;
- NUMA behavior at larger scale.

None should be selected before isolated measurement.

### 3.5 Durable event history

Preserves accepted events across process and machine failures according to declared durability semantics.

Candidate mechanisms may include buffered files, append-only segments, direct I/O, memory mapping, synchronous writes, group commit, checksummed records, and storage-device-specific optimizations.

These are candidates, not commitments.

### 3.6 Materialization bus

A conceptual boundary through which independent materializers consume committed history or validated event streams.

Potential derived forms include:

- row-oriented structures;
- column-oriented structures;
- vector indexes;
- graph adjacency structures;
- secondary indexes;
- snapshots/checkpoints;
- archival formats;
- replication/change streams.

The bus need not initially be a generalized plugin framework. Early implementations should be direct and minimal.

## 4. Durability pipeline

A likely performance opportunity is pipelining:

```text
foreground command processing
        |
        +--> append / acknowledgement path
        |
        +--> update active memory state
        |
        +--> background batching / persistence
        |
        +--> background secondary materialization
```

However, a pipeline is only safe when the acknowledgement point is explicit. "Asynchronous" is not synonymous with "durable."

The project must distinguish at least:

- visibility — when another reader can observe the change;
- commit — when the system considers the mutation logically accepted;
- durability — what failures the accepted mutation can survive;
- materialization freshness — when secondary views incorporate the change.

## 5. Recovery model

The candidate recovery model is:

```text
latest validated checkpoint (optional)
            +
canonical events after checkpoint
            |
            v
reconstructed in-memory state
```

The project should benchmark recovery and replay as first-class performance dimensions rather than treating startup time as incidental.

## 6. Temporal model

Canonical history creates opportunities for:

- point-in-time state reconstruction;
- deterministic replay;
- historical queries;
- alternate materializations built from the same history;
- forensic debugging;
- migration between physical representations.

Temporal features are architectural consequences to investigate, not guaranteed product features yet.

## 7. Server boundary

The eventual server should be an adapter around a proven engine, not the definition of the engine.

```text
client protocol / network / auth
              |
              v
       server adapter
              |
              v
      Rusty Data OS core
```

This preserves the ability to embed the engine locally and keeps network latency from contaminating early core benchmarks.

## 8. Architectural unknowns

Major unresolved questions include:

- whether events should remain the sole canonical durable form;
- event granularity and encoding;
- sequencing and concurrency model;
- acknowledgement/durability modes;
- checkpoint strategy;
- optimal in-memory representation(s);
- whether different workloads require multiple simultaneous active memory layouts;
- backpressure between ingest and materializers;
- crash consistency across event history and checkpoints;
- schema evolution;
- garbage collection / history retention;
- transaction semantics;
- query model;
- distributed behavior.

These are intentionally unresolved. The roadmap exists to resolve them with evidence.
