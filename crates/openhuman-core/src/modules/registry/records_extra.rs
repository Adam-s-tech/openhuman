//! Registry records for additional first-party TinyBus modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The `tinybox` module, loaded on demand.
pub(crate) const TINYBOX: ModuleRecord = ModuleRecord {
    id: "tinybox",
    description: "Sandbox capability discovery through TinyBox",
    bus_name: "ai.tinyhumans.tinybox.Box",
    object_path: "/ai/tinyhumans/tinybox/Box",
    version: "0.1.11",
    release_url: "https://github.com/tinyhumansai/tinybox/releases/tag/v0.1.11",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinybox-0.1.11-ubuntu-24.04-x86_64.tar.gz",
            sha256: "ffa8a8c34b3895c6e8e0cda1bd725f624681a8b079acacdaee8b68e0333f979f",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinybox-0.1.11-ubuntu-24.04-arm64.tar.gz",
            sha256: "b9f9720341b04c1ba69dd348af1bfa45c5e4f75818f64f98db221b4f076a1b0e",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinybox-0.1.11-ubuntu-22.04-x86_64.tar.gz",
            sha256: "803fa415d8964573d87e3cfc19448e3be95f1d269daed914f62a0a8f01c61fca",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinybox-0.1.11-ubuntu-22.04-arm64.tar.gz",
            sha256: "03cb7b443428d5ffc47c76b1fd3a29ce172b8067cc7cd68e23259b95d28709fe",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinybox-0.1.11-macos-26-arm64.tar.gz",
            sha256: "c07cd3cf4d27f8dcf51ad67b23f00a447212a713cb4c1689486924d65091f8e5",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinybox-0.1.11-macos-26-x86_64.tar.gz",
            sha256: "01a2105a36483b56b29af0a8080d53d7facae5817ab20de91fc575fd5bebfe50",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinybox-0.1.11-macos-15-arm64.tar.gz",
            sha256: "c642b5c347d893051cfadbeb0b893f7322efc0b9462ddc4091f9d63f09a3d7e5",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinybox-0.1.11-macos-15-x86_64.tar.gz",
            sha256: "a493da039b4b72b779948ebc3c7783888e297a56c07cd34a4216ae05a2bd4b89",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinybox-0.1.11-windows-2025-x86_64.zip",
            sha256: "3f4773c3170bb16a05def99540391b2943d3fdd8f22a1ed4b9dabee65e8ac2f5",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinybox-0.1.11-windows-2022-x86_64.zip",
            sha256: "557963b920745ef46645a5f7a41a3033a249f87e84ae924c68add9ee3abd14a5",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinybox-0.1.11-windows-11-arm64.zip",
            sha256: "57c2a6818ab8ed63b48084c993160ad634d1ea6479e3f6e40e5f33c66ebf8224",
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
    version: "0.1.7",
    release_url: "https://github.com/tinyhumansai/tinychannels/releases/tag/v0.1.7",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinychannels-module-0.1.7-ubuntu-24.04-x86_64.tar.gz",
            sha256: "edcab34853733fe270056e0c7b743a1e1dcc5e9f2cb5a92f39429e72936ef0f1",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinychannels-module-0.1.7-ubuntu-24.04-arm64.tar.gz",
            sha256: "6a2bf972739c1b1ccd98671e324306a3534ea6d8cbfe0360917cde7a7867a351",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinychannels-module-0.1.7-ubuntu-22.04-x86_64.tar.gz",
            sha256: "1dc21218deaeeee14df2c10f3661911e4f2af44ed8dadfae576b86e83b2eb8fb",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinychannels-module-0.1.7-ubuntu-22.04-arm64.tar.gz",
            sha256: "a56d41f12bae886e33d55f867aed6ace48c1693a15c0944b7115fbefeb4dd492",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinychannels-module-0.1.7-macos-26-arm64.tar.gz",
            sha256: "e96edd82e8a585405920eee601f5a591c9f98a70b4434af9a40d0ca723a256fe",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinychannels-module-0.1.7-macos-26-x86_64.tar.gz",
            sha256: "a3dd8910fc21120eeb53412e88651bf40f78e6db204bc3e1d9673c894108782c",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinychannels-module-0.1.7-macos-15-arm64.tar.gz",
            sha256: "70d4491059733b82f065c3f484dc03d1a1c8135e1e95443aa6ca3dbf6019caad",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinychannels-module-0.1.7-macos-15-x86_64.tar.gz",
            sha256: "fb549fbb7a0a8ee35714cb950b4be8f238664d7528ae555b9edf0daf54a67c04",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinychannels-module-0.1.7-windows-2025-x86_64.zip",
            sha256: "ad05660012a83643c947494afbd72f96ff2ab4f3dcaa8e3780a53b24340931c9",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinychannels-module-0.1.7-windows-2022-x86_64.zip",
            sha256: "413aba06cf02c243fa3e62ffeea3d7e67c6be99bf2895cdef9215f422856bab8",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinychannels-module-0.1.7-windows-11-arm64.zip",
            sha256: "81d97c6be328c2ed9fb7a5f6ac37966c2d79f50a237578206e902a96c41f63ab",
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
    version: "0.2.0",
    release_url: "https://github.com/tinyhumansai/tinyhosts/releases/tag/v0.2.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyhosts-0.2.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "997e20ab8086237ff8b81e547d499a69bfa39ef31c95f0589a849bb3dc610662",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyhosts-0.2.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "18343056f07d9b64a459c1c51af5e69d0c609f0e4da4b016215e9d6b7cfafd98",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyhosts-0.2.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "62c0f55f692acc8b1774f14bd31a1411aa4cec6c9aa2afc8f9b879cf3a4672b6",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyhosts-0.2.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "d7c8aa7a9f7bbd4f45bac6f511d9b63c5376ffb9392b62b7d3118b007e112c05",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyhosts-0.2.0-macos-26-arm64.tar.gz",
            sha256: "ee1af07a90c8b0b6b22957cff19be8019f0c24e7d2927d4b46646fb844eee7da",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyhosts-0.2.0-macos-26-x86_64.tar.gz",
            sha256: "eb68898107487c5bbbad8947a61ae68efed39f76e9068393315f39a290664dd5",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyhosts-0.2.0-macos-15-arm64.tar.gz",
            sha256: "6e5c85dea016e264c50a8713eaa1daf72e7d05b512bdab948ab4f6fdc455ad3a",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyhosts-0.2.0-macos-15-x86_64.tar.gz",
            sha256: "fcdd37ee49cb91cd8b2304f85c2e446f2dcefdb1f07f993b7c78b1b3d829a5e5",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyhosts-0.2.0-windows-2025-x86_64.zip",
            sha256: "dbb51a8d93fbd705f927fe8cdf218c9df20dd10c4f0dfe505af42332d4dc0c3a",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyhosts-0.2.0-windows-2022-x86_64.zip",
            sha256: "a06b86cb0a407d0ccdd81d3f9cb6d7a9b5a9c230c4fa25e35df0b79ccc6b4cd4",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyhosts-0.2.0-windows-11-arm64.zip",
            sha256: "b909df338118a72c257e506aa305d1cfe3d9ee1b8dfb2f56b5e1f9c50b588d34",
        },
    ],
    load: LoadPolicy::Lazy,
};
