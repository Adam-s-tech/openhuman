//! Native desktop module. Published digests are added only from its release manifest.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};
use tinycomputer_bus::names;

pub(crate) const TINYDESKTOP: ModuleRecord = ModuleRecord {
    id: "tinydesktop",
    description: "Permission-aware native desktop observation and control",
    bus_name: names::INTERFACE,
    object_path: names::OBJECT_PATH,
    version: "0.5.2",
    release_url: "https://github.com/tinyhumansai/tinycomputer/releases/tag/v0.5.2",
    // Verbatim from the published tinycomputer v0.5.2 checksum.toml. Linux remains outside
    // the initial product surface; this registry admits macOS and Windows.
    assets: &[
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinycomputer-0.5.2-macos-26-arm64.tar.gz",
            sha256: "7f4aa7136c9c09a2ac7752b4adb7b4aa76c3a1b035971517cd6f2f605c54fab2",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinycomputer-0.5.2-macos-26-x86_64.tar.gz",
            sha256: "227704b461caa1a989ae86ef36b4ca5c9be2d95784a8ae02fede896841f81a5c",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinycomputer-0.5.2-macos-15-arm64.tar.gz",
            sha256: "490bcafe3022cc2873c9559f327cf4bd6d3a26f9886f8d6ca7f88c2f5910d9fb",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinycomputer-0.5.2-macos-15-x86_64.tar.gz",
            sha256: "9e1fda30cfc60848226f19085c98cbd6c8f2496675f6edddde19155d9e633fa4",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinycomputer-0.5.2-windows-2025-x86_64.zip",
            sha256: "af361b09abfceed383aef25a60dc467be24299ae465a48a33f36059c3aad12a3",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinycomputer-0.5.2-windows-2022-x86_64.zip",
            sha256: "94987c421235e362ed07fb477b3603d25774e4ebee2252ac853744186f6c546c",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinycomputer-0.5.2-windows-11-arm64.zip",
            sha256: "aa93c5821f62702588d5c02f5ba5b6be3a5622a7e8a67c1dfd29100da06e6b9c",
        },
    ],
    load: LoadPolicy::Lazy,
};
