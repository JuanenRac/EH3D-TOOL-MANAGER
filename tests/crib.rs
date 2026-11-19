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
        // no settings file of the person's computer changes the numbers
        .env("EH3D_TOOLS_DIR", env!("CARGO_TARGET_TMPDIR"))
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

#[test]
fn a_refused_counter_names_the_field_that_fails() {
    for (body, field) in [
        (r#""holes":-5"#, "holes"),
        (r#""holes":"many""#, "holes"),
        (r#""max_holes":-1"#, "max_holes"),
        (r#""flutes":2.5"#, "flutes"),
        (r#""flutes":"two""#, "flutes"),
    ] {
        let doc =
            format!(r#"{{"tools":[{{"id":1,"name":"a","kind":"drill","diameter_mm":1,{body}}}]}}"#);
        let err = from_json(&doc).unwrap_err();
        assert!(err.contains(field) && err.contains("tool 1"), "{err}");
    }
    let ok =
        r#"{"tools":[{"id":1,"name":"a","kind":"drill","diameter_mm":1,"holes":null,"flutes":3}]}"#;
    assert_eq!(from_json(ok).unwrap().tools[0].flutes, 3);
}

#[test]
fn the_command_line_refuses_counters_that_are_not_whole_numbers() {
    let dir = scratch("whole_flags");
    let file = dir.join("c.json");
    let f = file.to_str().unwrap();
    cli(&["init", f]);
    for args in [
        vec!["add", f, "a", "drill", "1", "--flutes", "2.9"],
        vec!["add", f, "a", "drill", "1", "--max-holes", "-5"],
        vec!["calc", "drill", "1", "fr4", "60000", "--flutes", "2.9"],
    ] {
        let out = cli(&args);
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        assert!(text(&out.stderr).contains("whole number"), "{args:?}");
    }
    assert!(cli(&[
        "add",
        f,
        "a",
        "drill",
        "1",
        "--flutes",
        "3",
        "--max-holes",
        "5"
    ])
    .status
    .success());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_last_number_does_not_wrap_round_and_the_crib_stays_readable() {
    let text = r#"{"schema_version":1,"tools":[{"id":4294967295,"name":"x","kind":"drill","diameter_mm":1}]}"#;
    let mut crib = from_json(text).unwrap();
    let e = crib.add(drill("y", 1.0)).unwrap_err();
    assert!(e.contains("too many tools"), "{e}");
    assert_eq!(crib.tools.len(), 1);
    assert!(crib.add(drill("z", 1.0)).is_err());
}

#[test]
fn saving_leaves_no_temporary_file_and_an_import_keeps_the_old_crib() {
    let dir = scratch("atomic");
    let file = dir.join("crib.json");
    let (f, table) = (file.to_str().unwrap(), dir.join("t.csv"));
    assert!(cli(&["init", f]).status.success());
    assert!(cli(&["add", f, "old", "drill", "1"]).status.success());
    assert!(!dir.join("crib.json.tmp").exists());
    let before = std::fs::read_to_string(&file).unwrap();
    let mut other = Crib::default();
    other.add(drill("new", 2.0)).unwrap();
    std::fs::write(&table, to_csv(&other)).unwrap();
    assert!(cli(&["import-csv", f, table.to_str().unwrap()])
        .status
        .success());
    assert_eq!(
        std::fs::read_to_string(dir.join("crib.json.bak")).unwrap(),
        before
    );
    assert_eq!(load(&file).unwrap().tools[0].name, "new");
    assert!(!dir.join("crib.json.tmp").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_crib_saved_with_a_byte_order_mark_is_read() {
    let text = "\u{feff}{\"schema_version\":1,\"tools\":[{\"id\":1,\"name\":\"x\",\"kind\":\"drill\",\"diameter_mm\":1}]}";
    assert_eq!(from_json(text).unwrap().tools.len(), 1);
}

// ---- a list with filters and an order, one tool in full, a worn tool replaced, a tool removed and the sums by kind ----

use eh3d_tool_manager::model::{Selection, Sort, KINDS};
use eh3d_tool_manager::sim::SimRng;

/// Four tools: 1 is at 95 % of its hours, 2 has no limits, 3 reached its holes and 4 is almost new.
fn shop() -> Crib {
    let mut c = Crib::default();
    let mut a = drill("Drill 0.8", 0.8);
    a.max_hours = 10.0;
    a.hours = 9.5;
    c.add(a).unwrap();
    let mut b = Tool::new("end mill 2", Kind::EndMill, 2.0);
    b.hours = 3.0;
    b.meters = 12.5;
    c.add(b).unwrap();
    let mut v = vbit();
    v.max_holes = 1000;
    v.holes = 1000;
    c.add(v).unwrap();
    let mut d = drill("alpha drill", 1.0);
    d.max_hours = 100.0;
    d.hours = 1.0;
    c.add(d).unwrap();
    c
}

fn shop_file(name: &str) -> (PathBuf, String) {
    let dir = scratch(name);
    let file = dir.join("crib.json");
    save(&shop(), &file).unwrap();
    let f = file.to_str().unwrap().to_string();
    (dir, f)
}

fn ids(list: &[&Tool]) -> Vec<u32> {
    list.iter().map(|t| t.id).collect()
}

#[test]
fn what_is_left_before_each_limit_is_told_and_never_goes_below_zero() {
    let c = shop();
    let r = c.get(1).unwrap().remaining();
    assert_eq!((r.hours, r.holes), (Some(0.5), None));
    let r = c.get(2).unwrap().remaining();
    assert_eq!((r.hours, r.holes), (None, None));
    let r = c.get(3).unwrap().remaining();
    assert_eq!((r.hours, r.holes), (None, Some(0)));
    let mut t = drill("over", 1.0);
    t.max_hours = 2.0;
    t.hours = 5.0;
    t.max_holes = 10;
    t.holes = u64::MAX;
    let r = t.remaining();
    assert_eq!((r.hours, r.holes), (Some(0.0), Some(0)));
}

#[test]
fn a_replaced_tool_keeps_everything_but_its_counters_and_says_what_they_were() {
    let mut c = shop();
    c.tools[0]
        .extra
        .insert("note".into(), serde_json::json!("from the drawer"));
    let before = c.get(1).unwrap().clone();
    let said = c.replace_tool(1).unwrap();
    assert_eq!(said, (9.5, 0.0, 0));
    let after = c.get(1).unwrap();
    assert_eq!((after.hours, after.meters, after.holes), (0.0, 0.0, 0));
    assert_eq!(after.wear(), Wear::Good(0.0));
    let kept = Tool {
        hours: 0.0,
        meters: 0.0,
        holes: 0,
        ..before
    };
    assert_eq!(*after, kept, "only the counters change");
    assert_eq!(c.tools.len(), 4);
    assert_eq!(c.next_id, 5);
}

#[test]
fn replacing_a_tool_that_is_not_there_changes_nothing() {
    let mut c = shop();
    let before = c.clone();
    assert!(c.replace_tool(99).unwrap_err().contains("no tool 99"));
    assert_eq!(c, before);
}

#[test]
fn a_list_can_show_only_a_kind_or_only_the_worn_tools_or_both() {
    let c = shop();
    let all = Selection::default();
    assert_eq!(ids(&c.select(&all)), vec![1, 2, 3, 4]);
    let drills = Selection {
        kind: Some(Kind::Drill),
        ..all
    };
    assert_eq!(ids(&c.select(&drills)), vec![1, 4]);
    let worn = Selection {
        worn_only: true,
        ..all
    };
    assert_eq!(ids(&c.select(&worn)), vec![1, 3]);
    let both = Selection {
        kind: Some(Kind::VBit),
        worn_only: true,
        ..all
    };
    assert_eq!(ids(&c.select(&both)), vec![3]);
    let none = Selection {
        kind: Some(Kind::Laser),
        ..all
    };
    assert!(c.select(&none).is_empty());
    assert!(Crib::default().select(&all).is_empty());
}

#[test]
fn a_list_can_be_sorted_by_number_by_name_without_regard_to_case_or_by_wear() {
    let c = shop();
    let by = |sort| {
        ids(&c.select(&Selection {
            sort,
            ..Selection::default()
        }))
    };
    assert_eq!(by(Sort::Id), vec![1, 2, 3, 4]);
    // "alpha drill", "Drill 0.8", "end mill 2", "V 30"
    assert_eq!(by(Sort::Name), vec![4, 1, 2, 3]);
    // to replace, then 95 %, then 1 %, then the tool with no limit
    assert_eq!(by(Sort::Wear), vec![3, 1, 4, 2]);
}

#[test]
fn the_way_to_sort_is_read_in_any_case_and_a_wrong_word_names_the_three() {
    assert_eq!(Sort::parse(" WEAR "), Ok(Sort::Wear));
    assert_eq!(Sort::parse("Name"), Ok(Sort::Name));
    assert_eq!(Sort::parse("id"), Ok(Sort::Id));
    assert_eq!(Sort::default(), Sort::Id);
    let e = Sort::parse("size").unwrap_err();
    assert!(e.contains("size") && e.contains("id, name or wear"), "{e}");
    assert!(Sort::parse("").is_err());
}

#[test]
fn the_sums_by_kind_follow_the_order_of_the_kinds_and_count_the_worn_tools() {
    let rows = shop().summary();
    let kinds: Vec<Kind> = rows.iter().map(|r| r.kind).collect();
    assert_eq!(kinds, vec![Kind::Drill, Kind::EndMill, Kind::VBit]);
    assert_eq!(
        (rows[0].tools, rows[0].hours, rows[0].to_replace),
        (2, 10.5, 1)
    );
    assert_eq!(
        (rows[1].tools, rows[1].meters, rows[1].to_replace),
        (1, 12.5, 0)
    );
    assert_eq!((rows[2].holes, rows[2].to_replace), (1000, 1));
    assert!(Crib::default().summary().is_empty());
    let mut c = Crib::default();
    for (i, name) in ["a", "b"].into_iter().enumerate() {
        let mut t = drill(name, 1.0);
        t.holes = u64::MAX - i as u64;
        c.add(t).unwrap();
    }
    assert_eq!(
        c.summary()[0].holes,
        u64::MAX,
        "the sum stops, it does not wrap"
    );
}

#[test]
fn seeded_cribs_keep_the_order_the_filters_and_the_sums_consistent() {
    for seed in 1..=25u64 {
        let mut rng = SimRng::new(seed);
        let mut c = Crib::default();
        for i in 0..12 {
            let kind = KINDS[rng.below(KINDS.len() as u64) as usize];
            let mut t = Tool::new(
                &format!("T{}{i}", ["a", "B", "c"][rng.below(3) as usize]),
                kind,
                1.0,
            );
            if matches!(kind, Kind::VBit | Kind::Engraver) {
                t.tip_angle_deg = Some(30.0);
            }
            t.hours = rng.below(200) as f64 / 10.0;
            t.meters = rng.below(50) as f64;
            t.holes = rng.below(3000);
            if rng.below(2) == 0 {
                t.max_hours = (10 + rng.below(20)) as f64;
            }
            if rng.below(2) == 0 {
                t.max_holes = 500 + rng.below(2000);
            }
            c.add(t).unwrap();
        }
        let sel = |sort, worn_only| Selection {
            kind: None,
            worn_only,
            sort,
        };
        let by_wear = c.select(&sel(Sort::Wear, false));
        assert_eq!(by_wear.len(), 12, "seed {seed}");
        for pair in by_wear.windows(2) {
            assert!(
                pair[0].wear().fraction() >= pair[1].wear().fraction(),
                "seed {seed}"
            );
        }
        let by_name = c.select(&sel(Sort::Name, false));
        for pair in by_name.windows(2) {
            assert!(pair[0].name.to_lowercase() <= pair[1].name.to_lowercase());
        }
        let worn: Vec<u32> = ids(&c.select(&sel(Sort::Wear, true)));
        let mut expected: Vec<u32> = c.to_replace().iter().map(|(t, _)| t.id).collect();
        let mut got = worn.clone();
        got.sort_unstable();
        expected.sort_unstable();
        assert_eq!(got, expected, "seed {seed}");
        let rows = c.summary();
        assert_eq!(rows.iter().map(|r| r.tools).sum::<usize>(), 12);
        assert_eq!(
            rows.iter().map(|r| r.holes).sum::<u64>(),
            c.tools.iter().map(|t| t.holes).sum::<u64>()
        );
        assert_eq!(
            rows.iter().map(|r| r.to_replace).sum::<usize>(),
            expected.len()
        );
    }
}

#[test]
fn the_list_command_filters_and_sorts_and_says_when_nothing_matches() {
    let (dir, f) = shop_file("list_filters");
    let o = cli(&["list", &f, "--worn", "--sort", "wear"]);
    assert!(o.status.success(), "{}", text(&o.stderr));
    let out = text(&o.stdout);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "{out}");
    assert!(lines[0].contains("V 30") && lines[0].contains("REPLACE"));
    assert!(lines[1].contains("Drill 0.8") && lines[1].contains("replace soon (95 %)"));
    let o = cli(&["list", &f, "--kind", "End-Mill"]);
    assert_eq!(text(&o.stdout).lines().count(), 1);
    let o = cli(&["list", &f, "--kind", "laser"]);
    assert!(o.status.success());
    assert_eq!(text(&o.stdout).trim(), "no tool matches");
    let o = cli(&["list", &f, "--sort", "name"]);
    let out = text(&o.stdout);
    let first = out.lines().next().unwrap();
    assert!(first.contains("alpha drill"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_list_command_refuses_a_kind_or_a_way_to_sort_that_does_not_exist() {
    let (dir, f) = shop_file("list_refuses");
    let o = cli(&["list", &f, "--kind", "saw"]);
    assert!(!o.status.success());
    assert!(text(&o.stderr).contains("not a kind of tool"));
    let o = cli(&["list", &f, "--sort", "size"]);
    assert!(!o.status.success());
    assert!(text(&o.stderr).contains("id, name or wear"));
    let o = cli(&["list", &f, "--sort"]);
    assert!(text(&o.stderr).contains("needs a value"));
    let o = cli(&["list", &f, "--oops"]);
    assert!(text(&o.stderr).contains("unknown option"));
    // an empty crib with no filter prints nothing and is not an error
    let empty = dir.join("empty.json");
    assert!(cli(&["init", empty.to_str().unwrap()]).status.success());
    let o = cli(&["list", empty.to_str().unwrap()]);
    assert!(o.status.success() && o.stdout.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_show_command_prints_every_value_and_what_is_left_before_the_limits() {
    let (dir, f) = shop_file("show");
    let o = cli(&["show", &f, "1"]);
    assert!(o.status.success(), "{}", text(&o.stderr));
    let out = text(&o.stdout);
    for part in [
        "tool 1: Drill 0.8",
        "kind        drill",
        "diameter    0.8 mm",
        "used        9.5 h, 0.0 m, 0 holes",
        "hour limit  10 h (0.5 h left)",
        "hole limit  no limit",
        "wear        replace soon (95 %)",
    ] {
        assert!(out.contains(part), "missing '{part}' in\n{out}");
    }
    assert!(!out.contains("tip angle"));
    let out = text(&cli(&["show", &f, "3"]).stdout);
    assert!(out.contains("tip angle   30 degrees"), "{out}");
    assert!(out.contains("hole limit  1000 (0 left)"), "{out}");
    assert!(out.contains("hour limit  no limit"), "{out}");
    assert!(out.contains("wear        REPLACE"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_show_command_refuses_a_tool_that_is_not_there_or_a_number_that_is_not_one() {
    let (dir, f) = shop_file("show_refuses");
    let o = cli(&["show", &f, "99"]);
    assert!(!o.status.success() && text(&o.stderr).contains("there is no tool 99"));
    let o = cli(&["show", &f, "one"]);
    assert!(text(&o.stderr).contains("'one' is not a tool number"));
    let o = cli(&["show", &f]);
    assert!(text(&o.stderr).contains("needs a file and a tool number"));
    let o = cli(&["show", &f, "1", "2"]);
    assert!(text(&o.stderr).contains("unexpected argument"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_replace_command_puts_the_counters_back_to_zero_and_keeps_the_previous_crib() {
    let (dir, f) = shop_file("replace_cmd");
    let before = std::fs::read_to_string(&f).unwrap();
    let o = cli(&["replace", &f, "3"]);
    assert!(o.status.success(), "{}", text(&o.stderr));
    assert_eq!(
        text(&o.stdout).trim(),
        "tool 3: counters back to zero (they said 0.0 h, 0.0 m, 1000 holes)"
    );
    let crib = load(std::path::Path::new(&f)).unwrap();
    let t = crib.get(3).unwrap();
    assert_eq!((t.hours, t.holes, t.max_holes), (0.0, 0, 1000));
    assert_eq!(
        std::fs::read_to_string(format!("{f}.bak")).unwrap(),
        before,
        "the crib that was there is kept"
    );
    // only the tool that was close to its limit is left on the list of worn tools
    let out = text(&cli(&["wear", &f]).stdout);
    assert!(out.contains("Drill 0.8") && !out.contains("V 30"), "{out}");
    // a tool that is not there changes nothing
    let kept = std::fs::read_to_string(&f).unwrap();
    let o = cli(&["replace", &f, "42"]);
    assert!(!o.status.success() && text(&o.stderr).contains("there is no tool 42"));
    assert_eq!(std::fs::read_to_string(&f).unwrap(), kept);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_remove_command_takes_the_tool_out_and_its_number_is_not_given_again() {
    let (dir, f) = shop_file("remove_cmd");
    let o = cli(&["remove", &f, "4"]);
    assert!(o.status.success(), "{}", text(&o.stderr));
    assert_eq!(
        text(&o.stdout).trim(),
        "tool 4 (alpha drill) removed; its number is not given again"
    );
    let crib = load(std::path::Path::new(&f)).unwrap();
    assert_eq!(crib.tools.len(), 3);
    assert!(crib.get(4).is_none());
    let o = cli(&["add", &f, "new drill", "drill", "1.2"]);
    assert_eq!(text(&o.stdout).trim(), "tool 5 added");
    // a number that is not there leaves the file as it was
    let kept = std::fs::read_to_string(&f).unwrap();
    let o = cli(&["remove", &f, "4"]);
    assert!(!o.status.success() && text(&o.stderr).contains("there is no tool 4"));
    assert_eq!(std::fs::read_to_string(&f).unwrap(), kept);
    let o = cli(&["remove", &f, "-1"]);
    assert!(text(&o.stderr).contains("is not a tool number"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_stats_command_adds_up_the_tools_by_kind_with_a_total() {
    let (dir, f) = shop_file("stats_cmd");
    let o = cli(&["stats", &f]);
    assert!(o.status.success(), "{}", text(&o.stderr));
    let out = text(&o.stdout);
    let row = |name: &str| -> Vec<String> {
        out.lines()
            .find(|l| l.starts_with(name))
            .unwrap_or_else(|| panic!("no row '{name}' in\n{out}"))
            .split_whitespace()
            .map(str::to_string)
            .collect()
    };
    assert_eq!(row("drill"), ["drill", "2", "10.5", "0.0", "0", "1"]);
    assert_eq!(row("end-mill"), ["end-mill", "1", "3.0", "12.5", "0", "0"]);
    assert_eq!(row("v-bit"), ["v-bit", "1", "0.0", "0.0", "1000", "1"]);
    assert_eq!(row("total"), ["total", "4", "13.5", "12.5", "1000", "2"]);
    assert!(!out.contains("laser"), "a kind with no tool has no row");
    let empty = dir.join("empty.json");
    assert!(cli(&["init", empty.to_str().unwrap()]).status.success());
    let o = cli(&["stats", empty.to_str().unwrap()]);
    assert_eq!(text(&o.stdout).trim(), "the crib has no tool");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_help_names_the_new_commands_and_their_options() {
    let out = text(&cli(&["help"]).stdout);
    for word in [
        "show", "replace", "remove", "stats", "--kind", "--worn", "--sort",
    ] {
        assert!(out.contains(word), "the help does not name {word}");
    }
}
