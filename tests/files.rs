//! The size limit of the crib and CSV files that are read and the atomic write of the files that are written.

use eh3d_tool_manager::files::{
    mib, read_text_limited, write_atomic, NOT_TEXT, TOO_LARGE, UNREADABLE, UNWRITABLE,
};
use eh3d_tool_manager::model::{Crib, Kind, Tool};
use eh3d_tool_manager::store;
use std::fs;

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("eh3d_tools_files_{}_{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_file_within_the_limit_is_read_and_a_bigger_one_is_refused_with_its_code() {
    let dir = scratch("size");
    let file = dir.join("tools.csv");
    fs::write(&file, "id,name\n").unwrap();
    assert_eq!(read_text_limited(&file, 8).unwrap(), "id,name\n");
    let why = read_text_limited(&file, 7).unwrap_err();
    assert!(
        why.starts_with("file_too_large: ") && why.contains("tools.csv") && why.contains("7 bytes"),
        "{why}"
    );
    assert!(why.starts_with(TOO_LARGE));
    assert_eq!(mib(1), 1_048_576);
    assert_eq!(mib(u64::MAX), u64::MAX);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_missing_file_and_a_file_that_is_not_text_have_their_own_codes() {
    let dir = scratch("codes");
    let why = read_text_limited(&dir.join("nothing.json"), 10).unwrap_err();
    assert!(
        why.starts_with(UNREADABLE) && why.contains("nothing.json"),
        "{why}"
    );
    let file = dir.join("binary.csv");
    fs::write(&file, [0xff, 0xfe]).unwrap();
    let why = read_text_limited(&file, 10).unwrap_err();
    assert!(
        why.starts_with(NOT_TEXT) && why.contains("binary.csv"),
        "{why}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_crib_is_loaded_only_within_the_limit() {
    let dir = scratch("crib");
    let file = dir.join("crib.json");
    let mut crib = Crib::default();
    crib.add(Tool::new("Drill 0.8", Kind::Drill, 0.8)).unwrap();
    store::save(&crib, &file).unwrap();
    assert_eq!(store::load_with_limit(&file, 1).unwrap().tools.len(), 1);
    let size = fs::metadata(&file).unwrap().len();
    assert!(size < 1_048_576);
    fs::write(&file, vec![b' '; 1_048_577]).unwrap();
    let why = store::load_with_limit(&file, 1).unwrap_err();
    assert!(why.starts_with("file_too_large: "), "{why}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_file_is_replaced_whole_and_no_temporary_file_is_left() {
    let dir = scratch("atomic");
    let file = dir.join("crib.json");
    write_atomic(&file, b"first").unwrap();
    write_atomic(&file, b"second").unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "second");
    assert!(!dir.join("crib.json.tmp").exists());
    // a folder cannot be replaced by a file: the error has its code, the folder stays and the temporary file is gone
    let folder = dir.join("folder");
    fs::create_dir_all(&folder).unwrap();
    let why = write_atomic(&folder, b"x").unwrap_err();
    assert!(
        why.starts_with(UNWRITABLE) && why.contains("folder"),
        "{why}"
    );
    assert!(folder.is_dir());
    assert!(!dir.join("folder.tmp").exists());
    // a place that does not exist cannot be written
    let why = write_atomic(&dir.join("none").join("a.json"), b"x").unwrap_err();
    assert!(why.starts_with(UNWRITABLE), "{why}");
    let _ = fs::remove_dir_all(&dir);
}
