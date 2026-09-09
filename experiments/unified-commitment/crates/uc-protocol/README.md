# uc-protocol — EXP-0005 / Step 4b-i

A std-only experimental protocol-22 payload codec, framing, Store/dispatch and generic
Read + Write connection loop. [Method and authority](../../../../docs/experiments/EXP-0005-protocol-facade.md).

Construct one Registry with ordered `(name, Arc<dyn Store>)` entries and a primary index;
share it by reference or clone across all calls to handle_connection. Registry names must
match Store::table_name, be nonempty and unique. Metadata and foreign targets must remain
stable while registered. Reconstructing separate registries over the same mutable stores
would create separate coordination boundaries; use Registry::clone instead.

Store implementations own their synchronization and transaction atomicity. They must check
the read set and every write precondition under the same exclusive section as commit.
write_batch_checked defaults to refusing nonempty atomic batches. Bare dispatch/write_batch
are table-local APIs; served foreign checks and detach cascades require handle_connection
and the shared registry. No actual domain adapter is provided in this tranche.

The wire codec uses little-endian fixed integers, u32 enum tags, byte Option/bool tags,
u64 sequence/string lengths and length-16-prefixed raw UUID bytes. The frame cap is 16 MiB.
Malformed payloads get an Err/Malformed response and leave the connection open; incomplete
frames return an I/O error. Clean disconnect discards any session. No authentication is
enforced, and no older-version content rewriting occurs. BeginWith flag availability is
checked at versions 5/6/7; other behavior follows D5 regardless of negotiation.

Tests copy the authorized 66-vector corpus and host expected-values file. The checked-in
expected_cases.rs is a literal transcription made by fixtures/generate_expected.py without
reading or decoding wire bytes. Rust tests pair every exact host line with its literal value,
assert equality of the complete decoded Request/Response and then byte-exact re-encoding.
The test-only map Store and duplex pipes live in tests/support. The deterministic shared
registry test also demonstrates the dangling-edge race with the mutex removed as a negative
control. No live sockets, cross-repo dependencies, durability or performance claims.
