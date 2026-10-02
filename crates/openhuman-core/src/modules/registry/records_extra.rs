//! Registry records for additional first-party TinyBus modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The `tinybox` module, loaded on demand.
pub(crate) const TINYBOX: ModuleRecord = ModuleRecord {
    id: "tinybox",
    description: "Sandbox capability discovery through TinyBox",
    bus_name: "ai.tinyhumans.tinybox.Box",
    object_path: "/ai/tinyhumans/tinybox/Box",
    version: "0.1.14",
    release_url: "https://github.com/tinyhumansai/tinybox/releases/tag/v0.1.14",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinybox-0.1.14-ubuntu-24.04-x86_64.tar.gz",
            sha256: "828758f5b71ee003fbf7051eaaa3d04505dafa9cca2000ccb93aa2337401ac07",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinybox-0.1.14-ubuntu-24.04-arm64.tar.gz",
            sha256: "a9dea8c6d40a37249825c4a285466410801c9ed6eeb0fca62d7cd4e80e93467b",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinybox-0.1.14-ubuntu-22.04-x86_64.tar.gz",
            sha256: "cd9c7e73829a9e21af33041f277b46d570fc0d1ff85662aa76b918c538568a68",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinybox-0.1.14-ubuntu-22.04-arm64.tar.gz",
            sha256: "0792b52caeedc8911fefa46bb7b03aee693ff156874f2590b3cb38d94b7491a5",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinybox-0.1.14-macos-26-arm64.tar.gz",
            sha256: "15a44ebc3e10d1a3c68679330e38a971ecedf0552bf71984ed534f712ce9142c",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinybox-0.1.14-macos-26-x86_64.tar.gz",
            sha256: "9147dde505cc142e0e6b58ce948139c5b5710bcefc37e2db7bf1b3f385923c97",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinybox-0.1.14-macos-15-arm64.tar.gz",
            sha256: "e6c62a77dcce726ad0e9f474c0d21364af0fd51fbc7503f5647c30e9c65e513c",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinybox-0.1.14-macos-15-x86_64.tar.gz",
            sha256: "c1bb50e01571ecc3d5888ee36dcc97d95514cf9618127e11bb56965535b29f6d",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinybox-0.1.14-windows-2025-x86_64.zip",
            sha256: "bd4386e0b4d89117f83e095dafdda22a31f7637fe683a0d7fa0691c2380de92a",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinybox-0.1.14-windows-2022-x86_64.zip",
            sha256: "66a08bcaa7de36c444ecb8bd29cc10be241bfc4e85ef53d0f49a409ae0e0807d",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinybox-0.1.14-windows-11-arm64.zip",
            sha256: "b449b87b93c553cf00f64fb451faee847eb995652194cda54fe98f41d4ab1ef5",
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
    version: "0.1.10",
    release_url: "https://github.com/tinyhumansai/tinychannels/releases/tag/v0.1.10",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinychannels-module-0.1.10-ubuntu-24.04-x86_64.tar.gz",
            sha256: "7b4cf71ded75876d770850223245d30a909788a298af858e858c50bce4f87e95",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinychannels-module-0.1.10-ubuntu-24.04-arm64.tar.gz",
            sha256: "2a755576a874f349d1827ee1afc698c6f9521843c1f06c1197ce62fec59568ed",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinychannels-module-0.1.10-ubuntu-22.04-x86_64.tar.gz",
            sha256: "c7f4943f01fa1af2b46637cb6907e79067b6c6763e2a9850398123beb4aa459f",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinychannels-module-0.1.10-ubuntu-22.04-arm64.tar.gz",
            sha256: "32bd9760d8a9717c87a9e5aa4a832171aea8ae8b2189035ca2d943c155f8e34a",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinychannels-module-0.1.10-macos-26-arm64.tar.gz",
            sha256: "6947bb8a0c011c40a4ccce797451add6be7caca923501b39ef2d7ead874fdc77",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinychannels-module-0.1.10-macos-26-x86_64.tar.gz",
            sha256: "ecce989844432a3610dabdaf29d94bb60c0fb8ff0fed29a50755fd3e42493007",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinychannels-module-0.1.10-macos-15-arm64.tar.gz",
            sha256: "bd536e55835aab04495441ac19302d0f99e7366c77960e84b3308cfd8780cc9b",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinychannels-module-0.1.10-macos-15-x86_64.tar.gz",
            sha256: "95cc8906bc73d27709df6022988ef2337ce36fec9ab68daf07d9ddff8ed9651a",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinychannels-module-0.1.10-windows-2025-x86_64.zip",
            sha256: "a272609902aa01692a11198696d0d25c8acc6f52e2f231237a5f9ddc70166b8c",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinychannels-module-0.1.10-windows-2022-x86_64.zip",
            sha256: "17ce261f8a9b772d2c8a4265bb1b7c22e1687073b7898a9ab9b9f54b81e979a9",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinychannels-module-0.1.10-windows-11-arm64.zip",
            sha256: "3bfcf3159c2bd7be84a9e8cddc5cc5f5a68b9742ccfe3d9e7fbc59558c0a8eb6",
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
    version: "0.2.2",
    release_url: "https://github.com/tinyhumansai/tinyhosts/releases/tag/v0.2.2",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyhosts-0.2.2-ubuntu-24.04-x86_64.tar.gz",
            sha256: "92e4176fc3e4d23f0d6b8395a6596ca7f548602eedff9a08746e1a934f7a91a7",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyhosts-0.2.2-ubuntu-24.04-arm64.tar.gz",
            sha256: "de5c2cd3b0ba8f350b658664bbff0937bcc4121451f8e706401a76c2b3c785a9",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyhosts-0.2.2-ubuntu-22.04-x86_64.tar.gz",
            sha256: "8a61cd86e6c3812a84f0a3b473221352fd6f4b2045dd7b5dbef43cfad2fa189b",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyhosts-0.2.2-ubuntu-22.04-arm64.tar.gz",
            sha256: "2cbe7797290b4cc16be08a3885b7abbc2a870f2822cee8f746dd1c2ecc74d796",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyhosts-0.2.2-macos-26-arm64.tar.gz",
            sha256: "afeba2bb84316f6e0bfdbae55dea3a4214e9b2bdf5a623139239d5b2f94b4a51",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyhosts-0.2.2-macos-26-x86_64.tar.gz",
            sha256: "8a23e7d910b47b24c4609f46af62d827c4439439a0b9b25e83356a53f7027695",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyhosts-0.2.2-macos-15-arm64.tar.gz",
            sha256: "d04c0778b253a6dce67d3bbdf54a02e1df594e664b29fab6aabbefbfad63dd77",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyhosts-0.2.2-macos-15-x86_64.tar.gz",
            sha256: "38842da342ad502b7a3eca6f0d10fe32bb5bfbe023ec4d39069934e61b133134",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyhosts-0.2.2-windows-2025-x86_64.zip",
            sha256: "afadbc834a2c0e77021365113a2af1d52c55159e98aebd0340d4c10a6b2532ac",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyhosts-0.2.2-windows-2022-x86_64.zip",
            sha256: "34d9302cf474e1f0e383fbd9d30b2da567de1b02458b6bfa9309671adddb361a",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyhosts-0.2.2-windows-11-arm64.zip",
            sha256: "27f1bd2da3b6bde602e0a3f12c104ef8569636d82643365c402e9777b081d4ad",
        },
    ],
    load: LoadPolicy::Lazy,
};
