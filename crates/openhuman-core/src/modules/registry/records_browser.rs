//! TinyBrowser release assets, pinned from its published `v0.2.2` checksum.toml.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

pub(crate) const TINYBROWSER: ModuleRecord = ModuleRecord {
    id: "tinybrowser",
    description: "Chrome browser automation",
    bus_name: tinybrowser_bus::names::INTERFACE,
    object_path: tinybrowser_bus::names::OBJECT_PATH,
    version: "0.2.3",
    release_url: "https://github.com/tinyhumansai/tinybrowser/releases/tag/v0.2.3",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinybrowser-0.2.3-ubuntu-24.04-x86_64.tar.gz",
            sha256: "71f2186e5ca1ff23d47257ad068cb9f9c0f6e588227942dc2a5ef16f152903ec",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinybrowser-0.2.3-ubuntu-24.04-arm64.tar.gz",
            sha256: "21d8f95e4e2f802d7095d3203fb14472676231527f42a62398aa220389fc8b16",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinybrowser-0.2.3-ubuntu-22.04-x86_64.tar.gz",
            sha256: "7f4d035e7c963e551e89be9672bdc482af9dc6c29e1b675abb36ff6cc3554ad0",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinybrowser-0.2.3-ubuntu-22.04-arm64.tar.gz",
            sha256: "61078341d33f11778687666fc12c7b9117e163e74a39a806358e61debac148dd",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinybrowser-0.2.3-macos-26-arm64.tar.gz",
            sha256: "45e5f56e873e1259e0be1dd0c6f11e2d86ec8ff780eeef6594d3a65dd0ecd910",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinybrowser-0.2.3-macos-26-x86_64.tar.gz",
            sha256: "832e1397404edbebde17652ebe71bc0894b851a6a66476d999ebca760f696663",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinybrowser-0.2.3-macos-15-arm64.tar.gz",
            sha256: "6f2ba00360ccd3324b7c48dccaf86412d3e907794fe4c9bbf2a6c08cffcb6f23",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinybrowser-0.2.3-macos-15-x86_64.tar.gz",
            sha256: "9c6ad8f7b719aa293e4fa833c3af6ed418fc831a2fbfda3d6114d739d2fcc12c",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinybrowser-0.2.3-windows-2025-x86_64.zip",
            sha256: "cc3b3229b980c94b84ed007c1f1969f45fa37fbde27d7f4887183c7425ba8582",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinybrowser-0.2.3-windows-2022-x86_64.zip",
            sha256: "3e50c4d1477e8c698cfcb3a52a418173ead052e81a6549f812af7ffef300cd74",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinybrowser-0.2.3-windows-11-arm64.zip",
            sha256: "e33748c60f6b636aa1cba4a3560c34a8fbdf2119f1a592dd73aac120ebcee1d0",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinybrowser-0.2.3-fedora-43-x86_64.tar.gz",
            sha256: "45dfad71ff02b2b72449971235ad0c9833abdfba40f64d41a6364785f029b780",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinybrowser-0.2.3-fedora-43-arm64.tar.gz",
            sha256: "56a195f3c4136eea350ffe247effaaa5ad9b3adbbb408b97265363489565f596",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinybrowser-0.2.3-fedora-44-x86_64.tar.gz",
            sha256: "6aa6724275f01666954bde604116819c76e44dd1660506720bda74dab513078d",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinybrowser-0.2.3-fedora-44-arm64.tar.gz",
            sha256: "337e6beb0238e355dc58833b369d60eb51424a56588aa0751d66832e9b30fa7f",
        },
        PlatformAsset {
            host_key: "archlinux-rolling-x86_64",
            archive: "tinybrowser-0.2.3-archlinux-rolling-x86_64.tar.gz",
            sha256: "c2b423512e6ac6e552866e5aac231a890bf150a43275c989409db04c722969d5",
        },
    ],
    load: LoadPolicy::Lazy,
};
