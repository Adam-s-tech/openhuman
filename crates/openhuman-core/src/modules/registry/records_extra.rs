//! Registry records for additional first-party TinyBus modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The `tinybox` module, loaded on demand.
pub(crate) const TINYBOX: ModuleRecord = ModuleRecord {
    id: "tinybox",
    description: "Sandbox capability discovery through TinyBox",
    bus_name: "ai.tinyhumans.tinybox.Box",
    object_path: "/ai/tinyhumans/tinybox/Box",
    version: "0.1.12",
    release_url: "https://github.com/tinyhumansai/tinybox/releases/tag/v0.1.12",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinybox-0.1.12-ubuntu-24.04-x86_64.tar.gz",
            sha256: "688a57769e28677a8ba2203ee50520e2e25184e797c95833cf3751a67fb70727",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinybox-0.1.12-ubuntu-24.04-arm64.tar.gz",
            sha256: "512fa5a076f477e9365a4de1210e6a7942bcc3449ca629aac750a0ff28ebb41e",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinybox-0.1.12-ubuntu-22.04-x86_64.tar.gz",
            sha256: "fd4f1b7bf94c8578fdafc6af56cae5a80816b8bc93d42e3683ae74fa656cfbc8",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinybox-0.1.12-ubuntu-22.04-arm64.tar.gz",
            sha256: "cadcb6155dc9a8cceff999c6c88f4fa868f910e0cfd0da6ee71c2c50993ffe1d",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinybox-0.1.12-macos-26-arm64.tar.gz",
            sha256: "808b76f3b056d970aad6c06b1864a61bc3f11834ba618a10d2edd5b92745335b",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinybox-0.1.12-macos-26-x86_64.tar.gz",
            sha256: "484aab8ae5029a90c909fc264247ec174d6fbf1c3e5d49d4e25dbcfff9279e96",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinybox-0.1.12-macos-15-arm64.tar.gz",
            sha256: "379272681a057a1885a46b99e0790a66a56fecb21072f2c79a7f0c08c6a21b0c",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinybox-0.1.12-macos-15-x86_64.tar.gz",
            sha256: "1ac79bae11b328e1de5e470b8dfda121b53fdd17d3623026fd913b5d5f8f6d23",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinybox-0.1.12-windows-2025-x86_64.zip",
            sha256: "389a3a622fa41f3fb72edcdfc49b957d80985b47ebe1636b5e84e0e7de48338b",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinybox-0.1.12-windows-2022-x86_64.zip",
            sha256: "8e4ceaadf870ebc257727aaec9d26de129e280895a5cd3312dd6cbb1c2ed2a46",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinybox-0.1.12-windows-11-arm64.zip",
            sha256: "a3469c92ead646d61a7f9a1a1d88661fa07a0fc42188bc9892a418f1250c7d8b",
        },
    ],
    load: LoadPolicy::Lazy,
};

/// The `tinychannels` module, loaded on demand.
pub(crate) const TINYCHANNELS: ModuleRecord = ModuleRecord {
    id: "tinychannels",
    description: "Channel provider lifecycle and message transport",
    bus_name: "ai.tinyhumans.tinychannels.Channels",
    object_path: "/ai/tinyhumans/tinychannels/Channels",
    version: "0.1.8",
    release_url: "https://github.com/tinyhumansai/tinychannels/releases/tag/v0.1.8",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinychannels-module-0.1.8-ubuntu-24.04-x86_64.tar.gz",
            sha256: "66e307704c24fc83cd9ee774715273f6087749fca7675eae871730c24dec2294",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinychannels-module-0.1.8-ubuntu-24.04-arm64.tar.gz",
            sha256: "664752386a4d87b32f2ce6f6110218fb40b00c2e8a2ff2e3486b4ced8fa17a0b",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinychannels-module-0.1.8-ubuntu-22.04-x86_64.tar.gz",
            sha256: "016a4c35a8fbd3adf7c166fdfe85b1d16b4e4db82856a65a443c56d4d77450f4",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinychannels-module-0.1.8-ubuntu-22.04-arm64.tar.gz",
            sha256: "ea3f9db26bd6cef5ea6554c6ff61014253785379510bdb501e2d26c67364bcf7",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinychannels-module-0.1.8-macos-26-arm64.tar.gz",
            sha256: "3413c5a67558d4a67d84135e7094bb12548e6e3768d31c1921f504215525992b",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinychannels-module-0.1.8-macos-26-x86_64.tar.gz",
            sha256: "f23c6187ff2bc21c5b47b62b7159a7440d6598d32c3826386fef3a1785f8f52b",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinychannels-module-0.1.8-macos-15-arm64.tar.gz",
            sha256: "4271bcd6610e431f42d99e5b90f861217043465f5b273cbed7278a1a3d69bb16",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinychannels-module-0.1.8-macos-15-x86_64.tar.gz",
            sha256: "24b16b38c0e966b56b346c2ca1fdb523a4526385bbaf2b8c32ba92d9990be462",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinychannels-module-0.1.8-windows-2025-x86_64.zip",
            sha256: "42b458aef3dbfd534eeda4fc72b0aed34463902abc49f68d7e3f41f5506bc554",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinychannels-module-0.1.8-windows-2022-x86_64.zip",
            sha256: "12164e63fca5caa47468b3dab53534253b2149dc0425e293e4dc4b91fa9a0e24",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinychannels-module-0.1.8-windows-11-arm64.zip",
            sha256: "b7cb2506aeb27d87d64f2776f16ffcf9fd70518b6de0750d7d8d04d47af9a840",
        },
    ],
    load: LoadPolicy::Lazy,
};

/// The `tinyhosts` module, loaded on demand.
pub(crate) const TINYHOSTS: ModuleRecord = ModuleRecord {
    id: "tinyhosts",
    description: "Hosting provider operations",
    bus_name: "ai.tinyhumans.tinyhosts.Hosting",
    object_path: "/ai/tinyhumans/tinyhosts/Hosting",
    version: "0.2.1",
    release_url: "https://github.com/tinyhumansai/tinyhosts/releases/tag/v0.2.1",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyhosts-0.2.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "d98b3821dd3df1966c76005a033f33a971e968d25aa24d41dfc4c464f6c3927e",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyhosts-0.2.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "72c759b1ce4d6e2b9d651fece15c2bd797dea087fe48e4351c677e57a40fa0aa",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyhosts-0.2.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "4256b7017fad9536b8c7642e6154500bcd2d342a49efa6c801b1033a37ad7ac0",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyhosts-0.2.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "bd08b3bd8d5a99ded5a77e8c9a0117e06bcc3610a7f3f3565f0903b8eab34700",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyhosts-0.2.1-macos-26-arm64.tar.gz",
            sha256: "b6025f13f9f11776864299d6b46823e7ec6116c9d3f19074ffc2f0765a023d79",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyhosts-0.2.1-macos-26-x86_64.tar.gz",
            sha256: "18fc22353a2b9469e3add074fdce135a3b120d592428af9d652ac6f2669d303e",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyhosts-0.2.1-macos-15-arm64.tar.gz",
            sha256: "97f3823533b168e555ba07b4e3020a8b72463450a8c36f8d49e6daeabd2dd18d",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyhosts-0.2.1-macos-15-x86_64.tar.gz",
            sha256: "46dd60c9a6bd85b751dbba27f619e85f61e83877038238eb6dfcf4a54b402936",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyhosts-0.2.1-windows-2025-x86_64.zip",
            sha256: "dfbdcfdcc3ab4a374beae411a36384db33c0eded331ab1e85495681baef3c6a1",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyhosts-0.2.1-windows-2022-x86_64.zip",
            sha256: "f1e780cd4bcc68cc7cd4a4e57757af765ff50fe3205b9fca9f85b766c96f2dd8",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyhosts-0.2.1-windows-11-arm64.zip",
            sha256: "335407db7c06989c8d35e545a79268b1a0675a966d55c00847abf4ce1f52bb91",
        },
    ],
    load: LoadPolicy::Lazy,
};
