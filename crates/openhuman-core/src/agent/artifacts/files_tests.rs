use std::path::{Path, PathBuf};

use tempfile::TempDir;

use super::*;
use crate::agent::artifacts::store::{
    create_artifact, delete_artifact, fail_artifact, finalize_artifact, list_artifacts,
    read_artifact_bytes, save_artifact_meta, REGENERATE_TARGET_ID,
};
use crate::agent::artifacts::types::ArtifactKind;

async fn create_ready(
    workspace: &Path,
    files_dir: &Path,
    title: &str,
    bytes: &[u8],
) -> ArtifactMeta {
    let (meta, path) = create_artifact(workspace, files_dir, ArtifactKind::Document, title, "txt")
        .await
        .expect("create_artifact");
    tokio::fs::write(&path, bytes).await.unwrap();
    finalize_artifact(workspace, &meta.id, bytes.len() as u64)
        .await
        .expect("finalize_artifact")
}

fn legacy_meta(id: &str, path: &str) -> ArtifactMeta {
    ArtifactMeta {
        id: id.to_string(),
        kind: ArtifactKind::Document,
        title: "Legacy".to_string(),
        path: path.to_string(),
        file: None,
        file_root: None,
        size_bytes: 3,
        status: ArtifactStatus::Ready,
        created_at: chrono::Utc::now(),
        error: None,
        thread_id: None,
        tool_call_id: None,
    }
}

fn names_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[tokio::test]
async fn new_artifact_is_written_to_the_files_folder_not_the_workspace() {
    let tmp = TempDir::new().unwrap();
    let workspace = tmp.path().join("ws");
    let files_dir = tmp.path().join("Files");

    let (meta, path) = create_artifact(
        &workspace,
        &files_dir,
        ArtifactKind::Presentation,
        "Q3 Deck",
        "pptx",
    )
    .await
    .unwrap();

    assert_eq!(path, files_dir.join("q3-deck.pptx"));
    assert!(path.is_file(), "the name is claimed with a placeholder");
    assert_eq!(meta.path, "q3-deck.pptx");
    assert_eq!(meta.file.as_deref(), Some(path.to_string_lossy().as_ref()));
    assert_eq!(
        meta.file_root.as_deref(),
        Some(files_dir.to_string_lossy().as_ref())
    );
    // Metadata stays hidden in the workspace; no bytes land beside it.
    assert_eq!(
        names_in(&workspace.join("artifacts").join(&meta.id)),
        vec!["meta.json"]
    );
}

#[tokio::test]
async fn files_folder_is_created_on_first_use() {
    let tmp = TempDir::new().unwrap();
    let files_dir = tmp.path().join("not").join("yet").join("Files");
    create_artifact(
        tmp.path(),
        &files_dir,
        ArtifactKind::Document,
        "Notes",
        "docx",
    )
    .await
    .unwrap();
    assert!(files_dir.is_dir());
}

#[tokio::test]
async fn a_repeated_title_gets_a_numbered_name_instead_of_overwriting() {
    let tmp = TempDir::new().unwrap();
    let files_dir = tmp.path().join("Files");
    let first = create_ready(tmp.path(), &files_dir, "Report", b"one").await;
    let second = create_ready(tmp.path(), &files_dir, "Report", b"two").await;

    assert_eq!(first.path, "report.txt");
    assert_eq!(second.path, "report (2).txt");
    assert_eq!(
        read_artifact_bytes(tmp.path(), &first.id).await.unwrap(),
        b"one"
    );
    assert_eq!(
        read_artifact_bytes(tmp.path(), &second.id).await.unwrap(),
        b"two"
    );
}

/// The files folder is shared by every account on the OS user; metadata is
/// not. Two accounts saving the same title must get distinct files, and each
/// account's listing must show only its own records.
#[tokio::test]
async fn two_accounts_share_the_files_folder_without_clobbering_or_leaking() {
    let tmp = TempDir::new().unwrap();
    let files_dir = tmp.path().join("Files");
    let ws_a = tmp.path().join("users").join("a").join("workspace");
    let ws_b = tmp.path().join("users").join("b").join("workspace");

    let a = create_ready(&ws_a, &files_dir, "Budget", b"account a").await;
    let b = create_ready(&ws_b, &files_dir, "Budget", b"account b").await;

    assert_ne!(a.file, b.file);
    assert_eq!(
        read_artifact_bytes(&ws_a, &a.id).await.unwrap(),
        b"account a"
    );
    assert_eq!(
        read_artifact_bytes(&ws_b, &b.id).await.unwrap(),
        b"account b"
    );

    let (listed_b, total_b) = list_artifacts(&ws_b, 0, 50, None).await.unwrap();
    assert_eq!(total_b, 1);
    assert_eq!(listed_b[0].id, b.id);
    assert!(
        get_artifact(&ws_b, &a.id).await.is_err(),
        "B cannot open A's record"
    );
}

#[tokio::test]
async fn regenerate_overwrites_the_file_it_owns() {
    let tmp = TempDir::new().unwrap();
    let files_dir = tmp.path().join("Files");
    let original = create_ready(tmp.path(), &files_dir, "Deck", b"v1").await;

    let (again, path) = REGENERATE_TARGET_ID
        .scope(original.id.clone(), async {
            create_artifact(
                tmp.path(),
                &files_dir,
                ArtifactKind::Document,
                "Deck",
                "txt",
            )
            .await
        })
        .await
        .unwrap();

    assert_eq!(again.id, original.id);
    assert_eq!(again.file, original.file);
    assert_eq!(path, PathBuf::from(original.file.clone().unwrap()));
    assert_eq!(names_in(&files_dir), vec!["deck.txt"]);
}

#[tokio::test]
async fn a_failed_generation_leaves_no_empty_file_behind() {
    let tmp = TempDir::new().unwrap();
    let files_dir = tmp.path().join("Files");
    let (meta, path) = create_artifact(
        tmp.path(),
        &files_dir,
        ArtifactKind::Document,
        "Doomed",
        "docx",
    )
    .await
    .unwrap();
    assert!(path.is_file());

    fail_artifact(tmp.path(), &meta.id, "engine exploded")
        .await
        .unwrap();

    assert!(!path.exists(), "the zero-byte placeholder is removed");
}

#[tokio::test]
async fn delete_removes_the_file_and_the_record() {
    let tmp = TempDir::new().unwrap();
    let files_dir = tmp.path().join("Files");
    let meta = create_ready(tmp.path(), &files_dir, "Gone", b"bye").await;
    let file = PathBuf::from(meta.file.clone().unwrap());

    delete_artifact(tmp.path(), &meta.id).await.unwrap();

    assert!(!file.exists());
    assert!(!tmp.path().join("artifacts").join(&meta.id).exists());
}

#[tokio::test]
async fn a_file_removed_outside_openhuman_is_reported_missing() {
    let tmp = TempDir::new().unwrap();
    let files_dir = tmp.path().join("Files");
    let meta = create_ready(tmp.path(), &files_dir, "Moved", b"abc").await;
    std::fs::remove_file(meta.file.as_deref().unwrap()).unwrap();

    let err = read_artifact_bytes(tmp.path(), &meta.id).await.unwrap_err();
    assert!(err.contains("file missing"), "{err}");
    let err = resolve_ready_file(tmp.path(), &meta.id).await.unwrap_err();
    assert!(err.contains("file missing"), "{err}");
}

async fn tampered(workspace: &Path, file: &Path, file_root: &Path) -> String {
    let mut meta = legacy_meta("tampered", "x.txt");
    meta.file = Some(file.to_string_lossy().into_owned());
    meta.file_root = Some(file_root.to_string_lossy().into_owned());
    save_artifact_meta(workspace, &meta).await.unwrap();
    read_artifact_bytes(workspace, "tampered")
        .await
        .unwrap_err()
}

#[tokio::test]
async fn the_escape_guard_rejects_a_file_outside_its_folder() {
    let tmp = TempDir::new().unwrap();
    let files_dir = tmp.path().join("Files");
    std::fs::create_dir_all(&files_dir).unwrap();
    let secret = tmp.path().join("secret.txt");
    std::fs::write(&secret, b"private").unwrap();

    let err = tampered(tmp.path(), &secret, &files_dir).await;
    assert!(err.contains("escape guard"), "{err}");

    let dotdot = files_dir.join("..").join("secret.txt");
    let err = tampered(tmp.path(), &dotdot, &files_dir).await;
    assert!(err.contains("escape guard"), "{err}");
}

#[tokio::test]
async fn the_escape_guard_keeps_the_credential_store_floor() {
    let tmp = TempDir::new().unwrap();
    let ssh = tmp.path().join(".ssh");
    std::fs::create_dir_all(&ssh).unwrap();
    let key = ssh.join("id_ed25519");
    std::fs::write(&key, b"-----BEGIN").unwrap();

    // Even a record whose root claims the key's parent is refused.
    let err = tampered(tmp.path(), &key, tmp.path()).await;
    assert!(err.contains("escape guard"), "{err}");
}

#[cfg(unix)]
#[tokio::test]
async fn the_escape_guard_rejects_a_symlink_out_of_the_folder() {
    let tmp = TempDir::new().unwrap();
    let files_dir = tmp.path().join("Files");
    std::fs::create_dir_all(&files_dir).unwrap();
    let secret = tmp.path().join("secret.txt");
    std::fs::write(&secret, b"private").unwrap();
    let link = files_dir.join("innocent.txt");
    std::os::unix::fs::symlink(&secret, &link).unwrap();

    let err = tampered(tmp.path(), &link, &files_dir).await;
    assert!(err.contains("escape guard"), "{err}");
}

#[tokio::test]
async fn a_legacy_record_still_resolves_under_the_workspace() {
    let tmp = TempDir::new().unwrap();
    let meta = legacy_meta("legacy-1", "legacy-1/old.txt");
    save_artifact_meta(tmp.path(), &meta).await.unwrap();
    std::fs::write(tmp.path().join("artifacts/legacy-1/old.txt"), b"old").unwrap();

    assert_eq!(
        read_artifact_bytes(tmp.path(), "legacy-1").await.unwrap(),
        b"old"
    );
}

/// `root.join("../x").starts_with(root)` is true lexically, so the legacy
/// guard must reject `..` components rather than rely on `starts_with`.
#[tokio::test]
async fn a_legacy_path_with_parent_components_is_rejected() {
    let tmp = TempDir::new().unwrap();
    let workspace = tmp.path().join("ws");
    std::fs::write(tmp.path().join("secret.txt"), b"private").unwrap();
    let meta = legacy_meta("legacy-2", "../../secret.txt");
    save_artifact_meta(&workspace, &meta).await.unwrap();

    let err = read_artifact_bytes(&workspace, "legacy-2")
        .await
        .unwrap_err();
    assert!(err.contains("escapes artifacts root"), "{err}");
}
