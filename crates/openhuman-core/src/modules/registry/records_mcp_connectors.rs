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
    version: "0.12.1",
    release_url: "https://github.com/tinyhumansai/tinyconnectors/releases/tag/v0.12.1",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyconnectors-0.12.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "4c645157682328371a28d8c653708cb4be2ada4733b7010b908a3c403ab57b7a",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyconnectors-0.12.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "6897b3d7933c2a9481d501be3be0e427b20aea89108e447296af11353f6516ff",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyconnectors-0.12.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "62d0f74571a28ab8f55fbca61725b6859b6abb0cde9fc4b30b726c21e34e32e9",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyconnectors-0.12.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "784f10b06ead43838d3dcdfe57d1d3f67749dd9fcd7af3ee8029db70400a10e3",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyconnectors-0.12.1-macos-26-arm64.tar.gz",
            sha256: "cfc0ead03b6aaf701223833a8c7f0e362a7b34e00d3186c0e8fb20f65c079f9b",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyconnectors-0.12.1-macos-26-x86_64.tar.gz",
            sha256: "1f84de545402d0234bd5f77843dd1c603eb012780d0d539ee6e3cbfed4b8b472",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyconnectors-0.12.1-macos-15-arm64.tar.gz",
            sha256: "c294dacecc3f16e0e0207996a2679a4d9821934066dee13706b7073ba59087c6",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyconnectors-0.12.1-macos-15-x86_64.tar.gz",
            sha256: "5bd5fdc2c4692e7cba0b005fe3bcd1687b250369a59106d19251ea9d88b20c80",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyconnectors-0.12.1-windows-2025-x86_64.zip",
            sha256: "0495cde55856fca1c08746def82c3d026970cab9451b9b917f2d6ebfd043e220",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyconnectors-0.12.1-windows-2022-x86_64.zip",
            sha256: "d27b18cba4a15dd702221712e729eabbe57cae01565054734ebcec1204ab4720",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyconnectors-0.12.1-windows-11-arm64.zip",
            sha256: "dcacaacdb4fb2d7adbc1bd418ffc0fef4496af96621e7ef12e8c63f335fdd7ef",
        },
    ],
    // Lazy: a user with no connected accounts should not pay to load it, and
    // most sessions never touch a connector. Safe even signed out — the module
    // loads without configuration and still answers the capability members.
    load: LoadPolicy::Lazy,
};
