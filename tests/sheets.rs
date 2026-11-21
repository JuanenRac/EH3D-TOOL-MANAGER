//! The sheets of stencil material: the model, the JSON document (a crib with no sheet is written as before), the table of sheets (the table of tools does not change) and the limits of
//! the reads.

use eh3d_tool_manager::csv::{
    from_csv, from_csv_limited, from_sheets_csv, from_sheets_csv_limited, parse_records, to_csv,
    to_sheets_csv, HEADER, SHEET_HEADER,
};
use eh3d_tool_manager::model::{Crib, Kind, Tool};
use eh3d_tool_manager::sheet::{Sheet, MAX_KERF_MM, MAX_THICKNESS_MM};
use eh3d_tool_manager::store::{from_json, to_json, SHEETS_SCHEMA_VERSION};

fn sheet(name: &str, thickness: f64) -> Sheet {
    Sheet::new(name, "stainless", thickness)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn a_sheet_is_numbered_and_refuses_a_repeated_name_and_what_makes_no_sense() {
    let mut c = Crib::default();
    assert_eq!(c.add_sheet(sheet("foil 0.12", 0.12)).unwrap(), 1);
    assert_eq!(c.add_sheet(sheet("foil 0.15", 0.15)).unwrap(), 2);
    let e = c.add_sheet(sheet("FOIL 0.12", 0.12)).unwrap_err();
    assert!(e.contains("already"), "{e}");
    let mut bad = sheet("a", 0.1);
    for (set, field) in [
        (
            Box::new(|s: &mut Sheet| s.thickness_mm = 0.0) as Box<dyn Fn(&mut Sheet)>,
            "thickness_mm",
        ),
        (
            Box::new(|s: &mut Sheet| s.thickness_mm = MAX_THICKNESS_MM + 1.0),
            "thickness_mm",
        ),
        (
            Box::new(|s: &mut Sheet| s.thickness_mm = f64::NAN),
            "thickness_mm",
        ),
        (
            Box::new(|s: &mut Sheet| s.reduction_mm = -0.01),
            "reduction_mm",
        ),
        (
            Box::new(|s: &mut Sheet| s.kerf_mm = MAX_KERF_MM + 0.1),
            "kerf_mm",
        ),
        (
            Box::new(|s: &mut Sheet| s.min_aspect_ratio = -1.0),
            "min_aspect_ratio",
        ),
        (
            Box::new(|s: &mut Sheet| s.min_area_ratio = f64::INFINITY),
            "min_area_ratio",
        ),
    ] {
        set(&mut bad);
        let e = c.add_sheet(bad.clone()).unwrap_err();
        assert!(e.contains(field), "{field}: {e}");
        bad = sheet("a", 0.1);
    }
    assert!(c.add_sheet(sheet("", 0.1)).is_err());
    assert!(c.add_sheet(sheet(&"x".repeat(81), 0.1)).is_err());
    assert_eq!(c.sheets.len(), 2);
}

#[test]
fn the_number_of_a_removed_sheet_is_not_given_again_and_a_sheet_has_a_number_of_its_own() {
    let mut c = Crib::default();
    c.add(Tool::new("drill", Kind::Drill, 0.8)).unwrap();
    assert_eq!(c.add_sheet(sheet("a", 0.1)).unwrap(), 1);
    assert_eq!(c.add_sheet(sheet("b", 0.1)).unwrap(), 2);
    assert_eq!(c.remove_sheet(2).unwrap().name, "b");
    assert!(c.remove_sheet(2).unwrap_err().contains("no sheet 2"));
    assert_eq!(c.add_sheet(sheet("c", 0.1)).unwrap(), 3);
    // the numbers of the tools were not touched
    assert_eq!(c.add(Tool::new("mill", Kind::EndMill, 1.0)).unwrap(), 2);
    // and the next number survives a trip through the file even when no sheet is left
    c.remove_sheet(1).unwrap();
    c.remove_sheet(3).unwrap();
    let back = from_json(&to_json(&c)).unwrap();
    assert!(back.sheets.is_empty() && back.next_sheet_id == 4);
}

#[test]
fn a_sheet_round_trips_in_json_and_a_crib_with_no_sheet_is_written_as_before() {
    let mut plain = Crib::default();
    plain.add(Tool::new("drill", Kind::Drill, 0.8)).unwrap();
    let before = to_json(&plain);
    assert!(before.contains("\"schema_version\": 1"));
    assert!(!before.contains("sheets") && !before.contains("next_sheet_id"));
    assert_eq!(from_json(&before).unwrap(), plain);

    let mut c = plain.clone();
    let mut s = sheet("foil", 0.12);
    s.reduction_mm = 0.03;
    s.kerf_mm = 0.05;
    s.min_aspect_ratio = 1.5;
    s.min_area_ratio = 0.66;
    c.add_sheet(s).unwrap();
    let json = to_json(&c);
    assert!(json.contains(&format!("\"schema_version\": {SHEETS_SCHEMA_VERSION}")));
    assert!(json.contains("\"kind\": \"sheet\""));
    assert_eq!(from_json(&json).unwrap(), c);
    // the tools are written as they were
    assert_eq!(from_json(&json).unwrap().tools, plain.tools);
}

#[test]
fn the_keys_of_a_sheet_that_the_program_does_not_know_are_kept_and_a_damaged_sheet_is_named() {
    let text = r#"{"schema_version":2,"tools":[],"sheets":[{"id":1,"name":"foil","kind":"sheet","material":"steel","thickness_mm":0.1,"colour":"blue"}]}"#;
    let c = from_json(text).unwrap();
    assert_eq!(c.sheets[0].extra.get("colour").unwrap(), "blue");
    assert!(to_json(&c).contains("\"colour\": \"blue\""));
    for (doc, wanted) in [
        (r#"{"tools":[],"sheets":{}}"#, "not a list"),
        (
            r#"{"tools":[],"sheets":[{"id":1,"thickness_mm":0.1}]}"#,
            "sheet 1: it has no name",
        ),
        (
            r#"{"tools":[],"sheets":[{"id":1,"name":"a","thickness_mm":0.1},{"id":2,"name":"b","thickness_mm":-1}]}"#,
            "sheet 2",
        ),
        (
            r#"{"tools":[],"sheets":[{"id":1,"name":"a","kind":"drill","thickness_mm":0.1}]}"#,
            "not \"sheet\"",
        ),
        (
            r#"{"tools":[],"sheets":[{"id":1,"name":"a","thickness_mm":0.1},{"id":1,"name":"b","thickness_mm":0.1}]}"#,
            "used twice",
        ),
        (
            r#"{"tools":[],"sheets":[{"id":1,"name":"a","thickness_mm":0.1},{"id":2,"name":"A","thickness_mm":0.1}]}"#,
            "used twice",
        ),
        (
            r#"{"tools":[],"sheets":[{"name":"a","thickness_mm":0.1}]}"#,
            "missing",
        ),
        (r#"{"schema_version":3,"tools":[]}"#, "format 3"),
    ] {
        let e = from_json(doc).unwrap_err();
        assert!(e.contains(wanted), "{doc}: {e}");
    }
}

#[test]
fn the_table_of_tools_does_not_change_and_the_sheets_have_a_table_of_their_own() {
    assert_eq!(
        HEADER,
        "id,name,kind,diameter_mm,flutes,length_mm,tip_angle_deg,shank_mm,material,hours,meters,holes,max_hours,max_holes"
    );
    let mut plain = Crib::default();
    plain.add(Tool::new("drill", Kind::Drill, 0.8)).unwrap();
    let mut c = plain.clone();
    c.add_sheet(sheet("foil, \"thin\"", 0.12)).unwrap();
    c.add_sheet(Sheet::new("=1+1", "+steel", 0.2)).unwrap();
    assert_eq!(to_csv(&c), to_csv(&plain));

    let table = to_sheets_csv(&c);
    assert!(table.starts_with(SHEET_HEADER));
    assert!(table.lines().skip(1).all(|l| l.contains(",sheet,")));
    let back = from_sheets_csv(&table).unwrap();
    assert_eq!(back.sheets, c.sheets);
    assert_eq!(back.next_sheet_id, 3);
    // a header only is a crib with no sheet
    assert!(from_sheets_csv(SHEET_HEADER).unwrap().sheets.is_empty());
    // each reader says when it was given the other table
    assert!(from_csv(&table).unwrap_err().contains("table of sheets"));
    assert!(from_sheets_csv(&to_csv(&plain))
        .unwrap_err()
        .contains("table of tools"));
}

#[test]
fn a_damaged_table_of_sheets_names_the_row() {
    let row = |tail: &str| format!("{SHEET_HEADER}\n1,a,sheet,steel,0.1,0,0,0,0\n{tail}\n");
    for (tail, wanted) in [
        ("2,b,drill,steel,0.1,0,0,0,0", "row 3: the kind 'drill'"),
        ("2,b,sheet,steel,0,0,0,0,0", "row 3: thickness_mm"),
        ("2,b,sheet,steel,wide,0,0,0,0", "row 3: thickness_mm 'wide'"),
        ("2,b,sheet,steel,0.1,0,2,0,0", "row 3: kerf_mm"),
        ("1,b,sheet,steel,0.1,0,0,0,0", "row 3: the id 1"),
        ("2,A,sheet,steel,0.1,0,0,0,0", "row 3: the name 'A'"),
        ("x,b,sheet,steel,0.1,0,0,0,0", "row 3: the id 'x'"),
        ("2,b,sheet,steel,0.1", "row 3: it has 5 fields"),
    ] {
        let e = from_sheets_csv(&row(tail)).unwrap_err();
        assert!(e.contains(wanted), "{tail}: {e}");
    }
    assert!(from_sheets_csv("").unwrap_err().contains("empty"));
    assert!(from_sheets_csv("id,name\n")
        .unwrap_err()
        .contains("exactly"));
}

#[test]
fn an_aperture_is_checked_against_the_ratios_and_the_reduction() {
    let mut s = sheet("foil", 0.12);
    // no limit is set: any aperture passes
    assert!(s.check_aperture(0.05, 0.05).is_ok());
    let (aspect, area) = s.ratios(0.5, 0.5).unwrap();
    assert!(close(aspect, 0.5 / 0.12) && close(area, 0.25 / (2.0 * 1.0 * 0.12)));
    s.min_aspect_ratio = 1.5;
    assert!(s.check_aperture(0.5, 0.5).is_ok());
    let e = s.check_aperture(0.15, 0.15).unwrap_err();
    assert!(e.contains("aspect ratio") && e.contains("foil"), "{e}");
    s.min_area_ratio = 0.7;
    // a long narrow slot: 0.2 x 1.0 has an aspect of 1.67 but an area ratio of 0.69
    let e = s.check_aperture(0.2, 1.0).unwrap_err();
    assert!(e.contains("area ratio"), "{e}");
    s.min_area_ratio = 0.69;
    assert!(s.check_aperture(0.2, 1.0).is_ok());
    for bad in [
        (0.0, 1.0),
        (-1.0, 1.0),
        (f64::NAN, 1.0),
        (1.0, f64::INFINITY),
    ] {
        assert!(s.check_aperture(bad.0, bad.1).is_err(), "{bad:?}");
    }
    // the reduction is on each side
    s.reduction_mm = 0.03;
    let (w, l) = s.reduced(0.5, 0.8).unwrap();
    assert!(close(w, 0.44) && close(l, 0.74));
    s.reduction_mm = 0.3;
    assert!(s.reduced(0.5, 0.5).unwrap_err().contains("no size"));
}

#[test]
fn a_table_with_more_rows_than_the_limit_is_refused_and_a_broken_quote_names_its_row() {
    let mut text = format!("{HEADER}\n");
    for i in 1..=3 {
        text.push_str(&format!("{i},t{i},drill,1,2,0,,3.175,carbide,0,0,0,0,0\n"));
    }
    assert_eq!(from_csv_limited(&text, 3).unwrap().tools.len(), 3);
    let e = from_csv_limited(&text, 2).unwrap_err();
    assert!(
        e.contains("3 rows") && e.contains("2 allowed") && e.contains("max_csv_rows"),
        "{e}"
    );
    let mut sheets = format!("{SHEET_HEADER}\n");
    for i in 1..=3 {
        sheets.push_str(&format!("{i},s{i},sheet,steel,0.1,0,0,0,0\n"));
    }
    assert!(from_sheets_csv_limited(&sheets, 3).is_ok());
    assert!(from_sheets_csv_limited(&sheets, 2)
        .unwrap_err()
        .contains("3 rows"));
    // the header is row 1
    assert!(parse_records("a,b\nc,\"d").unwrap_err().contains("row 2"));
    assert!(parse_records("a,b\n\"c\"x,d")
        .unwrap_err()
        .contains("row 2"));
    assert!(parse_records("\"a\"x").unwrap_err().contains("row 1"));
    assert!(parse_records("a\nb\nc,\"d\n\n")
        .unwrap_err()
        .contains("row 3"));
}
