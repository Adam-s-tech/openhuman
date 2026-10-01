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
    version: "0.3.6",
    release_url: "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.3.6",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymcp-0.3.6-ubuntu-24.04-x86_64.tar.gz",
            sha256: "624541b4e0e333b0eeae67ed986ad2b3fd6f0cbb3da513b979ae62f80528018d",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymcp-0.3.6-ubuntu-24.04-arm64.tar.gz",
            sha256: "0e6ab378e2d4c4c870e048355ada6041b93ec5249aac18fb58a8d1d8ff243cd3",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymcp-0.3.6-ubuntu-22.04-x86_64.tar.gz",
            sha256: "103e10496e355d0c38f33a825f229510939e49451a5671350eb88084e9695368",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymcp-0.3.6-ubuntu-22.04-arm64.tar.gz",
            sha256: "e8210a0dbe55a28206c9ca6822a2d420b25597aeef78c84d8c0d250555899aa5",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymcp-0.3.6-macos-26-arm64.tar.gz",
            sha256: "f0c5c711600bb4d036689de3a990876b03a56298420756834e9dd095fb403cfa",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymcp-0.3.6-macos-26-x86_64.tar.gz",
            sha256: "441ae8ebdb5f14168c0090d0a87b63d4951ebcc390f93c988fe11bf4e72e20eb",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymcp-0.3.6-macos-15-arm64.tar.gz",
            sha256: "c7399a222aaa823b61bd5240130337bc8f9fa58951a2c6cf29680e22ee8fba02",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymcp-0.3.6-macos-15-x86_64.tar.gz",
            sha256: "042173605ca634c76e17d5fc002b3beb91e228a871723cc6dc6e5b5895954349",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymcp-0.3.6-windows-2025-x86_64.zip",
            sha256: "4c64f10f12d3d9b5648c85832a8e0e7d7949dcffbd232fb9446bf25272877328",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymcp-0.3.6-windows-2022-x86_64.zip",
            sha256: "65b1c0a382d7a75749f64a2739eb1b79616d6e6abb21e9078dea84ea879c5349",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymcp-0.3.6-windows-11-arm64.zip",
            sha256: "34b5b86ee87de6abe0ca486aae2f851f956884258f4182987926c7e36cc8219e",
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
