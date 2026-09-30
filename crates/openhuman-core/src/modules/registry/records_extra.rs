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
    version: "0.1.5",
    release_url: "https://github.com/tinyhumansai/tinychannels/releases/tag/v0.1.5",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinychannels-module-0.1.5-ubuntu-24.04-x86_64.tar.gz",
            sha256: "61a1fa203b30ecd5aa79994d16fa8f00dbf969c1c2b859f00db1eda9723a2990",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinychannels-module-0.1.5-ubuntu-24.04-arm64.tar.gz",
            sha256: "e56a4747fd1652ecc9c10b02fa06290c7e7edf0585c350e09f06cd94158312a2",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinychannels-module-0.1.5-ubuntu-22.04-x86_64.tar.gz",
            sha256: "7be1b571055b7a4c08155bd9c0c6f55028ecd6a22045ca783d8e51060ebf6d42",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinychannels-module-0.1.5-ubuntu-22.04-arm64.tar.gz",
            sha256: "b0da1dd58b93690d6dbbdc33917065b56f8e9b17387fc6dfaa4a8825d40cadd2",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinychannels-module-0.1.5-macos-26-arm64.tar.gz",
            sha256: "060f390f389db7b5f7841e33e451b4beb811a913ae740aa3a0bf24fff16a36cd",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinychannels-module-0.1.5-macos-26-x86_64.tar.gz",
            sha256: "deb55d3b79e93793d18d5310d4cd7dc4fff0c997f69bc428e1ba4f2fefc99eec",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinychannels-module-0.1.5-macos-15-arm64.tar.gz",
            sha256: "a38c1dfa6babe489ff3639b936b0663a4e91ff23ea12ff98083150faaef354c3",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinychannels-module-0.1.5-macos-15-x86_64.tar.gz",
            sha256: "61de14d23dec40946011b154d3e39bc28e53d618ba193638d783a1cd941bcaca",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinychannels-module-0.1.5-windows-2025-x86_64.zip",
            sha256: "7b203eff540aa86040f55ad83e0bc7d18b05ec329d88ca75647b4b6aa8fe59be",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinychannels-module-0.1.5-windows-2022-x86_64.zip",
            sha256: "efa1783d09da527c8c020d8c4b5ae59787ecf1a813ba0bd124241e64456179d0",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinychannels-module-0.1.5-windows-11-arm64.zip",
            sha256: "8192939f8549cc2294b60d257e2c652d6aa5cfdaad9a58cf1f5aac1af9a47576",
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
    version: "0.1.9",
    release_url: "https://github.com/tinyhumansai/tinyhosts/releases/tag/v0.1.9",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyhosts-0.1.9-ubuntu-24.04-x86_64.tar.gz",
            sha256: "ab2af4c7cc89d1fa1e2dcdb5e55f59e227920438827e16cd3c9f0f53e50c8ad7",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyhosts-0.1.9-ubuntu-24.04-arm64.tar.gz",
            sha256: "22d819273bc8a41ff292dd717b0933524155f389fcfb8d57e8a536eee7aa4696",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyhosts-0.1.9-ubuntu-22.04-x86_64.tar.gz",
            sha256: "47bed7056c19520f29d6eda35c56f45161989b94c4d2b47d686d5f3c2c982a09",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyhosts-0.1.9-ubuntu-22.04-arm64.tar.gz",
            sha256: "d51c8d606e438aac521ceb1aee5dedc42c9a23dd864abe474c7e93b18de0bddf",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyhosts-0.1.9-macos-26-arm64.tar.gz",
            sha256: "b32a3f8da97c7919b17dfa80768a1368d039117b74e093dd093e8b34c81075ef",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyhosts-0.1.9-macos-26-x86_64.tar.gz",
            sha256: "eff55c0dda3d09332da74433afefd444b607d40a62c582054f7abe88d1eccd97",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyhosts-0.1.9-macos-15-arm64.tar.gz",
            sha256: "b021cbb7f80db7636577769de693939326b6ba09290c20ea4cd7fab911db8741",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyhosts-0.1.9-macos-15-x86_64.tar.gz",
            sha256: "521f4c7b8322fc7d18287e62dd041840214eb588020233658229374482546aca",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyhosts-0.1.9-windows-2025-x86_64.zip",
            sha256: "4b77786d1b3cfb52b5fffb79f494b562629c574d53633da421f95c1af9d552c9",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyhosts-0.1.9-windows-2022-x86_64.zip",
            sha256: "1e2e327e49fa7821530f27781a72714e95248816d7875e1b2357b8e6237bb4f2",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyhosts-0.1.9-windows-11-arm64.zip",
            sha256: "8f90106ecb4f7840c031d76a34ecfff86f7677d13251e2e646180c380cc8a1d0",
        },
    ],
    load: LoadPolicy::Lazy,
};
