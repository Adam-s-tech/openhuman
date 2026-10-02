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
    version: "0.2.4",
    release_url: "https://github.com/tinyhumansai/tinyruntime-nodejs/releases/tag/v0.2.4",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyruntime-nodejs-0.2.4-ubuntu-24.04-x86_64.tar.gz",
            sha256: "1c88a1d2d9921c257c40406de57d0262330c4cfedbabac4db7795d409787f10e",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyruntime-nodejs-0.2.4-ubuntu-24.04-arm64.tar.gz",
            sha256: "7a7d35e1c22fbcad917cbc877ad5ab58b5764b9911fe2fba97eaf75abd6db344",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyruntime-nodejs-0.2.4-ubuntu-22.04-x86_64.tar.gz",
            sha256: "3a243b4f7d9ed9b1a63c20b548e1f99bdc3e41dd14d065081fa14f990ee5346d",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyruntime-nodejs-0.2.4-ubuntu-22.04-arm64.tar.gz",
            sha256: "d5e4c5f9df1116d1119aae3202facf9cd19bff6aa6a035a393f4c959254d7999",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyruntime-nodejs-0.2.4-macos-26-arm64.tar.gz",
            sha256: "141e54ae09f988bb69522eaf949ce44f3d832946610d16461633bf0287fc47d1",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyruntime-nodejs-0.2.4-macos-26-x86_64.tar.gz",
            sha256: "64860efdd2d4a4463313e3a890a5b64829863a76e12fb39338b551b7f41e773f",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyruntime-nodejs-0.2.4-macos-15-arm64.tar.gz",
            sha256: "5360d01d6ca50e9ab72053fff2efc58c2f3f2f2149dac3f6eba2237e94e59071",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyruntime-nodejs-0.2.4-macos-15-x86_64.tar.gz",
            sha256: "09f8bc3f86cfb4832c96063ced38e87246815fec4e6c842257a600398b56fd40",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyruntime-nodejs-0.2.4-windows-2025-x86_64.zip",
            sha256: "df51942d335aebcb3eaffd94c574a73a6481e5d8102b69ffd139c9efffbe7c5c",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyruntime-nodejs-0.2.4-windows-2022-x86_64.zip",
            sha256: "26e735d04526fb8021b0fa52f854b230bed39543c784a1f18af44808eddfb3d8",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyruntime-nodejs-0.2.4-windows-11-arm64.zip",
            sha256: "58c52fb1abd7d7d3ad814ee7179163f4b5a6e9b89bdcf97f9ad213f344a8ccc8",
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
    version: "0.2.4",
    release_url: "https://github.com/tinyhumansai/tinyruntime-python/releases/tag/v0.2.4",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyruntime-python-0.2.4-ubuntu-24.04-x86_64.tar.gz",
            sha256: "5ba3cf361783278056ad76072ae7862b6fa0be9ca0b1000e281fcf1bb49675c0",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyruntime-python-0.2.4-ubuntu-24.04-arm64.tar.gz",
            sha256: "51ce7df3793fbca5822d5e777df1617671315bd01ce2629ec7a7cb140941a1b7",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyruntime-python-0.2.4-ubuntu-22.04-x86_64.tar.gz",
            sha256: "48c21ede08c5bd8153656d6348d2eaf0d24c5e782ebe22233a7173ce8bc8013f",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyruntime-python-0.2.4-ubuntu-22.04-arm64.tar.gz",
            sha256: "e548cfd528f0aac9b44d53d2afe584dccd2fead24cbb67f95401b0132c066aa9",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyruntime-python-0.2.4-macos-26-arm64.tar.gz",
            sha256: "647858a385576143c743c468c36ed0ba8c3828449a2877494bda6e864049739f",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyruntime-python-0.2.4-macos-26-x86_64.tar.gz",
            sha256: "9648f7808ac2afb537839d7acc9acc25d7f932525602767fbf80e84dae0f6502",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyruntime-python-0.2.4-macos-15-arm64.tar.gz",
            sha256: "53fe640b15e616ace0a2bf721b1735ae5c0ba547de1533ee00bd5b1dffe872b0",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyruntime-python-0.2.4-macos-15-x86_64.tar.gz",
            sha256: "260a2bbf6c6895ea66a75a9d095d65befc5e1c6a095c36eac2bdfa8121d18936",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyruntime-python-0.2.4-windows-2025-x86_64.zip",
            sha256: "fa1ed8a79101549b98eac38c4cd9764db4f10b57a4583465eb71ea35486ba5a9",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyruntime-python-0.2.4-windows-2022-x86_64.zip",
            sha256: "d97b492a87451486fbe657b617b733e004db8df3cf30748dcda580794d4763f2",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyruntime-python-0.2.4-windows-11-arm64.zip",
            sha256: "50b31096589d57fafdff23e193548db09a95de1648f72188bae80b960fbc0235",
        },
    ],
    load: LoadPolicy::Lazy,
};
