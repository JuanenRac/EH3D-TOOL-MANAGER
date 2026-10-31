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
