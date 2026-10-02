//! Registry records for the `tinyruntime` router and its Node.js and Python
//! sidecars.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The `tinyruntime` module: the runtime router.
///
/// Resolves a language runtime, installs one when the host has none, reuses one
/// when it does, and runs code on a bounded pool of warm interpreter processes.
/// It is a router: on its own it knows no languages, and it routes to the two
/// provider records below.
///
/// Lazy, because a host that never runs a skill, a flow step, or a `node_exec`
/// should not pay a download and a `dlopen` for the ability to.
///
/// The digests below are v0.2.9's, taken verbatim from that release's
/// `checksum.toml`. Until it existed this record carried no assets at all and
/// the module was reachable only from a developer build named by
/// `modules.local` or found on `OPENHUMAN_MODULE_PATH` — so on any machine that
/// had not built it, the runtime domain was a set of tools that could not run.
pub(crate) const TINYRUNTIME: ModuleRecord = ModuleRecord {
    id: "tinyruntime",
    description: "Language runtime resolution, installation, and pooled execution",
    bus_name: "ai.tinyhumans.runtime.Runtime",
    object_path: "/ai/tinyhumans/runtime/Runtime",
    version: "0.2.9",
    release_url: "https://github.com/tinyhumansai/tinyruntime/releases/tag/v0.2.9",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyruntime-0.2.9-ubuntu-24.04-x86_64.tar.gz",
            sha256: "499840f1101521c2cc3bd7b515b13adc46ef2fdd8e09fb14729c7b03c857c4e5",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyruntime-0.2.9-ubuntu-24.04-arm64.tar.gz",
            sha256: "7c6ec983576c8624e647ce208d801e7c181e34d9c2eb5ff1b23d1fa8fa072149",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyruntime-0.2.9-ubuntu-22.04-x86_64.tar.gz",
            sha256: "58cd6561abaf3cfd4b2ed1a61086d679b2b469c60b67de21baceeac748ecce6d",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyruntime-0.2.9-ubuntu-22.04-arm64.tar.gz",
            sha256: "88b6950fba1b77a8e50cfa62d3fabbf49cfcacac328da55fffc7c0cd513670fa",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyruntime-0.2.9-macos-26-arm64.tar.gz",
            sha256: "9966db3a782d2f8ebcf00c0404522ad8eb23ac09473e5b84b7755be319e52da5",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyruntime-0.2.9-macos-26-x86_64.tar.gz",
            sha256: "0049b39b09f0d46ac0359f607e6b40531aae453732c508236587858d2e478a3e",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyruntime-0.2.9-macos-15-arm64.tar.gz",
            sha256: "12cb452485ecb69022e004d7741dbcb4842219532cef27c296087944d2225838",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyruntime-0.2.9-macos-15-x86_64.tar.gz",
            sha256: "8737c618f98ba395ea8be684a89e24e688ae8eefebffecce946c72fbfc23a18f",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyruntime-0.2.9-windows-2025-x86_64.zip",
            sha256: "880280175dc06df740add7b31752c5997f7e4d08c63eb227da960d94c69b2d27",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyruntime-0.2.9-windows-2022-x86_64.zip",
            sha256: "20e03a1499e5c8d878705073d2bc58c8820d86141bc211eaa67b50d3cdc66940",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyruntime-0.2.9-windows-11-arm64.zip",
            sha256: "50bb5edce0e0ad3bc83106ea27040998ffb7f5df31153993b3fdf9bfd4ee1b29",
        },
    ],
    load: LoadPolicy::Lazy,
};

/// The `tinyruntime-nodejs` module: the Node.js half of the router's knowledge.
///
/// Answers which host interpreters count, which archive nodejs.org publishes for
/// this machine, where the binaries land, and what a warm Node worker is. It
/// installs nothing itself.
///
/// It implements the shared `ai.tinyhumans.runtime.Provider` interface but
/// serves at its own object path, because two modules cannot claim one bus name
/// and tinybus derives the path from the name.
///
/// Lazy, and loaded by the same call that loads the router: a language is only
/// worth its `dlopen` when something asks for that language.
///
/// Released from its own repository (own version line) against the router's source pin; see scripts/ci/module-provider-pins.json.
pub(crate) const TINYRUNTIME_NODEJS: ModuleRecord = ModuleRecord {
    id: "tinyruntime-nodejs",
    description: "Node.js runtime provider for tinyruntime",
    bus_name: "ai.tinyhumans.runtime.nodejs.Provider",
    object_path: "/ai/tinyhumans/runtime/nodejs/Provider",
    version: "0.2.5",
    release_url: "https://github.com/tinyhumansai/tinyruntime-nodejs/releases/tag/v0.2.5",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyruntime-nodejs-0.2.5-ubuntu-24.04-x86_64.tar.gz",
            sha256: "d2ce00a1c3669d12f5fa16febf49bd4bda4f73e3a64b45066e72a448e1a4ca0f",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyruntime-nodejs-0.2.5-ubuntu-24.04-arm64.tar.gz",
            sha256: "6ffa7cddb7cf2432f7d23eea0e3ca837d4ca3c4ad397d8ddf6b58f00271283fa",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyruntime-nodejs-0.2.5-ubuntu-22.04-x86_64.tar.gz",
            sha256: "4650d56e620e1f3065b4c3f5cfac897367d2c3f68f79afdfe50d5d8550ea7433",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyruntime-nodejs-0.2.5-ubuntu-22.04-arm64.tar.gz",
            sha256: "3d271fbdfb286a8c719ddb01a5e3ee4697e0b13447ddebdb4f2cddf960946816",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyruntime-nodejs-0.2.5-macos-26-arm64.tar.gz",
            sha256: "688e690457101e9387205906608d0c8decbb87abc4332675c855a855103bbe0f",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyruntime-nodejs-0.2.5-macos-26-x86_64.tar.gz",
            sha256: "8e0866d1d3051b389104da567a3d92010f4f42aa3b411e84ee360d611d3b5036",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyruntime-nodejs-0.2.5-macos-15-arm64.tar.gz",
            sha256: "9e7d8122a6c2af05dc56e9524072d74ae6be7648e9d41ce4ed4e21df8b857c0c",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyruntime-nodejs-0.2.5-macos-15-x86_64.tar.gz",
            sha256: "50e671633cacc6c0a004a5fb4df011130da4be4c34f0800e5bdc1fb8ce48864a",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyruntime-nodejs-0.2.5-windows-2025-x86_64.zip",
            sha256: "6f9ce6e8ff5bd33f66c5b800fe095d9fb58f3dade968abb13404615eaa103315",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyruntime-nodejs-0.2.5-windows-2022-x86_64.zip",
            sha256: "7352cc4545865e2b9cb51fb6ba3fadad273db420a82e16dc9d753eac41bf95cc",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyruntime-nodejs-0.2.5-windows-11-arm64.zip",
            sha256: "ebfba2183f569e677abdae41ed603c787b68e0de200b3d2d04d8dfd9d87b4d51",
        },
    ],
    load: LoadPolicy::Lazy,
};

/// The `tinyruntime-python` module: the Python half of the router's knowledge.
///
/// Answers which host interpreters count, which standalone build to install, and
/// what a warm Python worker is. It installs nothing itself.
///
/// Released from its own repository (own version line) against the router's source pin; see scripts/ci/module-provider-pins.json.
pub(crate) const TINYRUNTIME_PYTHON: ModuleRecord = ModuleRecord {
    id: "tinyruntime-python",
    description: "Python runtime provider for tinyruntime",
    bus_name: "ai.tinyhumans.runtime.python.Provider",
    object_path: "/ai/tinyhumans/runtime/python/Provider",
    version: "0.2.5",
    release_url: "https://github.com/tinyhumansai/tinyruntime-python/releases/tag/v0.2.5",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyruntime-python-0.2.5-ubuntu-24.04-x86_64.tar.gz",
            sha256: "25f0fb4ee0b9f2cab2f683a1267540931cac4752d6765f06319d676c202373cf",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyruntime-python-0.2.5-ubuntu-24.04-arm64.tar.gz",
            sha256: "e1ef22670150d412b461fde1ce4e9a170c07cdddddd31f38c68805c6778ed6de",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyruntime-python-0.2.5-ubuntu-22.04-x86_64.tar.gz",
            sha256: "1b53f82bb40fca324aa91aa4d28133132600f9ad6ebf2f75128d96776d8c5254",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyruntime-python-0.2.5-ubuntu-22.04-arm64.tar.gz",
            sha256: "3493c946bcebd0c04c888eecf049250258034b69496bbfd99e4bc61861c0cd8f",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyruntime-python-0.2.5-macos-26-arm64.tar.gz",
            sha256: "8b474084738b00e0ea6307f2916ec1e36e01cec2b4a674915745a0624cbcf673",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyruntime-python-0.2.5-macos-26-x86_64.tar.gz",
            sha256: "a77a6f58e9e1eedbc45f2082243c75f782108d0746ce39dd2c5cc1d226958ac7",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyruntime-python-0.2.5-macos-15-arm64.tar.gz",
            sha256: "0bf8248c6feda1da5faf1ff12962f2c41f295915a6ff793a4bdca284a34b7326",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyruntime-python-0.2.5-macos-15-x86_64.tar.gz",
            sha256: "dc20154834817171182a55c697745e07c1906fb5c7e7508e30746f51b004ca42",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyruntime-python-0.2.5-windows-2025-x86_64.zip",
            sha256: "4703779380a060a9508210d0c94d84bb07ddb91b225acba2088a0e063dfa27c5",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyruntime-python-0.2.5-windows-2022-x86_64.zip",
            sha256: "0d9768e775fe2a13fbcff98a4de4cb52fb7b8a86e6f319ef26087f947a657249",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyruntime-python-0.2.5-windows-11-arm64.zip",
            sha256: "c1e7fc2f2c28e003fd43e57dd36bc1277ec06db8f595bfd1d8c33ee0dd1da45c",
        },
    ],
    load: LoadPolicy::Lazy,
};
