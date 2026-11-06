//! The width of the cut of a V-bit: `2 * depth * tan(angle / 2) + flat`, as a command (`vbit`), as the diameter a V-bit works with in `calc --depth`, and with the angle and the flat of
//! `tools.cfg`. The values are worked out by hand.

use eh3d_tool_manager::calc::{feeds_speeds_traced_at, CalcParams, Material};
use eh3d_tool_manager::csv::{from_csv, HEADER};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A folder of its own for the settings of one test (the program reads `tools.cfg` from there), with the text of the file when there is one.
fn folder(name: &str, cfg: Option<&str>) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("eh3d-tools-vbit-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    if let Some(text) = cfg {
        std::fs::write(dir.join("tools.cfg"), text).unwrap();
    }
    dir
}

fn cli(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_eh3d-tools"))
        .args(args)
        .env("EH3D_TOOLS_DIR", dir)
        .output()
        .expect("the program runs")
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn the_vbit_command_gives_the_width_of_the_cut_with_the_defaults_of_60_degrees_and_a_flat_of_a_tenth(
) {
    let dir = folder("default", None);
    // 60 degrees at 0.5 mm: 2 x 0.5 x tan 30 = 0.5774, plus the flat of 0.1 mm = 0.6774
    let o = cli(&dir, &["vbit", "--depth", "0.5"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).starts_with("cut width 0.6774 mm"), "{}", out(&o));
    // 90 degrees and no flat at 1 mm: the width is twice the depth
    let o = cli(
        &dir,
        &["vbit", "--depth", "1", "--angle", "90", "--flat", "0"],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).starts_with("cut width 2.0000 mm"), "{}", out(&o));
    // no depth: only the flat
    let o = cli(&dir, &["vbit", "--depth", "0"]);
    assert!(out(&o).starts_with("cut width 0.1000 mm"), "{}", out(&o));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_angle_and_the_flat_of_tools_cfg_are_the_defaults_and_the_options_win() {
    let dir = folder("cfg", Some("v_bit_angle_deg = 90\nv_bit_flat_mm = 0\n"));
    let o = cli(&dir, &["vbit", "--depth", "1"]);
    assert!(out(&o).starts_with("cut width 2.0000 mm"), "{}", out(&o));
    // 60 degrees at 1 mm with no flat: 2 x tan 30 = 1.1547
    let o = cli(&dir, &["vbit", "--depth", "1", "--angle", "60"]);
    assert!(out(&o).starts_with("cut width 1.1547 mm"), "{}", out(&o));
    let o = cli(&dir, &["vbit", "--depth", "1", "--flat", "0.2"]);
    assert!(out(&o).starts_with("cut width 2.2000 mm"), "{}", out(&o));
    // a value of the file that makes no sense is an error that names its line
    std::fs::write(dir.join("tools.cfg"), "\nv_bit_angle_deg = 3\n").unwrap();
    let o = cli(&dir, &["vbit", "--depth", "1"]);
    assert!(!o.status.success());
    assert!(
        err(&o).contains("line 2") && err(&o).contains("v_bit_angle_deg"),
        "{}",
        err(&o)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_vbit_command_refuses_what_makes_no_sense_and_says_which_value() {
    let dir = folder("bad", None);
    for (args, word) in [
        (vec!["vbit"], "--depth"),
        (vec!["vbit", "--depth", "-1"], "depth_mm"),
        (vec!["vbit", "--depth", "101"], "depth_mm"),
        (vec!["vbit", "--depth", "deep"], "--depth"),
        (vec!["vbit", "--depth", "1", "--angle", "9"], "angle_deg"),
        (vec!["vbit", "--depth", "1", "--angle", "171"], "angle_deg"),
        (vec!["vbit", "--depth", "1", "--flat", "-0.1"], "flat_mm"),
        (vec!["vbit", "--depth", "1", "--flat", "5.5"], "flat_mm"),
        (vec!["vbit", "--depth", "1", "--oops"], "--oops"),
        (vec!["vbit", "--depth", "1", "more"], "more"),
    ] {
        let o = cli(&dir, &args);
        assert!(!o.status.success(), "{args:?}");
        assert!(err(&o).contains(word), "{args:?}: {}", err(&o));
    }
    // the ends of the ranges are allowed
    assert!(cli(
        &dir,
        &["vbit", "--depth", "0", "--angle", "10", "--flat", "0"]
    )
    .status
    .success());
    assert!(cli(
        &dir,
        &["vbit", "--depth", "100", "--angle", "170", "--flat", "5"]
    )
    .status
    .success());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn calc_with_a_depth_works_a_v_bit_at_the_width_of_the_cut() {
    let dir = folder("calc", None);
    let base = [
        "calc",
        "v-bit",
        "3.175",
        "aluminium",
        "100000",
        "--angle",
        "60",
    ];
    let plain = cli(&dir, &base);
    assert!(plain.status.success(), "{}", err(&plain));
    assert!(!out(&plain).contains("cut width"), "{}", out(&plain));
    let mut with = base.to_vec();
    with.extend(["--depth", "0.5"]);
    let o = cli(&dir, &with);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("V-bit cut width 0.6774 mm"), "{}", out(&o));
    // the speed follows the width: 200 m/min on 0.6774 mm turns faster than on the 0.5 mm of the point alone
    let rpm = |text: &str| text.split(' ').next().unwrap().parse::<f64>().unwrap();
    assert!(
        rpm(&out(&o)) < rpm(&out(&plain)),
        "{} {}",
        out(&o),
        out(&plain)
    );
    // a depth is for a V-bit or an engraver
    let o = cli(
        &dir,
        &["calc", "drill", "1", "fr4", "24000", "--depth", "0.5"],
    );
    assert!(!o.status.success());
    assert!(
        err(&o).contains("V-bit") && err(&o).contains("drill"),
        "{}",
        err(&o)
    );
    let o = cli(
        &dir,
        &[
            "calc", "v-bit", "3", "fr4", "24000", "--angle", "30", "--depth", "-1",
        ],
    );
    assert!(
        !o.status.success() && err(&o).contains("depth_mm"),
        "{}",
        err(&o)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_v_bit_read_from_an_old_table_with_its_angle_gets_its_width_and_one_without_still_loads() {
    // the tip angle is a column of the table that was always there: the 60 degrees of this row are what the width is worked out with
    let table = format!(
        "{HEADER}\n1,V60,v-bit,3.175,2,0,60,3.175,carbide,0,0,0,0,0\n2,Drill,drill,0.8,2,0,,3.175,carbide,0,0,0,0,0\n"
    );
    let crib = from_csv(&table).unwrap();
    assert_eq!(crib.tools[0].tip_angle_deg, Some(60.0));
    assert_eq!(crib.tools[1].tip_angle_deg, None);
    let (_, t) = feeds_speeds_traced_at(
        &crib.tools[0],
        Material::Wood,
        100_000.0,
        &CalcParams::default(),
        Some(0.5),
    )
    .unwrap();
    assert!((t.v_width_mm.unwrap() - 0.677_350).abs() < 1e-5);
    // the drill has no angle and no use for a depth
    assert!(feeds_speeds_traced_at(
        &crib.tools[1],
        Material::Wood,
        100_000.0,
        &CalcParams::default(),
        Some(0.5)
    )
    .is_err());
}
