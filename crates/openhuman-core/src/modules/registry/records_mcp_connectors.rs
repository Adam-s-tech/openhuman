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
    version: "0.3.4",
    release_url: "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.3.4",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymcp-0.3.4-ubuntu-24.04-x86_64.tar.gz",
            sha256: "ac01fd66a77760dc89b7f66a15cf6b5a7821ded708306b0fb5d359194d394ac4",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymcp-0.3.4-ubuntu-24.04-arm64.tar.gz",
            sha256: "d49afd1b9e9d7ea78370b20db8462e0b98145fdaf0de5e028376f34a3861fdb8",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymcp-0.3.4-ubuntu-22.04-x86_64.tar.gz",
            sha256: "fbc17856d38152b3fc543d23db714a03cc7cec02409d66c52d79360fe115d949",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymcp-0.3.4-ubuntu-22.04-arm64.tar.gz",
            sha256: "3d6cd96e5269a74560b2f2677f9c395f5333101846c794d08d7bbd1a31dad331",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymcp-0.3.4-macos-26-arm64.tar.gz",
            sha256: "c3679947164247ac20c08422537294d69babf5ffc505f9e45b9ac182a76ad991",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymcp-0.3.4-macos-26-x86_64.tar.gz",
            sha256: "79c6c2b4d0f2037d2758915a34c3ed77f032fac1312c0db730502510ced9aca5",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymcp-0.3.4-macos-15-arm64.tar.gz",
            sha256: "196b75978a5dc2b5dd709e4b98736fece182e7f60e010f734b4e4550537824c7",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymcp-0.3.4-macos-15-x86_64.tar.gz",
            sha256: "c7a354f19f540a632e960fe30a8bb7e0a37538a95aa8fbe869d4d06d423d4ee2",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymcp-0.3.4-windows-2025-x86_64.zip",
            sha256: "460de6ad7cb7a697e63bf7e479977023059da58b25824d45c45aed158b01394b",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymcp-0.3.4-windows-2022-x86_64.zip",
            sha256: "0e3ee1f016cf89c06fec3bdb69e0bf9d283cfffea279f642ba46d05c9afddc95",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymcp-0.3.4-windows-11-arm64.zip",
            sha256: "1d3f5b89a178b562d0d9816b5bcadb16e8d2878c55cab8d0c7cbcd4147fbd13e",
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
