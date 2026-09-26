# memory/driver

A one-file namespace that documents where the memory driver seam lives, and
why there is no engine code in this directory.

## Why this folder is nearly empty

`mod.rs` is a module-level doc comment and nothing else. Memory execution runs
as a compiled TinyMemory TinyBus module, so there is no in-process engine
driver to put here. The host-side half of the driver contract is split across
two files that already existed before this folder did:

- [`memory/api.rs`](../api.rs): the tinybus module contract, a re-export of
  `tinymemory_api` naming only what actually crosses the bus between this host
  and the TinyMemory module.
- [`memory/binding.rs`](../binding.rs): `admit()`, which reads
  `MemorySubsystemConfig` and decides which driver actually binds for a
  workspace.

## The pluggable-engine story, as implemented today

This is where OpenHuman's "pluggable memory engines" pillar lives, and the
honest version is narrower than the config surface suggests. `tinymemory-api`
(vendored at `vendor/tinymemory/`) defines a driver-neutral `MemoryProvider`
trait and ships adapter crates for six remote engines under
`vendor/tinymemory/crates/tinymemory-remote/`: Supermemory, Mem0, Cognee,
CortexDB, AgentMemory, and LivingBrain.

`binding::admit` does not wire any of those in yet. It only accepts the
compiled TinyMemory module (registry id `tinymemory`, with `tinycortex` kept
as a legacy config alias) or the `null` fallback provider
(`tinymemory_api::null::NullMemoryProvider`). A driver id configured under
`[subsystems.memory.drivers.<id>]` other than those is refused with "external
driver transport is not implemented yet." In practice, TinyCortex (the
compiled TinyMemory module) is the memory engine every OpenHuman install runs.

The `[subsystems.memory]` config block already parses and persists ahead of
that wiring:

```toml
[subsystems.memory]
driver = "tinycortex"
```

`OPENHUMAN_MEMORY_DRIVER` overrides `driver` at the environment level (see
`config/schema/load/env_overlay/subsystems_update.rs`). Both exist today so
the shape is stable once a non-default driver actually switches engines;
neither one does that yet.

## Where next

- `gitbooks/developing/engines.md` for the cross-cutting pluggable-engines
  page (LLM providers, embeddings, memory, web search) and the current state
  of this seam in one place.
- [`memory/binding_admission_tests.rs`](../binding_admission_tests.rs) for the
  admission behavior this README describes.
- [`memory/README.md`](../README.md) for the memory domain as a whole.
