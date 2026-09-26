# memory/driver

The memory-driver namespace. It exists to hold a place in the module tree, not
to hold code: `mod.rs` is five lines, and none of them define a driver.

## Why it is nearly empty

Memory execution is provided by the compiled TinyMemory TinyBus module, the
same native `cdylib` that backs [`crate::modules::memory`](../../modules/memory/README.md).
There is intentionally no in-process engine driver here. Earlier revisions of
this core linked `tinymemory-core` directly and picked between embedded
engines in process; that class of driver is gone, and this directory is what
is left of the namespace it used to occupy.

The host-side contract and binding live one level up, in
[`memory::api`](../api.rs) (the bus vocabulary, re-exported from
`tinymemory_api`) and [`memory::binding`](../binding.rs) (the
workspace-keyed `for_config` lookup that resolves which driver backs a given
workspace, reached through `CoreContext::memory_binding`). Both are described
in the top-level [`memory/README.md`](../README.md#wiring).

## Pluggable engines

Memory is one of this project's pluggable-engine subsystems: the driver a
workspace binds to is chosen by config (`[subsystems.memory] driver`, or
`OPENHUMAN_MEMORY_DRIVER`), not compiled in. TinyCortex is the engine embedded
by default; remote engines such as Supermemory, Mem0, Cognee, CortexDB,
AgentMemory, and LivingBrain are reached the same way, through the
`tinymemory-api` contract rather than a driver-specific code path. See
[engines.md](../../../../../gitbooks/developing/engines.md) for how engine
selection works across subsystems.

## Where next

- [`memory/README.md`](../README.md) for the full split between this host and
  the extracted `tinymemory-core` engine crate.
- [`memory/binding.rs`](../binding.rs) for the workspace-to-driver cache and
  its fail-closed rules (`docs/specs/kernel.md` §3.1, §3.4, §3.7).
- [`modules/memory`](../../modules/memory/README.md) for the module-loading
  side: how the TinyMemory `cdylib` is admitted and reached over the bus.
