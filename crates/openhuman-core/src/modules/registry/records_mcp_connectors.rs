//! Registry records for the `tinymcp` and `tinyconnectors` modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The `tinymcp` module: the Model Context Protocol client.
///
/// Owns both transports (Streamable HTTP and a subprocess over stdio), the
/// statically declared server set a host puts in its own configuration, the
/// dynamic registry of user-installed servers with its SQLite store, the
/// reconnect supervisor, the browser sign-in flow, and the write-audit log.
///
/// Lazy, because dialing an MCP server is something most sessions never do: a
/// host with no installed servers and no configured ones would otherwise pay a
/// download and a `dlopen` for a capability it never reaches. That differs from
/// the module's own `lazy = false` export hint, which speaks for a host whose
/// servers should be connected the moment it comes up — this host decides when
/// that moment is, and does so on the first ask.
///
/// **What stays out of the module is host policy**, and the split is the same
/// one the contract's own documentation draws: the prompt-injection scan over
/// remote tool definitions, the `mcp_clients` RPC surface, the
/// agent-facing tools, and the proxy *scoping* decision all belong to this
/// application's threat model, not to a protocol client. `tinymcp-bus` carries
/// the vocabulary; this table says which bytes may speak it.
pub(crate) const TINYMCP: ModuleRecord = ModuleRecord {
    id: "tinymcp",
    description: "Model Context Protocol client: transports, registry, and the write-audit log",
    bus_name: "ai.tinyhumans.tinymcp.Mcp",
    object_path: "/ai/tinyhumans/tinymcp/Mcp",
    version: "0.3.3",
    release_url: "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.3.3",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymcp-0.3.3-ubuntu-24.04-x86_64.tar.gz",
            sha256: "7c49adafbdeed02b5555d35aafb9e0b1d6a27ea6586aa24518d19d58dc21079b",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymcp-0.3.3-ubuntu-24.04-arm64.tar.gz",
            sha256: "996637662a3406681eae477d2d333312894a9709c83fa179a7aacfe068ba8be0",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymcp-0.3.3-ubuntu-22.04-x86_64.tar.gz",
            sha256: "82da931e0e5e9e82feb189042ac130fa37f3ae5ffec8d935ddd89b41859ef55c",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymcp-0.3.3-ubuntu-22.04-arm64.tar.gz",
            sha256: "a69a995808c0098ce80ef9a38d590b829e83abd9370410807cf482f0ae77097a",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymcp-0.3.3-macos-26-arm64.tar.gz",
            sha256: "5fefcc5a3ec34dbd4bcee70615fff456be9cf28a74c1d06d07f810137facf5d7",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymcp-0.3.3-macos-26-x86_64.tar.gz",
            sha256: "c04b9ec6746f3317c695cde897a168101a6ea9fe1fe9b5e08a85aba3a2429baa",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymcp-0.3.3-macos-15-arm64.tar.gz",
            sha256: "67172f3d6be08a57ee5e83039d8100be60132e1bbf32d027f6b2eb150eb59b3f",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymcp-0.3.3-macos-15-x86_64.tar.gz",
            sha256: "6eac1dd3e55a0e6ad5341c30182f24fd5ea0bba7cce07ecec17c607aea40882c",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymcp-0.3.3-windows-2025-x86_64.zip",
            sha256: "e9b7dd357deddc3e659eccac1eeb9a9bc018f083b3120de2bb36027b7824cd9b",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymcp-0.3.3-windows-2022-x86_64.zip",
            sha256: "961cf051976f8f650270764b3ad11ced8067d1a0f953c091ed4b9da524e18051",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymcp-0.3.3-windows-11-arm64.zip",
            sha256: "312eda791a4cc3fa9e8ef46415c8471ba280aea63534718486a92ee764debe61",
        },
    ],
    load: LoadPolicy::Lazy,
};

pub(crate) const TINYCONNECTORS: ModuleRecord = ModuleRecord {
    id: "tinyconnectors",
    description: "OAuth connector integrations: accounts, actions, triggers, and record sync",
    bus_name: "ai.tinyhumans.connectors.Composio",
    object_path: "/ai/tinyhumans/connectors/Composio",
    version: "0.10.3",
    release_url: "https://github.com/tinyhumansai/tinyconnectors/releases/tag/v0.10.3",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyconnectors-0.10.3-ubuntu-24.04-x86_64.tar.gz",
            sha256: "328e34c81a5f369297d050626a10dfc704d14d05248d4a85728f85736a4e872d",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyconnectors-0.10.3-ubuntu-24.04-arm64.tar.gz",
            sha256: "2a013331c8b4d25e134a0ad4427b1dcdf0fdd540e0c3d3c83c5a4e810846c08a",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyconnectors-0.10.3-ubuntu-22.04-x86_64.tar.gz",
            sha256: "40110ff86192e40ea4b262e99380df1c744da7f4ec6475f6d9ad90e73306daee",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyconnectors-0.10.3-ubuntu-22.04-arm64.tar.gz",
            sha256: "dee37022ec91841f6cb00e994a29cc4500b512926893f253aab6833d9e69bae4",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyconnectors-0.10.3-macos-26-arm64.tar.gz",
            sha256: "53efa700c50d2fef9a16b2a4bee5eda60ff25801cf3ca726322e037a503715bf",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyconnectors-0.10.3-macos-26-x86_64.tar.gz",
            sha256: "0252e6ab54c94d802d70fe9c8cda8990d236bff8fff60cc04e675ffa10d6e240",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyconnectors-0.10.3-macos-15-arm64.tar.gz",
            sha256: "94dbb97d0760610dc9cd4b21d46bbdd9a103aee391b38c439a4f811f9539f0dd",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyconnectors-0.10.3-macos-15-x86_64.tar.gz",
            sha256: "1f9c28259d6bd35e3c511c0ab5245a69a3a1bd14924def0bd87ea2669d85af95",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyconnectors-0.10.3-windows-2025-x86_64.zip",
            sha256: "2d87e7d4e3990e47fa9d61bc4b13b22e5924e39bfccb19fd4275986b828c2962",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyconnectors-0.10.3-windows-2022-x86_64.zip",
            sha256: "29c0f82f6a3a8ffd62c55eef7528ff69887c5029b8d9f788bcb74ae8a56e8968",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyconnectors-0.10.3-windows-11-arm64.zip",
            sha256: "dbaf94cfd39d5decb1f5bb16a442d4b9b6ebce5c4eea3ee21c68ae480f57235a",
        },
    ],
    // Lazy: a user with no connected accounts should not pay to load it, and
    // most sessions never touch a connector. Safe even signed out — the module
    // loads without configuration and still answers the capability members.
    load: LoadPolicy::Lazy,
};
