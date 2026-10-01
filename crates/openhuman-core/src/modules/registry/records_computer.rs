//! Native computer-use module (desktop and browser control). Published digests are added only from its release manifest.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};
use tinycomputer_bus::names;

pub(crate) const TINYCOMPUTER: ModuleRecord = ModuleRecord {
    id: "tinycomputer",
    description: "Permission-aware desktop and browser observation and control",
    bus_name: names::INTERFACE,
    object_path: names::OBJECT_PATH,
    version: "0.9.0",
    release_url: "https://github.com/tinyhumansai/tinycomputer/releases/tag/v0.9.0",
    // Verbatim from the published tinycomputer v0.9.0 checksum.toml. Linux remains outside
    // the initial product surface; this registry admits macOS and Windows.
    assets: &[
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinycomputer-0.9.0-macos-26-arm64.tar.gz",
            sha256: "46680123d5a5547831a296bbe1ce44e5be238be1ce94517f5c9b63ec0f90a55f",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinycomputer-0.9.0-macos-26-x86_64.tar.gz",
            sha256: "317eb06cc63c36eb4cdaac8b6b974b819961b5cce0290b3fd1fb6c2d2a7d0f33",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinycomputer-0.9.0-macos-15-arm64.tar.gz",
            sha256: "aa950fd1d72a27eeba1624a6245dfc23347fa251cee56ccb38ecf141da6c52ff",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinycomputer-0.9.0-macos-15-x86_64.tar.gz",
            sha256: "b1295a395efc38d0e3585b7e2a7b73dc3829047b1c46a178c2e75c801d5bfd55",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinycomputer-0.9.0-windows-2025-x86_64.zip",
            sha256: "034a10d02ef6eff432467e2f4c2fec75591437bb288cdeacf137fc2acd3de41c",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinycomputer-0.9.0-windows-2022-x86_64.zip",
            sha256: "f5a0f464e023386ed2c599c5955051154712259743f5d6307d276f3783826fa1",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinycomputer-0.9.0-windows-11-arm64.zip",
            sha256: "93bb51478673c39cc55be8b3a2e51039e6e2f84ef418783d8047ed159edf17c9",
        },
    ],
    load: LoadPolicy::Lazy,
};
