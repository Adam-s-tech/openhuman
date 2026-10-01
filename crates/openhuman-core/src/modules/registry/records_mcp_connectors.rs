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
    version: "0.11.0",
    release_url: "https://github.com/tinyhumansai/tinyconnectors/releases/tag/v0.11.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyconnectors-0.11.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "e57a7e20ee7071695eac7537c55487e7bc41713c119b0931c4793940f4039b61",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyconnectors-0.11.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "0baf09c899870840f3cae79f68348dd11aa999a29e0d1c2fc73cbf3ef1315013",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyconnectors-0.11.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "72d9885506329465cc8c4b544304f32882d9d2a5c73ec2e21992a8563e4e3b30",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyconnectors-0.11.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "a9a038c9d3d37da612eea6ec7ca9df7ab8a5922ec3152ab3ee57a2da2f3d9a14",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyconnectors-0.11.0-macos-26-arm64.tar.gz",
            sha256: "904d34c761d080ba78e23deb7c2347b3b648e891f54dec4d0f48f79831bec760",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyconnectors-0.11.0-macos-26-x86_64.tar.gz",
            sha256: "63e2c4f251bc7805dd1eb38bdc1eb2386ec1b5141a0a8c4fd8347620ec86a003",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyconnectors-0.11.0-macos-15-arm64.tar.gz",
            sha256: "a693e2c49281bc1cee82c9999434282bfe9eb8575e1ae05bf8a634e63c299676",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyconnectors-0.11.0-macos-15-x86_64.tar.gz",
            sha256: "a5ab99e9356d511212ce34b5a7d203fde90b0a789747e8a7b95b2b6ed5dd2183",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyconnectors-0.11.0-windows-2025-x86_64.zip",
            sha256: "1d9b4b9e89c207f2154292bc0e3447f2a6709110b4fc551c0fef2561e0e8f585",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyconnectors-0.11.0-windows-2022-x86_64.zip",
            sha256: "2cf82038c10db0225b27d4ff6ae5c962b5406eb9d35f17b7ea615e73fa2e7fcb",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyconnectors-0.11.0-windows-11-arm64.zip",
            sha256: "e234763ed750c0c9b252990825a7339beae375d5434ca7a5ae54615f24224888",
        },
    ],
    // Lazy: a user with no connected accounts should not pay to load it, and
    // most sessions never touch a connector. Safe even signed out — the module
    // loads without configuration and still answers the capability members.
    load: LoadPolicy::Lazy,
};
