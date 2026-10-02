//! TinySearch web-search module. Published digests are added only from its
//! release manifest.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

pub(crate) const TINYSEARCH: ModuleRecord = ModuleRecord {
    id: "tinysearch",
    description:
        "Web search, grounded answers and page contents across providers through TinySearch",
    bus_name: tinysearch_bus::names::INTERFACE,
    object_path: tinysearch_bus::names::OBJECT_PATH,
    version: "0.3.3",
    release_url: "https://github.com/tinyhumansai/tinysearch/releases/tag/v0.3.3",
    // Verbatim from the published v0.3.3 checksum.toml; the same host set as
    // the other native modules (macOS, Ubuntu, Windows).
    assets: &[
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysearch-0.3.3-macos-26-arm64.tar.gz",
            sha256: "79d04d1571cd1deba0f6cbd656164b94682a37c565d45178aafa261f0c1ae5f1",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysearch-0.3.3-macos-26-x86_64.tar.gz",
            sha256: "f662569dfd4060c99c2bc61b0262ad9e60e0e3f3d351f7047dfdc4d7f08b2ab4",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysearch-0.3.3-macos-15-arm64.tar.gz",
            sha256: "299cc57b91d53570f838237fd08b0839b3a9f51da92b64e2d9e74dc1c7cb4485",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysearch-0.3.3-macos-15-x86_64.tar.gz",
            sha256: "f707690145b65d8a3302e85075ee81d8d53d01090c24c19b8603918e54adfb40",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysearch-0.3.3-ubuntu-24.04-x86_64.tar.gz",
            sha256: "28dfba6269c8e14c382638961270a44d64e9a6365f1435feb3ebfb9a30cd89ea",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysearch-0.3.3-ubuntu-24.04-arm64.tar.gz",
            sha256: "3a4552bca4a969af4ee845bff3333468c8302f367f6a135b6fcb949b250b753f",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysearch-0.3.3-ubuntu-22.04-x86_64.tar.gz",
            sha256: "e5839c2a4a370b8a305f710a4c047234f038646d7d692536f1afc768c917706a",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysearch-0.3.3-ubuntu-22.04-arm64.tar.gz",
            sha256: "46a70b31f30b737aef65967dc24fb8e3cf015d7e8701f10a141d67a18bfbd534",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysearch-0.3.3-windows-2025-x86_64.zip",
            sha256: "29b33dd6a4a0c4e002d2874c3d3d34ee3a408fa0f055903fdad27dad844bb324",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysearch-0.3.3-windows-2022-x86_64.zip",
            sha256: "ebfae9cfef9bbab7f589e19b7360ed0e735cdffc9b52936ad6c4dbf9375b4090",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysearch-0.3.3-windows-11-arm64.zip",
            sha256: "099e1803e67ce08bc026d9e462bf33020d2459798fe2f4a88cff09d829c7d4e7",
        },
    ],
    load: LoadPolicy::Lazy,
};
