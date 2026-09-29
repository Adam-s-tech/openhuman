//! TinySearch web-search module. Published digests are added only from its
//! release manifest.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

pub(crate) const TINYSEARCH: ModuleRecord = ModuleRecord {
    id: "tinysearch",
    description:
        "Web search, grounded answers and page contents across providers through TinySearch",
    bus_name: tinysearch_bus::names::INTERFACE,
    object_path: tinysearch_bus::names::OBJECT_PATH,
    version: "0.3.2",
    release_url: "https://github.com/tinyhumansai/tinysearch/releases/tag/v0.3.2",
    // Verbatim from the published v0.3.2 checksum.toml; the same host set as
    // the other native modules (macOS, Ubuntu, Windows).
    assets: &[
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysearch-0.3.2-macos-26-arm64.tar.gz",
            sha256: "f7edfa71e840225d3eb529a90a54dd22ad9d0aa3b5cfebcc1e9a025a8d63497d",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysearch-0.3.2-macos-26-x86_64.tar.gz",
            sha256: "2e6755ceea67939d8f3aea1fd1ef672558f8f9b24cccaefca58d483c40869f25",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysearch-0.3.2-macos-15-arm64.tar.gz",
            sha256: "25535b6403faa6dfa645bdc811cdf553b0b2a4c9cba041f865a26e8dc5d51966",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysearch-0.3.2-macos-15-x86_64.tar.gz",
            sha256: "b08024ed59ff1f594f32831769ff01c0ce015534eb7c9c36ebd92b3979f87445",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysearch-0.3.2-ubuntu-24.04-x86_64.tar.gz",
            sha256: "d11a9654e196760252a1ef22a876856928c0367c8edec92dfe555b6c87960053",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysearch-0.3.2-ubuntu-24.04-arm64.tar.gz",
            sha256: "4508fa4b417ee34eb020710d6e4497dcfb6980c9b6cb52aea54f723e78109f81",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysearch-0.3.2-ubuntu-22.04-x86_64.tar.gz",
            sha256: "bd956d91f5831bdcbe884da320669823f57e4ce979e10190bc6ae93503bf1fa0",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysearch-0.3.2-ubuntu-22.04-arm64.tar.gz",
            sha256: "0f8f3a117e9d15b94f3089c97b51f30975dd76397bfd2504e86ce7dbdaaa99d9",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysearch-0.3.2-windows-2025-x86_64.zip",
            sha256: "f741b21d1fd25d61eee7e085ef1271956d123dc93d58015b118ab25bcf83c8d3",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysearch-0.3.2-windows-2022-x86_64.zip",
            sha256: "34087c878db07962360a794980fd309e4aae1db3b71f9001ef949f068e1b9c97",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysearch-0.3.2-windows-11-arm64.zip",
            sha256: "a36f95572de42c6c6bc8c9fd2b9c8a0e83ba84862b6cf1d9cc7793d520cd7b9f",
        },
    ],
    load: LoadPolicy::Lazy,
};
