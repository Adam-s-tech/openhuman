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
    version: "0.11.1",
    release_url: "https://github.com/tinyhumansai/tinyconnectors/releases/tag/v0.11.1",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyconnectors-0.11.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "6a182ea082dda6ccd7a35521faf10239fc9d40dfb412b7ad5f77bd46daccfcc1",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyconnectors-0.11.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "5bee8f3e27b57a88a712e9720b54eb2559913da5b8716c9a5b10e93bf5bd8972",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyconnectors-0.11.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "77490ba89c55acf6a602a5634b855a2bf1da3ea9af72575d802d3e5d8dbebbe8",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyconnectors-0.11.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "384ba91126fb14091200026e2c0d76d472a38b726cb8b6282186745848c213ff",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyconnectors-0.11.1-macos-26-arm64.tar.gz",
            sha256: "4e8a00da294802242e06e579dc057358764234f907f6097806f8c15ff7352e12",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyconnectors-0.11.1-macos-26-x86_64.tar.gz",
            sha256: "b1cd5dcd85a6c7944d9d6751ed06536b7112a94fb0183e7f3c6933f1d619c63c",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyconnectors-0.11.1-macos-15-arm64.tar.gz",
            sha256: "c53901e207c02f88a37e43ae32924402f1cbaf8da9a561c570b48063d846fac9",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyconnectors-0.11.1-macos-15-x86_64.tar.gz",
            sha256: "ee808d7498e35c6dcd5e30b02afc52db1eb82e548ef7273eb136fd1d69cd2afe",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyconnectors-0.11.1-windows-2025-x86_64.zip",
            sha256: "b3ee7d489424610bf48a226f69f76d8e498f361a232b33f20cb6fda14738efd1",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyconnectors-0.11.1-windows-2022-x86_64.zip",
            sha256: "e8380cb14801da35a3f42b1a9333842357d98779f509326dd5ed70bff0403ec9",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyconnectors-0.11.1-windows-11-arm64.zip",
            sha256: "99d42e4daee8b21ad00a8f7a383ed457f53e802e319e78573c958f15cb51ddcd",
        },
    ],
    // Lazy: a user with no connected accounts should not pay to load it, and
    // most sessions never touch a connector. Safe even signed out — the module
    // loads without configuration and still answers the capability members.
    load: LoadPolicy::Lazy,
};
