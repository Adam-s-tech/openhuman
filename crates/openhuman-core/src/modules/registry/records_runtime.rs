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
/// The digests below are v0.2.8's, taken verbatim from that release's
/// `checksum.toml`. Until it existed this record carried no assets at all and
/// the module was reachable only from a developer build named by
/// `modules.local` or found on `OPENHUMAN_MODULE_PATH` — so on any machine that
/// had not built it, the runtime domain was a set of tools that could not run.
pub(crate) const TINYRUNTIME: ModuleRecord = ModuleRecord {
    id: "tinyruntime",
    description: "Language runtime resolution, installation, and pooled execution",
    bus_name: "ai.tinyhumans.runtime.Runtime",
    object_path: "/ai/tinyhumans/runtime/Runtime",
    version: "0.2.8",
    release_url: "https://github.com/tinyhumansai/tinyruntime/releases/tag/v0.2.8",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyruntime-0.2.8-ubuntu-24.04-x86_64.tar.gz",
            sha256: "826faf52d787694fc6f97d9224c99049c5616105c2b643144e1639909598d778",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyruntime-0.2.8-ubuntu-24.04-arm64.tar.gz",
            sha256: "97e6941750310258682a7644d69072d5e5e51e02cc7d353fbbaf086315d44eb5",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyruntime-0.2.8-ubuntu-22.04-x86_64.tar.gz",
            sha256: "e79bbca49f71c3d06a88061fd2b915e86fecaf52fe7c14b18c6dd897859ca82e",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyruntime-0.2.8-ubuntu-22.04-arm64.tar.gz",
            sha256: "0b030b1f336c14e2e9ff8f0247e60a4a063a421d50dbccb693353f85b5b45aa6",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyruntime-0.2.8-macos-26-arm64.tar.gz",
            sha256: "f5915a2afa813f36632386e55b9ac315ea4b20a4a37080e90d8a48e7c46ab0bf",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyruntime-0.2.8-macos-26-x86_64.tar.gz",
            sha256: "f946d24b3097f266eb0f4b7050dd54b16c69afd9b78c129bd3be3cc49ff724e0",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyruntime-0.2.8-macos-15-arm64.tar.gz",
            sha256: "4edb4773fb4239c1ca9e13ede7505dc25651d7ca74656861409a51c4a4e87ff6",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyruntime-0.2.8-macos-15-x86_64.tar.gz",
            sha256: "6cfc6a3a3fa18dc563febe9fe6f512818c8bac182f8fe9c238a83bad86a8a9e1",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyruntime-0.2.8-windows-2025-x86_64.zip",
            sha256: "2a0973f2428ecd7482f35ba5600d171743df8e9cac5b12df5096c230d90b0720",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyruntime-0.2.8-windows-2022-x86_64.zip",
            sha256: "8eaaebdcf4c814367d511d1eb42e3ae5d47294fe03427b13cf768d5577447373",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyruntime-0.2.8-windows-11-arm64.zip",
            sha256: "15e74e4dd338eceb2df9f0348b1af61c0d206a95db7b796605abca2daa581739",
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
