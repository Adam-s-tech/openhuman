//! TinySearch web-search module. Published digests are added only from its
//! release manifest.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

pub(crate) const TINYSEARCH: ModuleRecord = ModuleRecord {
    id: "tinysearch",
    description:
        "Web search, grounded answers and page contents across providers through TinySearch",
    bus_name: tinysearch_bus::names::INTERFACE,
    object_path: tinysearch_bus::names::OBJECT_PATH,
    version: "0.3.4",
    release_url: "https://github.com/tinyhumansai/tinysearch/releases/tag/v0.3.4",
    // Verbatim from the published v0.3.4 checksum.toml; the same host set as
    // the other native modules (macOS, Ubuntu, Windows).
    assets: &[
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysearch-0.3.4-macos-26-arm64.tar.gz",
            sha256: "17e3965243875dbf19af3643f52d5a4b5807d6beaeb0b7b2a430cd32218866ff",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysearch-0.3.4-macos-26-x86_64.tar.gz",
            sha256: "099944a6b7c7c9992fabba7e6f20f77bf06c427fb6b9d615b0146dfb7c838071",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysearch-0.3.4-macos-15-arm64.tar.gz",
            sha256: "eac1add415517db2a3e420cf8096a6ebce430279ca094d2df560a26962106f6a",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysearch-0.3.4-macos-15-x86_64.tar.gz",
            sha256: "34d6186991473f829f22d62dcbe264d099301aec9ec87fbada7d023da8f1ad0d",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysearch-0.3.4-ubuntu-24.04-x86_64.tar.gz",
            sha256: "0bd3e0ae45132719ed8d6c15326254cdd7719b4e390f76e48ea039e5b56313d8",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysearch-0.3.4-ubuntu-24.04-arm64.tar.gz",
            sha256: "28dab13172c324790d6c7d86eab34090ef87f7c676d834eb90e442ba195e5a26",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysearch-0.3.4-ubuntu-22.04-x86_64.tar.gz",
            sha256: "5fb639db07eab0b007cdd8116eda228f7230b2f1d33fb12cd0018377a49c2a56",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysearch-0.3.4-ubuntu-22.04-arm64.tar.gz",
            sha256: "3ccd0f00a9df72153d9737e1dc4bd9331f9c36c80d3a171609193f053d522c70",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysearch-0.3.4-windows-2025-x86_64.zip",
            sha256: "ca0654a18d42003c8703c825300fabc37f97e5a36678ab52204057f7e05c46d8",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysearch-0.3.4-windows-2022-x86_64.zip",
            sha256: "7e6ea2f3b90ca476d6b26905e7ef42b0d4bbd08409563c9b4243eadfad250a76",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysearch-0.3.4-windows-11-arm64.zip",
            sha256: "f691cce95d1d284c28e33831f39b78707296fd88496ae961194ff4fb5fbd36c8",
        },
    ],
    load: LoadPolicy::Lazy,
};
