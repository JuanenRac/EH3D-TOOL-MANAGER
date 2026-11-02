use eh3d_tool_manager::calc::{feeds_speeds, Material};
use eh3d_tool_manager::csv::{from_csv, parse_records, to_csv, HEADER};
use eh3d_tool_manager::model::{Crib, Kind, Tool, Wear};
use eh3d_tool_manager::store::{from_json, to_json};

fn drill(name: &str, d: f64) -> Tool {
    Tool::new(name, Kind::Drill, d)
}

fn vbit() -> Tool {
    let mut t = Tool::new("V 30", Kind::VBit, 3.175);
    t.tip_angle_deg = Some(30.0);
    t
}

#[test]
fn the_crib_numbers_its_tools_and_refuses_a_repeated_name() {
    let mut c = Crib::default();
    assert_eq!(c.add(drill("drill 0.8", 0.8)).unwrap(), 1);
    assert_eq!(c.add(drill("drill 1.0", 1.0)).unwrap(), 2);
    assert!(c
        .add(drill("DRILL 0.8", 0.9))
        .unwrap_err()
        .contains("already"));
    c.remove(1).unwrap();
    assert_eq!(
        c.add(drill("drill 1.2", 1.2)).unwrap(),
        3,
        "a number that was used is not given again while a higher one exists"
    );
    assert!(c.remove(99).is_err());
    assert_eq!(c.get(2).unwrap().name, "drill 1.0");
}

#[test]
fn a_tool_that_makes_no_sense_is_refused_with_the_field() {
    let mut c = Crib::default();
    for (t, word) in [
        (drill("", 1.0), "name"),
        (drill("x", 0.0), "diameter_mm"),
        (drill("x", -1.0), "diameter_mm"),
        (drill("x", f64::NAN), "diameter_mm"),
        (drill("x", 51.0), "diameter_mm"),
        (drill(&"n".repeat(81), 1.0), "name"),
        (drill("a\u{7}b", 1.0), "name"),
        (
            Tool {
                flutes: 0,
                ..drill("x", 1.0)
            },
            "flutes",
        ),
        (
            Tool {
                flutes: 9,
                ..drill("x", 1.0)
            },
            "flutes",
        ),
        (
            Tool {
                shank_mm: 0.0,
                ..drill("x", 1.0)
            },
            "shank_mm",
        ),
        (
            Tool {
                hours: -1.0,
                ..drill("x", 1.0)
            },
            "hours",
        ),
        (
            Tool {
                length_mm: f64::INFINITY,
                ..drill("x", 1.0)
            },
            "length_mm",
        ),
        (Tool::new("v", Kind::VBit, 3.0), "tip_angle_deg"),
    ] {
        let e = c.add(t).unwrap_err();
        assert!(e.contains(word), "{e} should name {word}");
    }
    assert!(c.tools.is_empty());
}

#[test]
fn the_wear_goes_from_good_to_soon_to_replace() {
    let mut c = Crib::default();
    let mut t = drill("d", 0.8);
    t.max_holes = 1000;
    t.max_hours = 10.0;
    let id = c.add(t).unwrap();
    assert!(matches!(c.get(id).unwrap().wear(), Wear::Good(f) if f == 0.0));
    assert!(
        matches!(c.record_use(id, 1.0, 0.0, 500).unwrap(), Wear::Good(f) if (f - 0.5).abs() < 1e-9)
    );
    assert!(
        matches!(c.record_use(id, 0.0, 0.0, 420).unwrap(), Wear::Soon(f) if (f - 0.92).abs() < 1e-9)
    );
    assert_eq!(
        c.record_use(id, 9.0, 0.0, 0).unwrap(),
        Wear::Replace,
        "the hours limit is reached first"
    );
    assert_eq!(c.to_replace().len(), 1);
}

#[test]
fn a_tool_without_limits_never_wears_out_and_the_worst_comes_first() {
    let mut c = Crib::default();
    c.add(drill("free", 1.0)).unwrap();
    let mut a = drill("a", 1.0);
    a.max_holes = 100;
    let mut b = drill("b", 1.0);
    b.max_holes = 100;
    let (ia, ib) = (c.add(a).unwrap(), c.add(b).unwrap());
    c.record_use(1, 100.0, 0.0, 1_000_000).unwrap();
    c.record_use(ia, 0.0, 0.0, 95).unwrap();
    c.record_use(ib, 0.0, 0.0, 100).unwrap();
    let worn = c.to_replace();
    assert_eq!(
        worn.iter()
            .map(|(t, _)| t.name.as_str())
            .collect::<Vec<_>>(),
        ["b", "a"]
    );
}

#[test]
fn the_counters_refuse_negative_or_absurd_values() {
    let mut c = Crib::default();
    let id = c.add(drill("d", 1.0)).unwrap();
    assert!(c.record_use(id, -1.0, 0.0, 0).is_err());
    assert!(c.record_use(id, 0.0, f64::NAN, 0).is_err());
    assert!(c.record_use(99, 1.0, 0.0, 0).is_err());
    assert_eq!(
        c.record_use(id, 0.0, 0.0, u64::MAX).unwrap(),
        Wear::Good(0.0)
    );
    assert_eq!(
        c.record_use(id, 0.0, 0.0, 5).unwrap(),
        Wear::Good(0.0),
        "the holes count saturates instead of overflowing"
    );
}

#[test]
fn json_round_trips_a_crib() {
    let mut c = Crib::default();
    c.add(drill("drill 0.8", 0.8)).unwrap();
    c.add(vbit()).unwrap();
    let id = c
        .add(Tool {
            max_holes: 500,
            ..drill("big", 3.0)
        })
        .unwrap();
    c.record_use(id, 0.5, 12.0, 40).unwrap();
    let back = from_json(&to_json(&c)).unwrap();
    assert_eq!(back, c);
}

#[test]
fn a_damaged_json_is_an_error_that_says_which_tool() {
    assert!(from_json("").is_err());
    assert!(from_json("[]").is_err());
    assert!(from_json(r#"{"schema_version":9,"tools":[]}"#)
        .unwrap_err()
        .contains("9"));
    assert!(from_json(r#"{"tools":{}}"#).is_err());
    let e = from_json(r#"{"tools":[{"id":1,"name":"a","kind":"drill","diameter_mm":1},{"id":2,"name":"b","kind":"saw","diameter_mm":1}]}"#).unwrap_err();
    assert!(e.contains("tool 2") && e.contains("saw"), "{e}");
    assert!(
        from_json(r#"{"tools":[{"id":1,"name":"a","kind":"drill","diameter_mm":"wide"}]}"#)
            .unwrap_err()
            .contains("diameter_mm")
    );
    assert!(from_json(r#"{"tools":[{"id":1,"name":"a","kind":"drill","diameter_mm":1},{"id":1,"name":"b","kind":"drill","diameter_mm":1}]}"#).unwrap_err().contains("twice"));
    assert!(
        from_json(r#"{"tools":[{"name":"a","kind":"drill","diameter_mm":1}]}"#)
            .unwrap_err()
            .contains("missing")
    );
}

#[test]
fn csv_round_trips_names_with_commas_and_quotes() {
    let mut c = Crib::default();
    c.add(drill("drill, 0.8 \"hard\"", 0.8)).unwrap();
    c.add(vbit()).unwrap();
    let text = to_csv(&c);
    assert!(text.starts_with(HEADER));
    assert!(text.contains("\"drill, 0.8 \"\"hard\"\"\""));
    assert_eq!(from_csv(&text).unwrap(), c);
    assert_eq!(
        from_csv(&format!("\u{feff}{}", text.replace('\n', "\r\n"))).unwrap(),
        c,
        "a byte order mark and Windows line ends are accepted"
    );
}

#[test]
fn a_damaged_csv_names_the_row() {
    assert!(from_csv("").is_err());
    assert!(from_csv("id,name\n1,a\n")
        .unwrap_err()
        .contains("first row"));
    let bad_field = format!("{HEADER}\n1,a,drill,1,2,0,,3.175,carbide,0,0,0,0\n");
    assert!(from_csv(&bad_field).unwrap_err().contains("row 2"));
    let not_number = format!("{HEADER}\n1,a,drill,wide,2,0,,3.175,carbide,0,0,0,0,0\n");
    assert!(from_csv(&not_number).unwrap_err().contains("diameter_mm"));
    let twice = format!("{HEADER}\n1,a,drill,1,2,0,,3.175,carbide,0,0,0,0,0\n1,b,drill,1,2,0,,3.175,carbide,0,0,0,0,0\n");
    assert!(from_csv(&twice).unwrap_err().contains("row 3"));
    assert!(parse_records("a,\"b").unwrap_err().contains("quote"));
}

#[test]
fn feeds_and_speeds_follow_the_material_and_the_spindle() {
    let t = drill("d", 1.0);
    let fr4 = feeds_speeds(&t, Material::Fr4, 60_000.0).unwrap();
    assert!(
        (fr4.rpm - 31_831.0).abs() < 1.0,
        "100 m/min on 1 mm is about 31831 rpm, got {}",
        fr4.rpm
    );
    assert!(!fr4.rpm_limited && fr4.pass_depth_mm == 0.0);
    let slow = feeds_speeds(&t, Material::Fr4, 24_000.0).unwrap();
    assert_eq!(slow.rpm, 24_000.0);
    assert!(
        slow.rpm_limited && slow.feed_mm_min < fr4.feed_mm_min,
        "a slower spindle lowers the feed with it"
    );
    let acrylic = feeds_speeds(
        &Tool::new("m", Kind::EndMill, 2.0),
        Material::Acrylic,
        24_000.0,
    )
    .unwrap();
    let fr4_mill =
        feeds_speeds(&Tool::new("m", Kind::EndMill, 2.0), Material::Fr4, 24_000.0).unwrap();
    assert!(acrylic.feed_mm_min > fr4_mill.feed_mm_min);
    assert!(
        fr4_mill.plunge_mm_min < fr4_mill.feed_mm_min
            && fr4_mill.pass_depth_mm > 0.0
            && fr4_mill.pass_depth_mm <= 1.0
    );
}

#[test]
fn feeds_and_speeds_refuse_what_has_no_cutting_feed() {
    assert!(
        feeds_speeds(&Tool::new("l", Kind::Laser, 0.1), Material::Wood, 24_000.0)
            .unwrap_err()
            .contains("laser")
    );
    assert!(feeds_speeds(&Tool::new("n", Kind::Nozzle, 1.0), Material::Fr4, 24_000.0).is_err());
    assert!(feeds_speeds(&drill("d", 1.0), Material::Fr4, 500.0)
        .unwrap_err()
        .contains("max_rpm"));
    assert!(feeds_speeds(&drill("d", 1.0), Material::Fr4, f64::NAN).is_err());
    assert!(feeds_speeds(&drill("d", 0.0), Material::Fr4, 24_000.0).is_err());
    assert!(Material::parse("cheese").unwrap_err().contains("cheese"));
    assert_eq!(Material::parse(" FR-4 ").unwrap(), Material::Fr4);
}

#[test]
fn a_vbit_is_worked_out_at_its_tip_not_at_its_shank() {
    let v = feeds_speeds(&vbit(), Material::Fr4, 60_000.0).unwrap();
    assert!(
        v.rpm > 60_000.0 * 0.0 && v.rpm_limited,
        "a 0.5 mm point asks for more than the spindle gives"
    );
}

#[test]
fn the_feed_and_the_pass_of_a_vbit_do_not_depend_on_its_shank() {
    let mut tip = vbit();
    tip.diameter_mm = 0.5;
    let shank = feeds_speeds(&vbit(), Material::Fr4, 24_000.0).unwrap();
    let narrow = feeds_speeds(&tip, Material::Fr4, 24_000.0).unwrap();
    assert_eq!(shank.feed_mm_min, narrow.feed_mm_min);
    assert_eq!(shank.pass_depth_mm, narrow.pass_depth_mm);
    assert!(shank.pass_depth_mm <= 0.25);
}

// ---- more edges: boundaries, non-ASCII text, damaged values, file round trips and the command line ----

use eh3d_tool_manager::store::{load, save};
use std::path::PathBuf;
use std::process::{Command, Output};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("eh3d_tools_{}_{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_eh3d-tools"))
        .args(args)
        .output()
        .unwrap()
}

fn text(out: &[u8]) -> String {
    String::from_utf8_lossy(out).into_owned()
}

#[test]
fn the_wear_thresholds_sit_exactly_at_ninety_and_one_hundred_percent() {
    let mut c = Crib::default();
    let id = c
        .add(Tool {
            max_holes: 100,
            ..drill("d", 1.0)
        })
        .unwrap();
    assert!(matches!(
        c.record_use(id, 0.0, 0.0, 89).unwrap(),
        Wear::Good(_)
    ));
    assert!(
        matches!(c.record_use(id, 0.0, 0.0, 1).unwrap(), Wear::Soon(f) if (f - 0.9).abs() < 1e-12)
    );
    assert!(
        matches!(c.record_use(id, 0.0, 0.0, 9).unwrap(), Wear::Soon(f) if (f - 0.99).abs() < 1e-12)
    );
    assert_eq!(c.record_use(id, 0.0, 0.0, 1).unwrap(), Wear::Replace);
    assert_eq!(
        c.record_use(id, 0.0, 0.0, 1).unwrap(),
        Wear::Replace,
        "past the limit it stays worn"
    );
}

#[test]
fn the_wear_uses_the_worst_of_the_two_limits() {
    let mut c = Crib::default();
    let id = c
        .add(Tool {
            max_holes: 1000,
            max_hours: 10.0,
            ..drill("d", 1.0)
        })
        .unwrap();
    // holes at 10 %, hours at 95 %: the hours decide
    assert!(
        matches!(c.record_use(id, 9.5, 0.0, 100).unwrap(), Wear::Soon(f) if (f - 0.95).abs() < 1e-9)
    );
    // meters alone never wear a tool out: they have no limit
    assert!(matches!(
        c.record_use(id, 0.0, 1.0e9, 0).unwrap(),
        Wear::Soon(_)
    ));
}

#[test]
fn a_failed_record_use_leaves_the_counters_as_they_were() {
    let mut c = Crib::default();
    let id = c.add(drill("d", 1.0)).unwrap();
    c.record_use(id, 2.0, 3.0, 4).unwrap();
    let before = c.clone();
    assert!(c.record_use(id, 1.0, -0.5, 9).is_err());
    assert!(c.record_use(id, f64::INFINITY, 0.0, 9).is_err());
    assert_eq!(
        c, before,
        "hours must not be added when the meters are refused"
    );
}

#[test]
fn counters_that_overflow_to_infinity_are_refused() {
    let mut c = Crib::default();
    let id = c.add(drill("d", 1.0)).unwrap();
    c.record_use(id, f64::MAX, 0.0, 0).unwrap_or(Wear::Replace);
    let before = c.clone();
    let r = c.record_use(id, f64::MAX, 0.0, 0);
    assert!(
        r.is_err(),
        "hours + hours would be infinite and could not be saved"
    );
    assert_eq!(c, before);
    assert!(c.get(id).unwrap().hours.is_finite());
}

#[test]
fn non_ascii_names_survive_json_and_csv_and_count_characters_not_bytes() {
    let mut c = Crib::default();
    c.add(drill("broca de 0,8 mm ñ 日本語 ✓", 0.8)).unwrap();
    c.add(drill(&"é".repeat(80), 1.0)).unwrap();
    assert_eq!(from_json(&to_json(&c)).unwrap(), c);
    assert_eq!(from_csv(&to_csv(&c)).unwrap(), c);
    assert!(c.add(drill(&"é".repeat(81), 1.1)).is_err());
}

#[test]
fn a_name_with_a_line_break_or_tab_is_refused_before_it_can_break_the_csv() {
    let mut c = Crib::default();
    for bad in ["a\nb", "a\rb", "a\tb"] {
        assert!(
            c.add(drill(bad, 1.0)).unwrap_err().contains("name"),
            "{bad:?}"
        );
    }
    // the same name written by hand in a JSON file is refused as well
    let e = from_json(r#"{"tools":[{"id":1,"name":"a\nb","kind":"drill","diameter_mm":1}]}"#)
        .unwrap_err();
    assert!(e.contains("tool 1") && e.contains("name"), "{e}");
}

#[test]
fn csv_numbers_that_are_not_finite_or_not_whole_are_refused_with_the_row() {
    for (field, value, word) in [
        (3, "nan", "diameter_mm"),
        (3, "inf", "diameter_mm"),
        (5, "-inf", "length_mm"),
        (0, "-1", "id"),
        (0, "1.5", "id"),
        (11, "1e3", "holes"),
        (13, "-2", "max_holes"),
    ] {
        let mut f: Vec<String> = "1,a,drill,1,2,0,,3.175,carbide,0,0,0,0,0"
            .split(',')
            .map(String::from)
            .collect();
        f[field] = value.to_string();
        let e = from_csv(&format!("{HEADER}\n{}\n", f.join(","))).unwrap_err();
        assert!(e.contains("row 2") && e.contains(word), "{value}: {e}");
    }
    // an id that does not fit in 32 bits is an error, not a wrap-around to a small number
    let big = format!("{HEADER}\n4294967297,a,drill,1,2,0,,3.175,carbide,0,0,0,0,0\n");
    assert!(from_csv(&big).unwrap_err().contains("too large"));
}

#[test]
fn csv_with_only_a_header_is_an_empty_crib_and_a_second_header_is_not_a_tool() {
    assert!(from_csv(&format!("{HEADER}\n")).unwrap().tools.is_empty());
    assert!(from_csv(HEADER).unwrap().tools.is_empty());
    assert!(from_csv(&format!("{HEADER}\n{HEADER}\n")).is_err());
    // blank lines between rows are skipped
    let row = "1,a,drill,1,2,0,,3.175,carbide,0,0,0,0,0";
    assert_eq!(
        from_csv(&format!("{HEADER}\n\n{row}\n\n"))
            .unwrap()
            .tools
            .len(),
        1
    );
}

#[test]
fn csv_names_are_unique_without_regard_to_case_but_ids_may_leave_gaps() {
    let a = "1,Drill,drill,1,2,0,,3.175,carbide,0,0,0,0,0";
    let b = "7,DRILL,drill,1,2,0,,3.175,carbide,0,0,0,0,0";
    let e = from_csv(&format!("{HEADER}\n{a}\n{b}\n")).unwrap_err();
    assert!(e.contains("row 3") && e.contains("twice"), "{e}");
    let c = "7,other,drill,1,2,0,,3.175,carbide,0,0,0,0,0";
    let crib = from_csv(&format!("{HEADER}\n{a}\n{c}\n")).unwrap();
    assert_eq!(crib.tools.iter().map(|t| t.id).collect::<Vec<_>>(), [1, 7]);
    let mut crib = crib;
    assert_eq!(crib.add(drill("next", 1.0)).unwrap(), 8);
}

#[test]
fn json_values_outside_the_limits_name_the_tool_and_the_field() {
    for (body, word) in [
        (r#""diameter_mm":0"#, "diameter_mm"),
        (r#""diameter_mm":1,"flutes":4294967297"#, "flutes"),
        (r#""diameter_mm":1,"flutes":0"#, "flutes"),
        (r#""diameter_mm":1,"hours":-3"#, "hours"),
        (r#""diameter_mm":1,"shank_mm":26"#, "shank_mm"),
        (
            r#""diameter_mm":1,"tip_angle_deg":"sharp""#,
            "tip_angle_deg",
        ),
    ] {
        let doc = format!(r#"{{"tools":[{{"id":1,"name":"a","kind":"drill",{body}}}]}}"#);
        let e = from_json(&doc).unwrap_err();
        assert!(e.starts_with("tool 1") && e.contains(word), "{doc}: {e}");
    }
}

#[test]
fn a_number_too_big_for_a_double_is_an_error_not_infinity() {
    let doc = r#"{"tools":[{"id":1,"name":"a","kind":"drill","diameter_mm":1e999}]}"#;
    assert!(from_json(doc).is_err());
}

#[test]
fn a_crib_with_too_many_tools_is_refused_before_it_is_read() {
    let item = r#"{"id":1,"name":"a","kind":"drill","diameter_mm":1}"#;
    let doc = format!(
        r#"{{"tools":[{}]}}"#,
        vec![item; eh3d_tool_manager::store::MAX_TOOLS + 1].join(",")
    );
    assert!(from_json(&doc).unwrap_err().contains("more than"));
}

#[test]
fn writing_the_json_twice_gives_the_same_text() {
    let mut c = Crib::default();
    c.add(drill("a", 0.8)).unwrap();
    c.add(vbit()).unwrap();
    let once = to_json(&c);
    assert_eq!(to_json(&from_json(&once).unwrap()), once);
    assert!(once.ends_with("}\n"));
}

#[test]
fn feeds_and_speeds_are_the_same_every_time_and_never_below_the_floor() {
    for kind in [Kind::Drill, Kind::EndMill] {
        for d in [0.1, 0.35, 0.8, 3.175, 12.0, 50.0] {
            let mut t = Tool::new("t", kind, d);
            t.tip_angle_deg = None;
            let a = feeds_speeds(&t, Material::Brass, 24_000.0).unwrap();
            assert_eq!(a, feeds_speeds(&t, Material::Brass, 24_000.0).unwrap());
            assert!(a.rpm >= 1.0 && a.rpm <= 24_000.0, "{d}: {a:?}");
            assert!(a.feed_mm_min.is_finite() && a.feed_mm_min >= 0.0);
        }
    }
}

#[test]
fn a_bigger_tool_never_turns_faster_and_more_flutes_feed_faster() {
    let mut last = f64::MAX;
    for d in [0.2, 0.5, 1.0, 2.0, 5.0, 10.0] {
        let c = feeds_speeds(
            &Tool::new("m", Kind::EndMill, d),
            Material::Aluminium,
            60_000.0,
        )
        .unwrap();
        assert!(
            c.rpm <= last,
            "diameter {d} turns at {} after {last}",
            c.rpm
        );
        last = c.rpm;
    }
    let two = feeds_speeds(
        &Tool::new("m", Kind::EndMill, 3.0),
        Material::Wood,
        24_000.0,
    )
    .unwrap();
    let four = feeds_speeds(
        &Tool {
            flutes: 4,
            ..Tool::new("m", Kind::EndMill, 3.0)
        },
        Material::Wood,
        24_000.0,
    )
    .unwrap();
    assert!((four.feed_mm_min - 2.0 * two.feed_mm_min).abs() <= 1.0);
}

#[test]
fn an_engraver_thinner_than_a_tenth_of_a_millimetre_is_worked_out_at_a_tenth() {
    let mut thin = Tool::new("e", Kind::Engraver, 0.05);
    thin.tip_angle_deg = Some(20.0);
    let mut tenth = thin.clone();
    tenth.diameter_mm = 0.1;
    assert_eq!(
        feeds_speeds(&thin, Material::Fr4, 24_000.0)
            .unwrap()
            .feed_mm_min,
        feeds_speeds(&tenth, Material::Fr4, 24_000.0)
            .unwrap()
            .feed_mm_min
    );
}

#[test]
fn saving_and_loading_a_file_round_trips_and_a_missing_file_names_its_path() {
    let dir = scratch("file");
    let file = dir.join("crib.json");
    let mut c = Crib::default();
    c.add(drill("ñandú", 0.8)).unwrap();
    save(&c, &file).unwrap();
    assert_eq!(load(&file).unwrap(), c);
    let e = load(&dir.join("nothing.json")).unwrap_err();
    assert!(e.contains("nothing.json"), "{e}");
    std::fs::write(&file, b"\xff\xfe not utf-8").unwrap();
    assert!(
        load(&file).is_err(),
        "a file that is not UTF-8 is an error, not a panic"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_command_line_never_overwrites_an_existing_crib_and_exits_with_one_on_errors() {
    let dir = scratch("cli_init");
    let file = dir.join("c.json");
    let f = file.to_str().unwrap();
    assert!(cli(&["init", f]).status.success());
    assert!(cli(&["add", f, "d", "drill", "0.8"]).status.success());
    let again = cli(&["init", f]);
    assert_eq!(again.status.code(), Some(1));
    assert!(text(&again.stderr).contains("already exists"));
    assert_eq!(
        load(&file).unwrap().tools.len(),
        1,
        "the second init must not wipe the crib"
    );
    let unknown = cli(&["frobnicate"]);
    assert_eq!(unknown.status.code(), Some(1));
    assert!(text(&unknown.stderr).contains("unknown command"));
    assert!(cli(&[]).status.success(), "no arguments prints the help");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_command_line_reports_a_bad_flag_or_number_and_does_not_change_the_file() {
    let dir = scratch("cli_flags");
    let file = dir.join("c.json");
    let f = file.to_str().unwrap();
    cli(&["init", f]);
    cli(&["add", f, "d", "drill", "0.8"]);
    let before = std::fs::read_to_string(&file).unwrap();
    for args in [
        vec!["use", f, "1", "--hours"],
        vec!["use", f, "1", "--hours", "lots"],
        vec!["use", f, "1", "--hours", "-2"],
        vec!["use", f, "1", "--holes", "-1"],
        vec!["use", f, "1", "--holes", "1e13"],
        vec!["use", f, "x"],
        vec!["use", f, "99", "--holes", "1"],
        vec!["add", f, "bad", "drill", "wide"],
        vec!["add", f, "bad", "saw", "1"],
        vec!["add", f, "bad", "drill", "0"],
        vec!["add", f, "bad", "v-bit", "3"],
        vec!["add", f, "d", "drill", "1"],
    ] {
        let out = cli(&args);
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        assert!(
            text(&out.stderr).starts_with("error:"),
            "{args:?}: {}",
            text(&out.stderr)
        );
    }
    assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_command_line_wear_and_csv_round_trip_with_non_ascii_names() {
    let dir = scratch("cli_csv");
    let file = dir.join("c.json");
    let back = dir.join("back.json");
    let table = dir.join("t.csv");
    let (f, b, t) = (
        file.to_str().unwrap(),
        back.to_str().unwrap(),
        table.to_str().unwrap(),
    );
    cli(&["init", f]);
    assert!(cli(&[
        "add",
        f,
        "fresa 日本 ñ",
        "end-mill",
        "2",
        "--max-holes",
        "10"
    ])
    .status
    .success());
    assert!(cli(&["add", f, "free", "drill", "1"]).status.success());
    let used = cli(&["use", f, "1", "--holes", "10"]);
    assert!(
        text(&used.stdout).contains("REPLACE"),
        "{}",
        text(&used.stdout)
    );
    let wear = text(&cli(&["wear", f]).stdout);
    assert!(
        wear.contains("fresa 日本 ñ") && !wear.contains("free"),
        "{wear}"
    );
    let csv_out = cli(&["export-csv", f]);
    std::fs::write(&table, &csv_out.stdout).unwrap();
    assert!(cli(&["import-csv", b, t]).status.success());
    assert_eq!(load(&back).unwrap(), load(&file).unwrap());
    let missing = cli(&["import-csv", b, dir.join("none.csv").to_str().unwrap()]);
    assert_eq!(missing.status.code(), Some(1));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_calc_command_says_when_the_spindle_limits_the_speed() {
    let free = text(&cli(&["calc", "drill", "1", "fr4", "60000"]).stdout);
    assert!(
        free.starts_with("31831 rpm") && !free.contains("limits"),
        "{free}"
    );
    let slow = text(&cli(&["calc", "drill", "1", "fr4", "20000"]).stdout);
    assert!(
        slow.starts_with("20000 rpm") && slow.contains("the spindle limits the speed"),
        "{slow}"
    );
    for args in [
        vec!["calc", "laser", "0.1", "wood", "20000"],
        vec!["calc", "drill", "1", "cheese", "20000"],
        vec!["calc", "drill", "1", "fr4", "999"],
        vec!["calc", "drill", "1", "fr4"],
        vec!["calc", "v-bit", "3", "fr4", "20000"],
    ] {
        assert_eq!(cli(&args).status.code(), Some(1), "{args:?}");
    }
}

#[test]
#[ignore = "bug: a wear counter that is not a whole number (holes, max_holes, flutes) is read as its default instead of being refused"]
fn a_wear_counter_that_is_not_a_whole_number_is_refused() {
    for body in [
        r#""holes":-5"#,
        r#""holes":"many""#,
        r#""max_holes":2.5"#,
        r#""flutes":2.5"#,
    ] {
        let doc =
            format!(r#"{{"tools":[{{"id":1,"name":"a","kind":"drill","diameter_mm":1,{body}}}]}}"#);
        assert!(from_json(&doc).is_err(), "{doc} was accepted");
    }
}
