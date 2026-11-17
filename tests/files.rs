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

#[test]
fn a_save_keeps_the_previous_crib_beside_it_and_leaves_no_temporary_file() {
    let dir = scratch("backup");
    let file = dir.join("crib.json");
    let mut crib = Crib::default();
    crib.add(Tool::new("Drill 0.8", Kind::Drill, 0.8)).unwrap();
    store::save(&crib, &file).unwrap();
    assert!(!dir.join("crib.json.bak").exists());
    let first = fs::read_to_string(&file).unwrap();
    crib.add(Tool::new("Drill 1.0", Kind::Drill, 1.0)).unwrap();
    store::save(&crib, &file).unwrap();
    assert_eq!(
        fs::read_to_string(dir.join("crib.json.bak")).unwrap(),
        first
    );
    assert_eq!(store::load(&file).unwrap().tools.len(), 2);
    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["crib.json", "crib.json.bak"]);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_restore_puts_back_the_backup_on_request_keeps_the_replaced_file_and_refuses_a_bad_backup() {
    let dir = scratch("restore");
    let file = dir.join("crib.json");
    let mut crib = Crib::default();
    crib.add(Tool::new("Drill 0.8", Kind::Drill, 0.8)).unwrap();
    store::save(&crib, &file).unwrap();
    crib.add(Tool::new("Drill 1.0", Kind::Drill, 1.0)).unwrap();
    store::save(&crib, &file).unwrap(); // the backup holds one tool
    fs::write(&file, b"{ broken").unwrap();
    // nothing happens by itself: the damaged file still fails to load
    assert!(store::load(&file).is_err());
    assert_eq!(store::restore_from_backup(&file, 1).unwrap(), 1);
    assert_eq!(store::load(&file).unwrap().tools.len(), 1);
    assert_eq!(
        fs::read(dir.join("crib.json.damaged")).unwrap(),
        b"{ broken"
    );
    assert!(dir.join("crib.json.bak").exists());
    // a backup that does not read is refused and the file stays as it is
    fs::write(dir.join("crib.json.bak"), b"nope").unwrap();
    let before = fs::read(&file).unwrap();
    assert!(store::restore_from_backup(&file, 1).is_err());
    assert_eq!(fs::read(&file).unwrap(), before);
    // no backup at all is refused with its code
    let none = dir.join("other.json");
    let why = store::restore_from_backup(&none, 1).unwrap_err();
    assert!(why.starts_with("file_unreadable"), "{why}");
    let _ = fs::remove_dir_all(&dir);
}
