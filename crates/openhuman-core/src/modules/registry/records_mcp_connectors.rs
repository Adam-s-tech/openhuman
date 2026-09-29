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
    version: "0.3.5",
    release_url: "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.3.5",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymcp-0.3.5-ubuntu-24.04-x86_64.tar.gz",
            sha256: "6ff682a17af79c736281cfafc3332c2120aab3570f16c60a7fd5d266f041e483",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymcp-0.3.5-ubuntu-24.04-arm64.tar.gz",
            sha256: "fdaf771b1deafe8f44f890bc1e8748f365782e0ca664872e176e78a0bedc715a",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymcp-0.3.5-ubuntu-22.04-x86_64.tar.gz",
            sha256: "922e3b5af7ac0f72c240914b7edf45748e65b3376a937a470f595dbd6f578668",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymcp-0.3.5-ubuntu-22.04-arm64.tar.gz",
            sha256: "5ef11dd5c27640207c7c9e7046e44f49a9f06817dfbe55a07e2b58a7118b3a31",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymcp-0.3.5-macos-26-arm64.tar.gz",
            sha256: "e01827c178b2e0dd807eedbe80eced7c69e8e6460706af69d579609a983588d5",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymcp-0.3.5-macos-26-x86_64.tar.gz",
            sha256: "87787d4446202a6149455949b4f3203cd23145aebd45f2e407f500b724734c42",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymcp-0.3.5-macos-15-arm64.tar.gz",
            sha256: "1dd1026f1b83ae041003578010e07fd380468b3c2727529e3ec968951f73003d",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymcp-0.3.5-macos-15-x86_64.tar.gz",
            sha256: "8a1466f4807b793edfd7e55cd66c7b7aa1cb5ec45e7855b3078fbf60c58f7f49",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymcp-0.3.5-windows-2025-x86_64.zip",
            sha256: "f7f8e525eea23a09be5f03f4c338804a096865ba342529191a716f263fd60918",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymcp-0.3.5-windows-2022-x86_64.zip",
            sha256: "c6075374858e97d50ad8afee09d93d3603779238dc246d1db47cdcf31306ec7b",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymcp-0.3.5-windows-11-arm64.zip",
            sha256: "8984e02e1679377411b1b2ad0f5e21cffd05054743288b23dc37c94ffb7560de",
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
