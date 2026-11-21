//! The check of the crib against the tools and the sheets a job needs, and the command line that runs it (the exit codes, the sheet commands).

use eh3d_tool_manager::check::{check, parse_needs, NeedKind, NEEDS_HEADER};
use eh3d_tool_manager::model::{Crib, Kind, Tool};
use eh3d_tool_manager::sheet::Sheet;
use eh3d_tool_manager::store;
use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_eh3d-tools");

fn shop() -> Crib {
    let mut c = Crib::default();
    c.add(Tool::new("drill 0.8", Kind::Drill, 0.8)).unwrap();
    c.add(Tool::new("mill 2.0", Kind::EndMill, 2.0)).unwrap();
    c.add(Tool::new("beam", Kind::Laser, 0.05)).unwrap();
    let mut worn = Tool::new("old bit", Kind::Drill, 1.0);
    worn.max_holes = 10;
    worn.holes = 10;
    c.add(worn).unwrap();
    let mut foil = Sheet::new("foil 0.12", "stainless", 0.12);
    foil.reduction_mm = 0.03;
    c.add_sheet(foil).unwrap();
    c
}

fn needs(rows: &[&str]) -> Vec<eh3d_tool_manager::check::Need> {
    parse_needs(&format!("{NEEDS_HEADER}\n{}\n", rows.join("\n")), 100).unwrap()
}

#[test]
fn a_check_finds_the_tools_and_sheets_a_job_needs_and_names_what_is_missing() {
    let c = shop();
    let ok = needs(&[
        "drill 0.8,drill,0.8",
        "MILL 2.0,end-mill,2.005",
        "beam,laser,",
        "foil 0.12,sheet,0.12",
        "drill 0.8,,",
    ]);
    let r = check(&c, &ok, 0.01);
    assert!(r.passed(), "{}", r.text());
    assert!(r.text().ends_with("CHECK=PASS\n"));
    assert_eq!((r.looked_at, r.needs), (5, 5));

    let bad = needs(&[
        "drill 0.9,drill,0.9",
        "mill 2.0,drill,",
        "mill 2.0,end-mill,2.5",
        "foil 0.12,sheet,0.2",
        "foil 0.99,sheet,",
        "foil 0.12,,",
    ]);
    let r = check(&c, &bad, 0.01);
    assert!(!r.passed());
    let text = r.text();
    assert!(text.ends_with("CHECK=FAIL\n"));
    for wanted in [
        "need on row 2: there is no tool called 'drill 0.9'",
        "need on row 3: the tool 'mill 2.0' is a end-mill, not a drill",
        "need on row 4: the tool 'mill 2.0' has a diameter of 2 mm, not 2.5 mm",
        "need on row 5: the sheet 'foil 0.12' is 0.12 mm thick, not 0.2 mm",
        "need on row 6: there is no sheet called 'foil 0.99'",
        "need on row 7: there is no tool called 'foil 0.12' (there is a sheet with that name: write the kind sheet)",
    ] {
        assert!(text.contains(wanted), "{wanted}\n{text}");
    }
    assert_eq!(r.problems.len(), 6);
}

#[test]
fn a_tool_to_replace_is_a_warning_and_the_tolerance_is_the_one_given() {
    let c = shop();
    let r = check(&c, &needs(&["old bit,drill,1.0"]), 0.01);
    assert!(r.passed());
    assert!(r.warnings[0].contains("to be replaced"), "{:?}", r.warnings);
    // 0.8 against 0.85 is another tool for a tolerance of 0.01 and the same one for 0.1
    let n = needs(&["drill 0.8,drill,0.85"]);
    assert!(!check(&c, &n, 0.01).passed());
    assert!(check(&c, &n, 0.1).passed());
}

#[test]
fn a_crib_that_does_not_agree_with_itself_does_not_pass() {
    // a sheet with no kerf of its own needs a laser tool to take it from
    let mut c = Crib::default();
    c.add_sheet(Sheet::new("foil", "steel", 0.1)).unwrap();
    let r = check(&c, &[], 0.01);
    assert!(!r.passed());
    assert!(r.problems[0].contains("no kerf_mm"), "{:?}", r.problems);
    c.add(Tool::new("beam", Kind::Laser, 0.05)).unwrap();
    assert!(check(&c, &[], 0.01).passed());
    // a laser wider than any kerf does not count
    let mut wide = Crib::default();
    wide.add_sheet(Sheet::new("foil", "steel", 0.1)).unwrap();
    wide.add(Tool::new("wide beam", Kind::Laser, 3.0)).unwrap();
    assert!(!check(&wide, &[], 0.01).passed());
    // a kerf of its own needs no laser
    let mut own = Crib::default();
    let mut s = Sheet::new("foil", "steel", 0.1);
    s.kerf_mm = 0.04;
    own.add_sheet(s).unwrap();
    assert!(check(&own, &[], 0.01).passed());
    // a crib built by hand with a repeated number or a value that makes no sense is found
    let mut broken = shop();
    broken.sheets[0].thickness_mm = -1.0;
    broken.tools[1].id = broken.tools[0].id;
    let r = check(&broken, &[], 0.01);
    assert!(r
        .problems
        .iter()
        .any(|p| p.contains("sheet 1") && p.contains("thickness_mm")));
    assert!(r.problems.iter().any(|p| p.contains("used twice")));
}

#[test]
fn a_table_of_needs_that_is_damaged_names_the_row() {
    let head = NEEDS_HEADER;
    for (text, wanted) in [
        (String::new(), "empty"),
        ("name,kind\n".to_string(), "exactly"),
        (
            format!("{head}\na,drill\n"),
            "row 2: it has 2 fields, not 3",
        ),
        (format!("{head}\n,drill,1\n"), "row 2: the name"),
        (
            format!("{head}\na,drill,1\nb,saw,1\n"),
            "row 3: 'saw' is not a kind",
        ),
        (format!("{head}\na,drill,wide\n"), "row 2: size_mm 'wide'"),
        (format!("{head}\na,drill,0\n"), "row 2: size_mm '0'"),
        (format!("{head}\na,drill,NaN\n"), "row 2: size_mm"),
        (format!("{head}\na,\"drill\n"), "row 2: a quote is opened"),
    ] {
        let e = parse_needs(&text, 100).unwrap_err();
        assert!(e.contains(wanted), "{text:?}: {e}");
    }
    let many = format!("{head}\na,,\nb,,\nc,,\n");
    assert!(parse_needs(&many, 2).unwrap_err().contains("3 rows"));
    let n = needs(&["a,sheet,0.1", "b,,", "c,Laser,0.05"]);
    assert_eq!(n[0].kind, NeedKind::Sheet);
    assert_eq!(n[1].kind, NeedKind::AnyTool);
    assert_eq!(n[2].kind, NeedKind::Tool(Kind::Laser));
    assert_eq!((n[0].size_mm, n[1].size_mm, n[2].row), (Some(0.1), None, 4));
}

fn dir(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("dd_{name}"));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn run(d: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(BIN)
        .args(args)
        .current_dir(d)
        .env("EH3D_TOOLS_DIR", d)
        .output()
        .expect("the program runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn the_command_ends_with_code_2_when_the_check_is_not_passed_and_1_when_it_cannot_run() {
    let d = dir("exit_codes");
    let crib = d.join("crib.json");
    store::save(&shop(), &crib).unwrap();
    let crib_arg = crib.to_str().unwrap();
    let good = d.join("good.csv");
    std::fs::write(
        &good,
        format!("{NEEDS_HEADER}\ndrill 0.8,drill,0.8\nfoil 0.12,sheet,0.12\n"),
    )
    .unwrap();
    let bad = d.join("bad.csv");
    std::fs::write(
        &bad,
        format!("{NEEDS_HEADER}\ndrill 0.8,drill,0.8\nmissing,drill,1\n"),
    )
    .unwrap();

    let (code, out, _) = run(&d, &["check", crib_arg, "--needs", good.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("CHECK=PASS"));
    let (code, out, _) = run(&d, &["check", crib_arg]);
    assert_eq!(code, 0, "{out}");
    let (code, out, err) = run(&d, &["check", crib_arg, "--needs", bad.to_str().unwrap()]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(out.contains("CHECK=FAIL") && out.contains("no tool called 'missing'"));
    assert!(err.contains("verification not passed"));
    // a file that cannot be read, a table that is damaged and a command with nothing to check are errors, not a check that was not passed
    for args in [
        vec!["check", "no-such-file.json"],
        vec!["check", crib_arg, "--needs", "no-such-table.csv"],
        vec!["check"],
        vec!["nonsense"],
    ] {
        let (code, _, err) = run(&d, &args);
        assert_eq!(code, 1, "{args:?}: {err}");
    }
    std::fs::write(&bad, "name,kind\n").unwrap();
    let (code, _, err) = run(&d, &["check", crib_arg, "--needs", bad.to_str().unwrap()]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("bad.csv"), "{err}");
}

#[test]
fn the_sheet_commands_add_list_export_import_and_remove_and_leave_the_tools_alone() {
    let d = dir("sheet_commands");
    let crib = d.join("crib.json");
    let c = crib.to_str().unwrap();
    assert_eq!(run(&d, &["init", c]).0, 0);
    assert_eq!(run(&d, &["add", c, "beam", "laser", "0.05"]).0, 0);
    let (code, out, err) = run(
        &d,
        &[
            "add-sheet",
            c,
            "foil",
            "stainless",
            "0.12",
            "--reduction",
            "0.03",
            "--kerf",
            "0.05",
            "--min-aspect",
            "1.5",
            "--min-area",
            "0.66",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("sheet 1 added"));
    assert!(run(&d, &["add-sheet", c, "foil", "steel", "0.1"])
        .2
        .contains("already"));
    assert!(run(&d, &["add-sheet", c, "x", "steel", "0"])
        .2
        .contains("thickness_mm"));
    let (_, out, _) = run(&d, &["list-sheets", c]);
    assert!(
        out.contains("foil") && out.contains("stainless") && out.contains("0.12"),
        "{out}"
    );
    // the table of tools has no sheet; the table of sheets has it
    let (_, tools, _) = run(&d, &["export-csv", c]);
    assert!(!tools.contains("sheet") && tools.contains("beam"));
    let (_, table, _) = run(&d, &["export-sheets-csv", c]);
    assert!(
        table.starts_with("id,name,kind,material,thickness_mm")
            && table.contains(",sheet,stainless,0.12,0.03,0.05,1.5,0.66"),
        "{table}"
    );
    // an import of the table of tools keeps the sheets
    let tools_file = d.join("tools.csv");
    std::fs::write(&tools_file, &tools).unwrap();
    assert_eq!(
        run(&d, &["import-csv", c, tools_file.to_str().unwrap()]).0,
        0
    );
    assert!(run(&d, &["list-sheets", c]).1.contains("foil"));
    // an import of the table of sheets replaces the sheets and keeps the tools
    let sheets_file = d.join("sheets.csv");
    std::fs::write(&sheets_file, "id,name,kind,material,thickness_mm,reduction_mm,kerf_mm,min_aspect_ratio,min_area_ratio\n7,paper,sheet,card,0.3,0,0.1,0,0\n").unwrap();
    let (code, _, err) = run(&d, &["import-sheets-csv", c, sheets_file.to_str().unwrap()]);
    assert_eq!(code, 0, "{err}");
    let (_, out, _) = run(&d, &["list-sheets", c]);
    assert!(out.contains("paper") && !out.contains("foil"), "{out}");
    assert!(run(&d, &["list", c]).1.contains("beam"));
    // the number of the sheet that was taken out is not given again
    assert!(run(&d, &["remove-sheet", c, "7"])
        .1
        .contains("not given again"));
    assert!(run(&d, &["list-sheets", c]).1.contains("no sheet"));
    let (_, out, _) = run(
        &d,
        &["add-sheet", c, "again", "steel", "0.1", "--kerf", "0.05"],
    );
    assert!(out.contains("sheet 8 added"), "{out}");
    assert!(
        run(&d, &["import-sheets-csv", c, tools_file.to_str().unwrap()])
            .2
            .contains("table of tools")
    );
    assert!(run(&d, &["remove-sheet", c, "99"])
        .2
        .contains("no sheet 99"));
    // a tool profile is only in a build with the machining database
    #[cfg(not(feature = "common-machining"))]
    assert!(run(&d, &["profile", c, "1", "wood/balsa"])
        .2
        .contains("common-machining"));
}
