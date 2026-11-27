//! The size limit of the crib and CSV files that are read and the atomic write of the files that are written.

use eh3d_tool_manager::files::{
    lock_for_edit, mib, read_text_limited, write_atomic, LOCKED, NOT_TEXT, TOO_LARGE, UNREADABLE,
    UNWRITABLE,
};
use eh3d_tool_manager::model::{Crib, Kind, Tool};
use eh3d_tool_manager::store;
use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::Duration;

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

#[test]
fn a_file_that_does_not_read_never_takes_the_place_of_a_good_backup() {
    let dir = scratch("good_backup");
    let file = dir.join("crib.json");
    let mut crib = Crib::default();
    crib.add(Tool::new("Drill 0.8", Kind::Drill, 0.8)).unwrap();
    store::save(&crib, &file).unwrap();
    crib.add(Tool::new("Drill 1.0", Kind::Drill, 1.0)).unwrap();
    store::save(&crib, &file).unwrap(); // the backup holds one tool
    let good = fs::read(dir.join("crib.json.bak")).unwrap();
    // the file gets damaged and is written over, as an import or a pull with --force does
    fs::write(&file, b"{ cut").unwrap();
    assert_eq!(
        store::keep_previous(&file).unwrap(),
        Some(dir.join("crib.json.damaged"))
    );
    store::save(&crib, &file).unwrap();
    assert_eq!(fs::read(dir.join("crib.json.bak")).unwrap(), good);
    assert_eq!(fs::read(dir.join("crib.json.damaged")).unwrap(), b"{ cut");
    assert_eq!(store::load(&file).unwrap(), crib);
    assert_eq!(store::keep_previous(&dir.join("none.json")).unwrap(), None);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn an_import_over_a_damaged_crib_keeps_both_the_good_backup_and_the_damaged_file() {
    let dir = scratch("import_damaged");
    let file = dir.join("crib.json");
    let table = dir.join("table.csv");
    let mut crib = Crib::default();
    crib.add(Tool::new("Drill 0.8", Kind::Drill, 0.8)).unwrap();
    store::save(&crib, &file).unwrap();
    store::save(&crib, &file).unwrap();
    let good = fs::read(dir.join("crib.json.bak")).unwrap();
    fs::write(&file, b"{ cut").unwrap();
    fs::write(&table, eh3d_tool_manager::csv::to_csv(&crib)).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_eh3d-tools"))
        .args([
            "import-csv",
            file.to_str().unwrap(),
            table.to_str().unwrap(),
        ])
        .env("EH3D_TOOLS_DIR", &dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let said = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(said.contains("crib.json.damaged"), "{said}");
    assert_eq!(fs::read(dir.join("crib.json.bak")).unwrap(), good);
    assert_eq!(fs::read(dir.join("crib.json.damaged")).unwrap(), b"{ cut");
    let _ = fs::remove_dir_all(&dir);
}

/// `eh3d-tools` with its settings in `dir`.
fn cli(dir: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eh3d-tools"));
    command.args(args).env("EH3D_TOOLS_DIR", dir);
    command
}

fn run(dir: &Path, args: &[&str]) -> Output {
    cli(dir, args).output().expect("the program runs")
}

#[test]
fn a_file_being_changed_is_locked_for_the_others_and_freed_when_its_holder_goes() {
    let dir = scratch("lock");
    let file = dir.join("crib.json");
    let held = lock_for_edit(&file, Duration::ZERO).unwrap();
    let why = lock_for_edit(&file, Duration::from_millis(60)).unwrap_err();
    assert!(
        why.starts_with(LOCKED) && why.contains("crib.json") && why.contains("lock_wait_s"),
        "{why}"
    );
    // another file has a lock of its own
    drop(lock_for_edit(&dir.join("other.json"), Duration::ZERO).unwrap());
    drop(held);
    // the file of the lock stays where it was and holds nobody back
    assert!(dir.join("crib.json.lock").is_file());
    drop(lock_for_edit(&file, Duration::ZERO).unwrap());
    let why = lock_for_edit(&dir.join("none").join("a.json"), Duration::ZERO).unwrap_err();
    assert!(why.starts_with(UNWRITABLE), "{why}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn every_command_that_changes_a_crib_waits_for_its_lock_and_the_ones_that_read_do_not() {
    let dir = scratch("lock_cli");
    fs::write(dir.join("tools.cfg"), "lock_wait_s = 0\n").unwrap();
    let file = dir.join("crib.json");
    let f = file.to_str().unwrap();
    assert!(run(&dir, &["init", f]).status.success());
    assert!(run(&dir, &["add", f, "A", "drill", "0.8"]).status.success());
    assert!(run(&dir, &["add-sheet", f, "S", "steel", "0.1"])
        .status
        .success());
    let table = dir.join("in.csv");
    fs::write(&table, run(&dir, &["export-csv", f]).stdout).unwrap();
    let sheets = dir.join("sheets.csv");
    fs::write(&sheets, run(&dir, &["export-sheets-csv", f]).stdout).unwrap();
    let (t, s) = (table.to_str().unwrap(), sheets.to_str().unwrap());
    let before = fs::read_to_string(&file).unwrap();
    let held = lock_for_edit(&file, Duration::ZERO).unwrap();
    for args in [
        vec!["add", f, "B", "drill", "0.9"],
        vec!["use", f, "1", "--holes", "5"],
        vec!["replace", f, "1"],
        vec!["remove", f, "1"],
        vec!["add-sheet", f, "T", "steel", "0.2"],
        vec!["remove-sheet", f, "1"],
        vec!["import-csv", f, t],
        vec!["import-sheets-csv", f, s],
        vec!["restore", f],
        vec!["init", f],
    ] {
        let out = run(&dir, &args);
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {err}");
        assert!(err.contains(LOCKED), "{args:?}: {err}");
    }
    assert_eq!(fs::read_to_string(&file).unwrap(), before);
    assert!(run(&dir, &["list", f]).status.success());
    drop(held);
    assert!(run(&dir, &["use", f, "1", "--holes", "5"]).status.success());
    // a crib that is not there gets no file beside it
    let none = dir.join("none.json");
    let out = run(&dir, &["add", none.to_str().unwrap(), "A", "drill", "0.8"]);
    assert!(String::from_utf8_lossy(&out.stderr).contains(UNREADABLE));
    assert!(!dir.join("none.json.lock").exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn programs_that_add_to_one_crib_at_the_same_time_lose_no_tool() {
    let dir = scratch("lock_many");
    fs::write(dir.join("tools.cfg"), "lock_wait_s = 60\n").unwrap();
    let file = dir.join("crib.json");
    let f = file.to_str().unwrap();
    assert!(run(&dir, &["init", f]).status.success());
    let running: Vec<_> = (0..12)
        .map(|i| {
            cli(&dir, &["add", f, &format!("tool {i}"), "drill", "0.8"])
                .stdout(Stdio::null())
                .spawn()
                .expect("the program starts")
        })
        .collect();
    for mut child in running {
        assert!(child.wait().unwrap().success());
    }
    assert_eq!(store::load(&file).unwrap().tools.len(), 12);
    let _ = fs::remove_dir_all(&dir);
}
