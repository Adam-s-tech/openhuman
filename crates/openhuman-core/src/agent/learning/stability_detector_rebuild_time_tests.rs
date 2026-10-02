use super::*;

/// Rebuilds that finish together leave the latest time stored, whatever
/// order their writes land in, and an earlier rebuild finishing late does
/// not move it back.
#[test]
fn the_stored_rebuild_time_only_moves_forward() {
    let workspace = tempfile::tempdir().unwrap();
    let path = workspace.path().join(REBUILD_STATE_FILE);
    store_rebuild_time(&path, 2_000.0).unwrap();
    store_rebuild_time(&path, 1_000.0).unwrap();
    assert_eq!(read_rebuild_time(&path), Some(2_000.0));

    let writers: Vec<_> = (0..16)
        .map(|i| {
            let path = path.clone();
            std::thread::spawn(move || store_rebuild_time(&path, 3_000.0 + f64::from(i)))
        })
        .collect();
    for writer in writers {
        writer.join().unwrap().unwrap();
    }
    assert_eq!(read_rebuild_time(&path), Some(3_015.0));
}
