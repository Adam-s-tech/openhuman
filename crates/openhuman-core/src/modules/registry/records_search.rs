//! TinySearch web-search module. Published digests are added only from its
//! release manifest.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

pub(crate) const TINYSEARCH: ModuleRecord = ModuleRecord {
    id: "tinysearch",
    description:
        "Web search, grounded answers and page contents across providers through TinySearch",
    bus_name: tinysearch_bus::names::INTERFACE,
    object_path: tinysearch_bus::names::OBJECT_PATH,
    version: "0.3.1",
    release_url: "https://github.com/tinyhumansai/tinysearch/releases/tag/v0.3.1",
    // Verbatim from the published v0.3.0 checksum.toml; the same host set as
    // the other native modules (macOS, Ubuntu, Windows).
    assets: &[
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysearch-0.3.1-macos-26-arm64.tar.gz",
            sha256: "6accfe42433334b690d33272903734862ee29534e1c23325760254f321caf826",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysearch-0.3.1-macos-26-x86_64.tar.gz",
            sha256: "bdf23be8ce9c15652fc9082e8e33a56dfbcee880fedf707c7d65163c24ca78a5",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysearch-0.3.1-macos-15-arm64.tar.gz",
            sha256: "cdb53e52c50837725f9578585a659cb208617dc8591042c56153b1d3df6bf577",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysearch-0.3.1-macos-15-x86_64.tar.gz",
            sha256: "886fd1b917d16eea5979ee24bd1baf94b75151c0bcb0f439c993addb9c75dc4a",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysearch-0.3.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "a830494c9e8be4ec0867b04a813eb3a748bc00489abccc6b7114def555043b64",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysearch-0.3.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "8de0f3ef4038a31f7eb4403f94d37b43a3a9bcd290155d677bc888228ca6dbf8",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysearch-0.3.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "4a4900a92b1c2a53b02322f464f4f32eca67f627b0a6658500e7e47e22a1cfb7",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysearch-0.3.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "e93cefc02968b95773ca7030af143c6ecd7211c7c1740d1559d971a0dc4c8605",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysearch-0.3.1-windows-2025-x86_64.zip",
            sha256: "222c13d50c2568bcd67049f64a5f3e6e72662b294f3d0cdf1939085368463d49",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysearch-0.3.1-windows-2022-x86_64.zip",
            sha256: "15359b8deb8930e460964d458967d0f35232ce46e8a05a00f85f15319019eed2",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysearch-0.3.1-windows-11-arm64.zip",
            sha256: "de70311105c3e7a758ed613624353722349b528a05602e3a998a023faae8a925",
        },
    ],
    load: LoadPolicy::Lazy,
};
