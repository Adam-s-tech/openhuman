//! Registry records for additional first-party TinyBus modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The `tinybox` module, loaded on demand.
pub(crate) const TINYBOX: ModuleRecord = ModuleRecord {
    id: "tinybox",
    description: "Sandbox capability discovery through TinyBox",
    bus_name: "ai.tinyhumans.tinybox.Box",
    object_path: "/ai/tinyhumans/tinybox/Box",
    version: "0.1.8",
    release_url: "https://github.com/tinyhumansai/tinybox/releases/tag/v0.1.8",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinybox-0.1.8-ubuntu-24.04-x86_64.tar.gz",
            sha256: "0f996d4bd6d9bb953ae80c69e2c6f92498e74c5696012bd534bdbbc9491207f6",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinybox-0.1.8-ubuntu-24.04-arm64.tar.gz",
            sha256: "df2b211ea18d7658255c1ee96ed354426e00fb54940d667022b09a4a188a5c9e",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinybox-0.1.8-ubuntu-22.04-x86_64.tar.gz",
            sha256: "8c5dc45e969d76efaff8adcfa855de74a6e4212c79532363dd6fd13cbdecb3c9",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinybox-0.1.8-ubuntu-22.04-arm64.tar.gz",
            sha256: "0564b1ce15fcab20e96cc8f0f99454db195c0d95aaa3b0a4bab0bff911bf2701",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinybox-0.1.8-macos-26-arm64.tar.gz",
            sha256: "f3a1eca2abf6ee0ee8a40f9ec64ce8595485ac9a51dc07210ec13b3adae7a9e5",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinybox-0.1.8-macos-26-x86_64.tar.gz",
            sha256: "bed63fbcf841d4c3207370dbc8dbc0f3d48ce38660db571bd1e5b3cfe2c5bdc0",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinybox-0.1.8-macos-15-arm64.tar.gz",
            sha256: "711ec78c7d9e506ff2feaa1b86cf43f1b0ca48a814fbe6a147dedc544b05724f",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinybox-0.1.8-macos-15-x86_64.tar.gz",
            sha256: "6433369f26ee09a8cfe690a5aa6a06f877079dce1b8dff1bf3756d8b57c80378",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinybox-0.1.8-windows-2025-x86_64.zip",
            sha256: "851c4474fb625675810dda21f5e25953ddcc47918aad7e50d2cb2f90c26d42e4",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinybox-0.1.8-windows-2022-x86_64.zip",
            sha256: "7a4d05d6b257292e21d94cba8c3d7dd2703f64b525b3449c0ddce7f67f19944f",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinybox-0.1.8-windows-11-arm64.zip",
            sha256: "2ecb77a901897da68590cbdba9b1b7a61eb48a3abb07dcfa1f1fcbe7968bcf9a",
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
    version: "0.1.3",
    release_url: "https://github.com/tinyhumansai/tinychannels/releases/tag/v0.1.3",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinychannels-module-0.1.3-ubuntu-24.04-x86_64.tar.gz",
            sha256: "645ef4535adf2ebfb9117784e7d019ff5fbb7f6abf7ec7906830c4d45c691c88",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinychannels-module-0.1.3-ubuntu-24.04-arm64.tar.gz",
            sha256: "bc8987e0fe91b89e0456abd9c55b5f645ccf26e3e6a9f2f518f5c381688ba1f2",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinychannels-module-0.1.3-ubuntu-22.04-x86_64.tar.gz",
            sha256: "c169dd43a4818a0030d99b7decec6ea3f2061ca0a3bdc822e8d0a3540a1dc672",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinychannels-module-0.1.3-ubuntu-22.04-arm64.tar.gz",
            sha256: "fe7c9517a42112081a7b5d2b6183a89ddcc76a04f55e4dd79de7de2adb944029",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinychannels-module-0.1.3-macos-26-arm64.tar.gz",
            sha256: "694c4ba0cd3d307f4bbc15dc093ae825a15e16db6c8d055b7f30b8387c52a50c",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinychannels-module-0.1.3-macos-26-x86_64.tar.gz",
            sha256: "1b0caa40d1265d14357acbbbba33646be11a203aa478ad2d64b6ba822a6aff5c",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinychannels-module-0.1.3-macos-15-arm64.tar.gz",
            sha256: "f967608ac5876043f197cb14e4d12d6480b3452c238797200440dd33edb98431",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinychannels-module-0.1.3-macos-15-x86_64.tar.gz",
            sha256: "a2541637c002df3a24fd25ff9885b48d68352f9c16c4870beb451c56ac878a13",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinychannels-module-0.1.3-windows-2025-x86_64.zip",
            sha256: "8571f254adc598343f1dcbddfe877ad54e9571bd83cc5ecbe7cfc7e1d9601662",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinychannels-module-0.1.3-windows-2022-x86_64.zip",
            sha256: "d9b4d4543c9129600bd9ed52b1086476a555bc3c01310813a4ddff347d716ffa",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinychannels-module-0.1.3-windows-11-arm64.zip",
            sha256: "5da9f9210e080e603d60bdf6b04857948f6a35960623aaa370f9d1eb2a7b85a0",
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
    version: "0.1.8",
    release_url: "https://github.com/tinyhumansai/tinyhosts/releases/tag/v0.1.8",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyhosts-0.1.8-ubuntu-24.04-x86_64.tar.gz",
            sha256: "39dd267e35467a00e56342eeef2e16b20c6b87ebdeaa261db4ce232241ca1d62",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyhosts-0.1.8-ubuntu-24.04-arm64.tar.gz",
            sha256: "ad38fb8d54434ddba0cff658a6169b824a7f54e6bf963c35cb24c88a0168ac4d",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyhosts-0.1.8-ubuntu-22.04-x86_64.tar.gz",
            sha256: "e6f46161fdea50b33f78872b74ec1d04c30535372f034a1f6332e3a9a51948bc",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyhosts-0.1.8-ubuntu-22.04-arm64.tar.gz",
            sha256: "e17e1d2dba8ad322d6b0cd4e99184ea2341729913c06adf0229512aaa04df7bf",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyhosts-0.1.8-macos-26-arm64.tar.gz",
            sha256: "5f5c17a6fb80d0e6ff6707a12163f463cf3602cef288cbcbc7b48057bc3363a8",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyhosts-0.1.8-macos-26-x86_64.tar.gz",
            sha256: "240800f543ce372a95e8faa9dd26c1fc346123bb14c67a0c576855235d21b21f",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyhosts-0.1.8-macos-15-arm64.tar.gz",
            sha256: "72cbd32f03bee8ed2177ed31b7611d0e790cb0b37d2c417850625a8287179ad9",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyhosts-0.1.8-macos-15-x86_64.tar.gz",
            sha256: "3603531bf0116f3a3df6d5ac177b0ce29b9b4c6f2b7c0e22ee8d7d3bcab23707",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyhosts-0.1.8-windows-2025-x86_64.zip",
            sha256: "c09116f7c51e0dd66dfb33149435321c34ad94651a55642caee74b203fead4da",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyhosts-0.1.8-windows-2022-x86_64.zip",
            sha256: "31dfaa0a4ebea1c127b23bef398a04bed5de867ffbe1acf1c2c94f9e083bb94f",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyhosts-0.1.8-windows-11-arm64.zip",
            sha256: "f587eb466232b381e6dee62f8671c5b5ec56187d10f27afbb8f7c23e533aad53",
        },
    ],
    load: LoadPolicy::Lazy,
};
