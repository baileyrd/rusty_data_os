# Pinned legacy Memory runner

This standalone workspace depends on reviewed legacy Step 1 part A commit
`abda0a7e94a9727e410001e9724edc741f6e0d31`, with `server` enabled only to call
`MemoryConnectionStore` in process. It starts no network server.

See the [shared driver instructions](../convergence-memory/README.md) and
[EXP-0002](../../docs/experiments/EXP-0002-convergence-memory.md).
Candidate: D1 ordinary writes, no fsync, no crash-survival claim.
Legacy: journal off, runtime insert/replace/delete log `sync_data`, mmap count
updates without per-op flush; non-equivalent native durability, no power-loss claim.

CI fetches the locked dependency graph with network first, then validates offline.
The Windows sandbox uses the host-prefetched cache and `fetch --locked --offline`.
No additional dependencies or local modifications to the pinned engine are used.

The workflow assumes GitHub Actions can reach and fetch
`github.com/baileyrd/rusty_multimodal_db` and the transitive locked Git source
`github.com/Rusty-Mill/rusty_mill` (platform, platform-windows, rusty_tls and
winargv). Repository visibility was not verified from the host network. If either
is private, configure a token with access to both repositories and authenticated Git credentials before fetch, for example using
`CARGO_NET_GIT_FETCH_WITH_CLI=true` and a `GITHUB_TOKEN`-authenticated Git credential
when that token has the required access. The default `GITHUB_TOKEN` may not have
cross-repository access; use an appropriately scoped token if needed. Do not put
credentials in manifests, URLs, source, or logs. This workflow has not been
executed remotely.

Retention defaults to metadata and raw result files, with stores removed after
validation. Append `--retain-store` after `single`, `pipelined`, or `atomic` to
retain stores and the trace copy for a forensic run. The source tree is never
copied. Follow the shared driver's committed-evidence subset.
