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
    version: "0.12.2",
    release_url: "https://github.com/tinyhumansai/tinyconnectors/releases/tag/v0.12.2",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyconnectors-0.12.2-ubuntu-24.04-x86_64.tar.gz",
            sha256: "a445b91aef567e7a698c0942b0da872b95e3ce37eeae712bbd4c3a44e6e579a8",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyconnectors-0.12.2-ubuntu-24.04-arm64.tar.gz",
            sha256: "45c04a59e7e307f9779d505a81728fbb65889adc9026914d0ecbd8030dc23f5d",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyconnectors-0.12.2-ubuntu-22.04-x86_64.tar.gz",
            sha256: "256b827247f54025b8928c5b5dc4bf12e362af9f6645a6a3eae87e7ad574e37d",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyconnectors-0.12.2-ubuntu-22.04-arm64.tar.gz",
            sha256: "2d051031963d2ff6815b5fc1c2eab46f618e288ecbb9802e50a5ce361b4e29ca",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyconnectors-0.12.2-macos-26-arm64.tar.gz",
            sha256: "314b755abf55bbe8801f1580d05bffd121b98c4ee719bf3c5102c03d93585f15",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyconnectors-0.12.2-macos-26-x86_64.tar.gz",
            sha256: "b1c9cc7c525098ee8b9edfc482f9d77c35eede166f9ada7ff98ff700e23ae543",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyconnectors-0.12.2-macos-15-arm64.tar.gz",
            sha256: "bb8554a829ad633cbf3f3e22b8dd441f37aba938e0a703838558bdb77c0d3d84",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyconnectors-0.12.2-macos-15-x86_64.tar.gz",
            sha256: "fe4d8ce00ef8d5dde3fa471fe233df120550c554d915d56d9332d3ea5e5b97fc",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyconnectors-0.12.2-windows-2025-x86_64.zip",
            sha256: "a00c040d6568d793e6ba4def715166f54c04e08b3abbba620a4b3d5c71231473",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyconnectors-0.12.2-windows-2022-x86_64.zip",
            sha256: "7e94e4498d242a529a71f95c5dcd1f8d595f76f21955e69a8c0803343d16a99a",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyconnectors-0.12.2-windows-11-arm64.zip",
            sha256: "9bfd7ee8229514044f22168754602f5158afe3560a706ef52770a09203675d02",
        },
    ],
    // Lazy: a user with no connected accounts should not pay to load it, and
    // most sessions never touch a connector. Safe even signed out — the module
    // loads without configuration and still answers the capability members.
    load: LoadPolicy::Lazy,
};
