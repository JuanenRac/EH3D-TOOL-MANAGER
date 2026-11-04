//! More tests: the kinds, the wear thresholds, the use that is recorded, the order of the tools to replace, and the JSON and CSV files that are damaged or odd.

use eh3d_tool_manager::csv::{from_csv, parse_records, to_csv, HEADER};
use eh3d_tool_manager::model::{Crib, Kind, Tool, Wear, KINDS, MAX_DIAMETER_MM, MAX_NAME};
use eh3d_tool_manager::store::{from_json, to_json, MAX_TOOLS, SCHEMA_VERSION};

fn drill(name: &str) -> Tool {
    Tool::new(name, Kind::Drill, 0.8)
}

#[test]
fn every_kind_has_its_own_key_that_reads_back_in_any_case_and_with_blanks() {
    for k in KINDS {
        assert_eq!(Kind::parse(k.key()), Ok(k));
        assert_eq!(
            Kind::parse(&format!("  {}  ", k.key().to_ascii_uppercase())),
            Ok(k)
        );
    }
    let mut keys: Vec<&str> = KINDS.iter().map(|k| k.key()).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), KINDS.len());
    let e = Kind::parse("saw").unwrap_err();
    assert!(e.contains("saw") && e.contains("drill"), "{e}");
    assert!(Kind::parse("").is_err());
}

#[test]
fn the_wear_is_good_below_ninety_percent_soon_from_there_and_replace_at_the_limit() {
    let mut t = drill("a");
    t.max_hours = 10.0;
    t.hours = 8.9;
    assert!(matches!(t.wear(), Wear::Good(f) if (f - 0.89).abs() < 1e-9));
    t.hours = 9.0;
    assert!(matches!(t.wear(), Wear::Soon(_)));
    t.hours = 9.99;
    assert!(matches!(t.wear(), Wear::Soon(_)));
    t.hours = 10.0;
    assert_eq!(t.wear(), Wear::Replace);
    t.hours = 500.0;
    assert_eq!(t.wear(), Wear::Replace);
}

#[test]
fn a_tool_with_no_limits_never_wears_out_and_the_worst_of_hours_and_holes_decides() {
    let mut t = drill("a");
    t.hours = 1.0e6;
    t.holes = 1_000_000;
    assert_eq!(t.wear(), Wear::Good(0.0));
    t.max_hours = 100.0;
    t.max_holes = 1000;
    t.hours = 10.0;
    t.holes = 950;
    assert!(matches!(t.wear(), Wear::Soon(f) if (f - 0.95).abs() < 1e-9));
    t.holes = 1000;
    assert_eq!(t.wear(), Wear::Replace);
}

#[test]
fn a_tool_that_is_not_valid_says_which_field_and_the_limits_are_inclusive() {
    let mut t = Tool::new("ok", Kind::EndMill, MAX_DIAMETER_MM);
    assert!(t.validate().is_ok());
    t.diameter_mm = MAX_DIAMETER_MM + 0.001;
    assert!(t.validate().unwrap_err().contains("diameter_mm"));
    t.diameter_mm = f64::NAN;
    assert!(t.validate().is_err());
    t.diameter_mm = 1.0;
    for flutes in [0, 9] {
        t.flutes = flutes;
        assert!(t.validate().unwrap_err().contains("flutes"));
    }
    t.flutes = 8;
    assert!(t.validate().is_ok());
    t.shank_mm = 25.0;
    assert!(t.validate().is_ok());
    t.shank_mm = 25.5;
    assert!(t.validate().unwrap_err().contains("shank_mm"));
    t.shank_mm = 3.0;
    t.length_mm = -1.0;
    assert!(t.validate().unwrap_err().contains("length_mm"));
    t.length_mm = 0.0;
    t.max_hours = f64::INFINITY;
    assert!(t.validate().unwrap_err().contains("max_hours"));
}

#[test]
fn the_name_has_one_to_eighty_characters_counted_as_characters_and_no_control_character() {
    let mut t = drill(&"x".repeat(MAX_NAME));
    assert!(t.validate().is_ok());
    t.name = "x".repeat(MAX_NAME + 1);
    assert!(t.validate().is_err());
    // eighty letters that take more than one byte are still eighty
    t.name = "ñ".repeat(MAX_NAME);
    assert!(t.validate().is_ok());
    t.name = "a\nb".to_string();
    assert!(t.validate().is_err());
    t.name = String::new();
    assert!(t.validate().is_err());
}

#[test]
fn a_v_bit_and_an_engraver_need_their_angle_and_a_drill_does_not() {
    for kind in [Kind::VBit, Kind::Engraver] {
        let mut t = Tool::new("tip", kind, 3.0);
        assert!(t.validate().unwrap_err().contains("tip_angle_deg"));
        t.tip_angle_deg = Some(60.0);
        assert!(t.validate().is_ok());
        t.tip_angle_deg = Some(181.0);
        assert!(t.validate().is_err());
        t.tip_angle_deg = Some(0.0);
        assert!(t.validate().is_err());
    }
    assert!(drill("d").validate().is_ok());
}

#[test]
fn recording_a_use_adds_to_the_counters_and_says_the_wear_and_refuses_what_makes_no_sense() {
    let mut c = Crib::default();
    let mut t = drill("a");
    t.max_holes = 100;
    let id = c.add(t).unwrap();
    assert!(matches!(
        c.record_use(id, 0.5, 2.0, 50).unwrap(),
        Wear::Good(_)
    ));
    assert!(matches!(
        c.record_use(id, 0.0, 0.0, 45).unwrap(),
        Wear::Soon(_)
    ));
    assert_eq!(c.record_use(id, 0.0, 0.0, 5).unwrap(), Wear::Replace);
    let t = c.get(id).unwrap();
    assert_eq!((t.hours, t.meters, t.holes), (0.5, 2.0, 100));
    for (h, m) in [
        (-1.0, 0.0),
        (0.0, -1.0),
        (f64::NAN, 0.0),
        (0.0, f64::INFINITY),
    ] {
        assert!(c.record_use(id, h, m, 0).is_err(), "{h} {m}");
    }
    assert!(c.record_use(99, 1.0, 1.0, 1).unwrap_err().contains("99"));
    // nothing changed by the refused ones
    assert_eq!(c.get(id).unwrap().holes, 100);
}

#[test]
fn the_holes_counter_does_not_wrap_around_and_the_hours_cannot_grow_past_a_number() {
    let mut c = Crib::default();
    let id = c.add(drill("a")).unwrap();
    c.record_use(id, 0.0, 0.0, u64::MAX).unwrap();
    c.record_use(id, 0.0, 0.0, 10).unwrap();
    assert_eq!(c.get(id).unwrap().holes, u64::MAX);
    c.record_use(id, f64::MAX, 0.0, 0).unwrap();
    assert!(c
        .record_use(id, f64::MAX, 0.0, 0)
        .unwrap_err()
        .contains("past"));
}

#[test]
fn the_tools_to_replace_come_worst_first_and_the_good_ones_are_left_out() {
    let mut c = Crib::default();
    for (name, hours) in [
        ("good", 1.0),
        ("soon", 9.2),
        ("worn", 12.0),
        ("sooner", 9.8),
    ] {
        let mut t = drill(name);
        t.max_hours = 10.0;
        t.hours = hours;
        c.add(t).unwrap();
    }
    let names: Vec<&str> = c
        .to_replace()
        .iter()
        .map(|(t, _)| t.name.as_str())
        .collect();
    assert_eq!(names, ["worn", "sooner", "soon"]);
    assert!(Crib::default().to_replace().is_empty());
}

#[test]
fn a_number_that_was_removed_is_not_given_to_a_tool_that_comes_after_a_higher_one() {
    let mut c = Crib::default();
    for n in ["a", "b", "c"] {
        c.add(drill(n)).unwrap();
    }
    c.remove(3).unwrap();
    assert_eq!(c.add(drill("d")).unwrap(), 3);
    c.remove(1).unwrap();
    assert_eq!(c.add(drill("e")).unwrap(), 4);
}

#[test]
fn the_json_of_a_crib_reads_back_the_same_and_ends_with_a_new_line() {
    let mut c = Crib::default();
    let mut v = Tool::new("V 30, \"fine\"", Kind::VBit, 3.175);
    v.tip_angle_deg = Some(30.0);
    v.hours = 1.25;
    v.holes = 7;
    c.add(v).unwrap();
    c.add(drill("d 0.8")).unwrap();
    let json = to_json(&c);
    assert!(json.ends_with('\n'));
    assert!(json.contains(&format!("\"schema_version\": {SCHEMA_VERSION}")));
    assert_eq!(from_json(&json).unwrap(), c);
}

#[test]
fn a_json_that_is_damaged_names_what_is_wrong() {
    assert!(from_json("").unwrap_err().contains("not JSON"));
    assert!(from_json("{").unwrap_err().contains("not JSON"));
    assert!(from_json("[]").unwrap_err().contains("no list"));
    assert!(from_json("{\"tools\": 3}").unwrap_err().contains("no list"));
    let newer = format!(
        "{{\"schema_version\": {}, \"tools\": []}}",
        SCHEMA_VERSION + 1
    );
    assert!(from_json(&newer).unwrap_err().contains("format"));
    assert_eq!(from_json("{\"tools\": []}").unwrap(), Crib::default());
    let no_name = "{\"tools\": [{\"id\": 1, \"kind\": \"drill\", \"diameter_mm\": 1}]}";
    assert!(from_json(no_name).unwrap_err().contains("no name"));
    let bad_kind =
        "{\"tools\": [{\"id\": 1, \"name\": \"a\", \"kind\": \"saw\", \"diameter_mm\": 1}]}";
    assert!(from_json(bad_kind).unwrap_err().contains("tool 1"));
    let text_number =
        "{\"tools\": [{\"id\": 1, \"name\": \"a\", \"kind\": \"drill\", \"diameter_mm\": \"1\"}]}";
    assert!(from_json(text_number).unwrap_err().contains("diameter_mm"));
    let negative_holes = "{\"tools\": [{\"id\": 1, \"name\": \"a\", \"kind\": \"drill\", \"diameter_mm\": 1, \"holes\": -3}]}";
    assert!(from_json(negative_holes).unwrap_err().contains("holes"));
    let fractional = "{\"tools\": [{\"id\": 1, \"name\": \"a\", \"kind\": \"drill\", \"diameter_mm\": 1, \"holes\": 2.5}]}";
    assert!(from_json(fractional).unwrap_err().contains("holes"));
}

#[test]
fn the_numbers_of_the_tools_must_be_there_and_not_repeated_and_neither_may_the_names() {
    let item = |id: u32, name: &str| {
        format!("{{\"id\": {id}, \"name\": \"{name}\", \"kind\": \"drill\", \"diameter_mm\": 1}}")
    };
    let two = |a: String, b: String| format!("{{\"tools\": [{a}, {b}]}}");
    assert!(from_json(&two(item(1, "a"), item(1, "b")))
        .unwrap_err()
        .contains("twice"));
    assert!(from_json(&two(item(1, "a"), item(2, "A")))
        .unwrap_err()
        .contains("twice"));
    assert!(from_json(&two(item(0, "a"), item(2, "b")))
        .unwrap_err()
        .contains("missing"));
    assert_eq!(
        from_json(&two(item(5, "a"), item(2, "b")))
            .unwrap()
            .tools
            .len(),
        2
    );
}

#[test]
fn a_crib_with_more_tools_than_the_limit_is_refused_before_it_is_read() {
    let one = "{\"id\": 1, \"name\": \"a\", \"kind\": \"drill\", \"diameter_mm\": 1}";
    let many = vec![one; MAX_TOOLS + 1].join(",");
    let e = from_json(&format!("{{\"tools\": [{many}]}}")).unwrap_err();
    assert!(e.contains("more than"), "{e}");
}

#[test]
fn the_csv_quotes_a_name_with_a_comma_a_quote_or_a_line_break_and_reads_it_back() {
    let mut c = Crib::default();
    for name in ["plain", "with, comma", "with \"quote\""] {
        c.add(drill(name)).unwrap();
    }
    let text = to_csv(&c);
    assert!(text.starts_with(HEADER));
    assert!(text.contains("\"with, comma\""));
    assert!(text.contains("\"with \"\"quote\"\"\""));
    assert_eq!(from_csv(&text).unwrap(), c);
}

#[test]
fn a_csv_with_crlf_lines_a_byte_order_mark_and_blank_lines_is_read() {
    let mut c = Crib::default();
    c.add(drill("a")).unwrap();
    c.add(drill("b")).unwrap();
    let text = format!("\u{feff}{}", to_csv(&c).replace('\n', "\r\n\r\n"));
    assert_eq!(from_csv(&text).unwrap(), c);
}

#[test]
fn a_csv_that_is_damaged_names_its_row() {
    assert!(from_csv("").unwrap_err().contains("empty"));
    assert!(from_csv("id,name\n1,a\n").unwrap_err().contains("exactly"));
    let row = |r: &str| format!("{HEADER}\n{r}\n");
    assert!(from_csv(&row("1,a,drill,1,2,0,,3.175,carbide,0,0,0,0"))
        .unwrap_err()
        .contains("row 2"));
    assert!(from_csv(&row("1,a,drill,x,2,0,,3.175,carbide,0,0,0,0,0"))
        .unwrap_err()
        .contains("diameter_mm"));
    assert!(from_csv(&row("1,a,saw,1,2,0,,3.175,carbide,0,0,0,0,0")).is_err());
    assert!(from_csv(&row("x,a,drill,1,2,0,,3.175,carbide,0,0,0,0,0"))
        .unwrap_err()
        .contains("id"));
    assert!(from_csv(&row("0,a,drill,1,2,0,,3.175,carbide,0,0,0,0,0"))
        .unwrap_err()
        .contains("missing"));
    assert!(from_csv(&row("1,a,drill,1,2,0,,3.175,carbide,0,0,-1,0,0"))
        .unwrap_err()
        .contains("holes"));
    assert!(from_csv(&format!("{HEADER}\n1,\"a,drill\n"))
        .unwrap_err()
        .contains("quote"));
}

#[test]
fn the_records_of_a_csv_keep_a_quoted_comma_a_doubled_quote_and_a_line_break_inside_quotes() {
    let r = parse_records("a,\"b,c\",\"d \"\"e\"\"\",\"f\ng\"\n").unwrap();
    assert_eq!(r, vec![vec!["a", "b,c", "d \"e\"", "f\ng"]]);
    // a last line without a line break, and empty lines in between
    let r = parse_records("a,b\n\n\nc,d").unwrap();
    assert_eq!(r.len(), 2);
    assert_eq!(parse_records("").unwrap(), Vec::<Vec<String>>::new());
}

#[test]
fn a_number_that_was_given_is_not_given_again_after_its_tool_was_removed() {
    let mut c = Crib::default();
    assert_eq!(c.add(drill("a")).unwrap(), 1);
    assert_eq!(c.add(drill("b")).unwrap(), 2);
    c.remove(2).unwrap();
    assert_eq!(
        c.add(drill("c")).unwrap(),
        3,
        "the number 2 must not come back"
    );
    // it survives the file: a crib written and read again goes on from there
    c.remove(3).unwrap();
    let mut back = from_json(&to_json(&c)).unwrap();
    assert_eq!(back.add(drill("d")).unwrap(), 4);
    // a crib without the field (an older file) still goes above every number in use
    let old = from_json(
        r#"{"schema_version":1,"tools":[{"id":7,"name":"x","kind":"drill","diameter_mm":1}]}"#,
    )
    .unwrap();
    assert_eq!(old.next_id, 8);
}

#[test]
fn an_id_that_is_not_a_whole_positive_number_is_told_as_such() {
    for id in ["1.5", "-1", "\"2\"", "4294967296"] {
        let json =
            format!(r#"{{"tools":[{{"id":{id},"name":"a","kind":"drill","diameter_mm":1}}]}}"#);
        let e = from_json(&json).unwrap_err();
        assert!(e.contains("whole number") && e.contains("id"), "{id}: {e}");
    }
}

#[test]
fn the_material_has_a_limit_and_no_control_characters() {
    let mut t = drill("a");
    t.material = "x".repeat(MAX_NAME + 1);
    assert!(t.validate().is_err());
    t.material = "carbide\nsteel".into();
    assert!(t.validate().is_err());
    t.material = "carbide".into();
    assert!(t.validate().is_ok());
}

#[test]
fn text_after_the_closing_quote_of_a_field_is_an_error_and_a_big_flute_count_has_its_own_message() {
    let e = parse_records("a,\"b\"c,d\n").unwrap_err();
    assert!(e.contains("closing quote"), "{e}");
    assert!(parse_records("a,\"b\",d\n\"e\"\n").is_ok());
    let row =
        |flutes: &str| format!("{HEADER}\n1,a,drill,0.8,{flutes},0,,3.175,carbide,0,0,0,0,0\n");
    let e = from_csv(&row("99999999999")).unwrap_err();
    assert!(
        e.contains("row 2") && e.contains("flutes") && e.contains("too large"),
        "{e}"
    );
    let e = from_csv(&row("12")).unwrap_err();
    assert!(e.contains("flutes") && e.contains("1 to 8"), "{e}");
}

#[test]
fn a_text_that_a_spreadsheet_would_run_as_a_formula_is_written_as_text_and_read_back_whole() {
    let mut c = Crib::default();
    let mut t = drill("=HYPERLINK(\"http://x\")");
    t.material = "@sum".into();
    c.add(t).unwrap();
    c.add(drill("+1")).unwrap();
    c.add(drill("plain")).unwrap();
    let text = to_csv(&c);
    for line in text.lines().skip(1) {
        assert!(
            !line
                .split(',')
                .nth(1)
                .unwrap_or("")
                .starts_with(['=', '+', '-', '@']),
            "{line}"
        );
    }
    assert_eq!(from_csv(&text).unwrap(), c);
}
