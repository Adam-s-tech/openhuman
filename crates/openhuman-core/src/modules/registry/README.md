# modules/registry

The compiled-in table of every native module this build knows how to load.
The directory backs a single file, [`modules/registry.rs`](../registry.rs),
which is the only place allowed to say what counts as a legitimate module
artifact.

## Why a compiled-in table

A loaded module is trusted native code sharing this process's address space,
privileges, and crash domain, and tinybus never unloads it once admitted. So
which modules may load, and which bytes count as legitimate, are build-time
decisions rather than something a server can hand down at runtime. There is
no module marketplace and no RPC method to register a new one: adding a
module here is a source change, reviewed like any other.

## Contents

Records are split one file per module family, by relatedness rather than by
line count:

| File | Modules |
| --- | --- |
| `records_desktop.rs` | `tinydesktop` |
| `records_browser.rs` | `tinybrowser` |
| `records_search.rs` | `tinysearch` |
| `records_docs_wallet.rs` | `tinydocs`, `tinywallet` |
| `records_memory_juice.rs` | `tinymemory`, `tinyjuice` |
| `records_runtime.rs` | `tinyruntime`, `tinyruntime-nodejs`, `tinyruntime-python` |
| `records_mcp_connectors.rs` | `tinymcp`, `tinyconnectors` |
| `records_extra.rs` | `tinybox`, `tinychannels`, `tinyhosts` |
| `records_voice.rs` | `tinyvoice` |

`registry.rs` declares each file as a submodule, re-exports its `ModuleRecord`
constants, and wires them into `ALL` (the full list) and `find(id)` (lookup by
registry id). There is no `registry/mod.rs`: the parent file's name doubles as
the module declaration point, and the tests live beside it in
`registry_tests.rs`.

## Key types

Each record is a [`ModuleRecord`](../types.rs): a registry id, the release tag
it is pinned to, and a per-platform digest table (`ubuntu-24.04-x86_64`,
`macos-15-arm64`, and so on). `records_desktop.rs`'s `TINYDESKTOP` is
representative of the shape every other file repeats.

## How it fits

`registry::find(id)` is the first step of the loading pipeline described in
[`modules/README.md`](../README.md#loading-pipeline): a resolution request
looks up the record here before anything is downloaded, cached, or admitted.
The digests pinned in these files are the host's half of a two-sided check;
tinybus fetches the release's own `checksum.toml` and the two must agree.
Digests are copied verbatim from a published release, never recomputed from a
local build, since the point is to pin what the release publishes rather than
to agree with whatever happens to be on disk.

## Where next

- [`modules/README.md`](../README.md) for the loading pipeline these records
  feed, and the security invariants around admitting a module.
- [`modules/types.rs`](../types.rs) for `ModuleRecord`, `PlatformAsset`, and
  the other shared types.
- [`modules/platform.rs`](../platform.rs) for how a host's platform key is
  matched against a record's per-platform digests.
