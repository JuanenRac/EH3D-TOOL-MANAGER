//! Behaviour of the library at the exact edge of each rule: the cut and its limits, the wear, the numbers of the crib, the check and its report, the sheets, the tables and the JSON
//! document (also with seeded random cribs), the files, the settings and the words of the eight languages. Nothing here uses a clock or the network.

use eh3d_tool_manager::calc::{
    feeds_speeds, feeds_speeds_traced, feeds_speeds_traced_at, v_bit_width_mm, CalcParams,
    Material, MATERIALS,
};
use eh3d_tool_manager::check::{check, parse_needs, Need, NeedKind, NEEDS_HEADER};
use eh3d_tool_manager::csv;
use eh3d_tool_manager::files::{read_text_limited, write_atomic, NOT_TEXT, TOO_LARGE, UNREADABLE};
use eh3d_tool_manager::lang::{fill, Lang, Msg};
use eh3d_tool_manager::model::{Crib, Kind, Selection, Sort, Tool, Wear, KINDS};
use eh3d_tool_manager::settings::Settings;
use eh3d_tool_manager::sheet::Sheet;
use eh3d_tool_manager::store::{self, SCHEMA_VERSION, SHEETS_SCHEMA_VERSION};
use std::fs;

/// A small generator with a fixed seed (31 bits of state).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = (self.0 * 1_103_515_245 + 12_345) & 0x7FFF_FFFF;
        self.0 >> 8
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "eh3d_tools_behaviour_{}_{name}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn tool(name: &str, kind: Kind, diameter: f64) -> Tool {
    let mut t = Tool::new(name, kind, diameter);
    if matches!(kind, Kind::VBit | Kind::Engraver) {
        t.tip_angle_deg = Some(60.0);
    }
    t
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * (1.0 + a.abs().max(b.abs()))
}

fn random_tool(rng: &mut Lcg, i: usize) -> Tool {
    let prefixes = ["", "=", "+", "-", "@", "'", "\"", ",", "日本", "a b", "Ünï"];
    let suffixes = ["", ",x", "\"q\"", "é"];
    let name = format!(
        "{}t{i}{}",
        prefixes[rng.below(prefixes.len())],
        suffixes[rng.below(suffixes.len())]
    );
    let diameters = [0.1, 0.8, 3.175, 6.0, 12.7];
    let materials = ["carbide", "hss", "a,b", "=x"];
    let mut t = tool(
        &name,
        KINDS[rng.below(KINDS.len())],
        diameters[rng.below(5)],
    );
    t.flutes = 1 + rng.below(8) as u32;
    t.length_mm = rng.below(100) as f64 / 4.0;
    t.material = materials[rng.below(materials.len())].to_string();
    t.hours = rng.below(2000) as f64 / 8.0;
    t.meters = rng.below(5000) as f64 / 4.0;
    t.holes = rng.below(100_000) as u64;
    t.max_hours = rng.below(100) as f64;
    t.max_holes = rng.below(5000) as u64;
    t
}

fn random_crib(seed: u64, count: usize) -> Crib {
    let mut rng = Lcg(seed);
    let mut crib = Crib::default();
    for i in 0..count {
        crib.add(random_tool(&mut rng, i)).unwrap();
    }
    // some numbers are left behind, as in a crib that has been used
    for _ in 0..count / 4 {
        let at = rng.below(crib.tools.len());
        let id = crib.tools[at].id;
        crib.remove(id).unwrap();
    }
    for i in 0..count / 3 {
        let mut s = Sheet::new(&format!("s{i},\"x\""), "steel", (1 + i % 5) as f64 / 8.0);
        s.kerf_mm = (i % 4) as f64 / 8.0;
        s.reduction_mm = (i % 3) as f64 / 10.0;
        crib.add_sheet(s).unwrap();
    }
    crib
}

// ---------------------------------------------------------------- the cut

#[test]
fn a_one_millimetre_drill_in_fr4_is_limited_by_the_spindle_with_exact_numbers() {
    let cut = feeds_speeds(&tool("d1", Kind::Drill, 1.0), Material::Fr4, 24000.0).unwrap();
    assert!(cut.rpm_limited);
    assert_eq!(cut.rpm, 24000.0);
    assert_eq!(cut.feed_mm_min, 720.0);
    assert_eq!(cut.plunge_mm_min, 240.0);
    assert_eq!(cut.pass_depth_mm, 0.0);
}

#[test]
fn a_big_drill_is_not_limited_and_turns_at_the_speed_of_the_material() {
    let cut = feeds_speeds(&tool("d3", Kind::Drill, 3.0), Material::Fr4, 24000.0).unwrap();
    assert!(!cut.rpm_limited);
    assert_eq!(
        cut.rpm,
        (100.0 * 1000.0 / (std::f64::consts::PI * 3.0)).round()
    );
}

#[test]
fn the_spindle_must_turn_at_least_a_thousand_rpm_and_be_a_real_number() {
    let t = tool("d", Kind::Drill, 1.0);
    for bad in [
        999.99,
        0.0,
        -5.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        let why = feeds_speeds(&t, Material::Wood, bad).unwrap_err();
        assert!(why.contains("max_rpm"), "{bad}: {why}");
    }
    let cut = feeds_speeds(&t, Material::Wood, 1000.0).unwrap();
    assert_eq!(cut.rpm, 1000.0);
    assert!(cut.rpm_limited);
}

#[test]
fn a_laser_and_a_nozzle_have_no_cutting_numbers() {
    for kind in [Kind::Laser, Kind::Nozzle] {
        let why = feeds_speeds(&tool("x", kind, 0.4), Material::Acrylic, 24000.0).unwrap_err();
        assert!(why.contains("no feeds and speeds"), "{why}");
        assert!(why.contains(kind.key()), "{why}");
    }
}

#[test]
fn a_depth_of_cut_is_only_for_a_v_bit_or_an_engraver() {
    let params = CalcParams::default();
    for kind in [Kind::Drill, Kind::EndMill] {
        let t = tool("x", kind, 2.0);
        let why =
            feeds_speeds_traced_at(&t, Material::Fr4, 24000.0, &params, Some(0.1)).unwrap_err();
        assert!(why.contains("cut depth"), "{why}");
    }
    for kind in [Kind::VBit, Kind::Engraver] {
        let t = tool("x", kind, 3.175);
        assert!(feeds_speeds_traced_at(&t, Material::Fr4, 24000.0, &params, Some(0.1)).is_ok());
    }
}

#[test]
fn the_width_of_a_v_bit_cut_accepts_each_limit_and_refuses_what_is_just_outside() {
    assert!(v_bit_width_mm(0.0, 10.0, 0.0).is_ok());
    assert!(v_bit_width_mm(100.0, 170.0, 5.0).is_ok());
    assert!(v_bit_width_mm(100.001, 60.0, 0.1)
        .unwrap_err()
        .contains("depth_mm"));
    assert!(v_bit_width_mm(-0.001, 60.0, 0.1)
        .unwrap_err()
        .contains("depth_mm"));
    assert!(v_bit_width_mm(1.0, 9.99, 0.1)
        .unwrap_err()
        .contains("angle_deg"));
    assert!(v_bit_width_mm(1.0, 170.01, 0.1)
        .unwrap_err()
        .contains("angle_deg"));
    assert!(v_bit_width_mm(1.0, 60.0, 5.001)
        .unwrap_err()
        .contains("flat_mm"));
    assert!(v_bit_width_mm(f64::NAN, 60.0, 0.1).is_err());
    assert!(v_bit_width_mm(1.0, f64::INFINITY, 0.1).is_err());
}

#[test]
fn the_width_of_a_v_bit_cut_follows_the_formula_and_never_shrinks_with_the_depth() {
    assert_eq!(v_bit_width_mm(0.0, 60.0, 0.1).unwrap(), 0.1);
    assert!(close(v_bit_width_mm(10.0, 90.0, 0.0).unwrap(), 20.0));
    let mut rng = Lcg(7);
    for _ in 0..200 {
        let angle = 10.0 + rng.below(161) as f64;
        let flat = rng.below(51) as f64 / 10.0;
        let shallow = rng.below(500) as f64 / 10.0;
        let deep = shallow + rng.below(500) as f64 / 10.0;
        let a = v_bit_width_mm(shallow, angle, flat).unwrap();
        let b = v_bit_width_mm(deep, angle, flat).unwrap();
        assert!(b >= a, "angle {angle} flat {flat}: {a} then {b}");
        assert!(a >= flat);
    }
}

#[test]
fn the_working_diameter_of_a_v_bit_is_the_width_of_the_cut_between_the_floor_and_the_tool() {
    let mut params = CalcParams::default();
    let t = tool("v", Kind::VBit, 0.2);
    let (_, deep) =
        feeds_speeds_traced_at(&t, Material::Fr4, 24000.0, &params, Some(10.0)).unwrap();
    assert_eq!(deep.working_mm, 0.2);
    assert!(deep.v_width_mm.unwrap() > 0.2);
    let (_, surface) =
        feeds_speeds_traced_at(&t, Material::Fr4, 24000.0, &params, Some(0.0)).unwrap();
    assert!(close(surface.working_mm, 0.1));
    params.v_bit_flat_mm = 0.0;
    let (_, floor) =
        feeds_speeds_traced_at(&t, Material::Fr4, 24000.0, &params, Some(0.0)).unwrap();
    assert!(close(floor.working_mm, 0.01));
}

#[test]
fn without_a_depth_the_point_of_a_v_bit_is_kept_between_the_two_working_limits() {
    let params = CalcParams::default();
    for (diameter, expected) in [(0.05, 0.1), (0.3, 0.3), (3.175, 0.5)] {
        let t = tool("v", Kind::VBit, diameter);
        let (_, trace) = feeds_speeds_traced(&t, Material::Wood, 24000.0, &params).unwrap();
        assert!(trace.uses_point);
        assert!(
            close(trace.working_mm, expected),
            "{diameter}: {}",
            trace.working_mm
        );
        assert!(trace.v_width_mm.is_none() && trace.depth_mm.is_none());
    }
}

#[test]
fn the_materials_are_read_by_their_key_in_any_case_and_each_has_its_own_place() {
    for (i, m) in MATERIALS.into_iter().enumerate() {
        assert_eq!(m.index(), i);
        assert_eq!(Material::parse(m.key()), Ok(m));
        assert_eq!(Material::parse(&m.key().to_uppercase()), Ok(m));
    }
    assert_eq!(Material::parse(" FR-4 "), Ok(Material::Fr4));
    assert_eq!(Material::parse(" Aluminum "), Ok(Material::Aluminium));
    assert!(Material::parse("steel")
        .unwrap_err()
        .contains("'steel' is not a material"));
    assert!(Material::parse("").is_err());
}

#[test]
fn the_numbers_of_the_calculation_have_inclusive_limits_and_the_message_names_the_one() {
    assert!(CalcParams::default().validate().is_ok());
    let plunge = |v: f64| CalcParams {
        drill_plunge: v,
        ..CalcParams::default()
    };
    assert!(plunge(1.0).validate().is_ok());
    assert!(plunge(1.0001)
        .validate()
        .unwrap_err()
        .contains("drill_plunge"));
    let mut p = CalcParams::default();
    p.surface_speed[Material::Aluminium.index()] = 2000.0;
    assert!(p.validate().is_ok());
    p.surface_speed[Material::Aluminium.index()] = 2000.5;
    assert!(p
        .validate()
        .unwrap_err()
        .contains("surface_speed_aluminium"));
    let p = CalcParams {
        working_min_mm: 0.6,
        ..CalcParams::default()
    };
    assert!(p.validate().unwrap_err().contains("working_min_mm"));
    let mut p = CalcParams::default();
    p.chip_fraction[Material::Brass.index()] = f64::NAN;
    assert!(p.validate().unwrap_err().contains("chip_fraction_brass"));
}

#[test]
fn the_feed_grows_with_the_flutes_and_the_speed_falls_with_the_diameter() {
    let params = CalcParams::default();
    let one = {
        let mut t = tool("m", Kind::EndMill, 3.0);
        t.flutes = 1;
        feeds_speeds_traced(&t, Material::Wood, 24000.0, &params)
            .unwrap()
            .1
            .feed
    };
    for flutes in 1..=8u32 {
        let mut t = tool("m", Kind::EndMill, 3.0);
        t.flutes = flutes;
        let (_, trace) = feeds_speeds_traced(&t, Material::Wood, 24000.0, &params).unwrap();
        assert!(close(trace.feed, one * f64::from(flutes)), "{flutes}");
    }
    let mut last = f64::MAX;
    for diameter in [0.5, 1.0, 2.0, 3.175, 6.0, 12.7, 25.0] {
        let cut = feeds_speeds(
            &tool("m", Kind::EndMill, diameter),
            Material::Fr4,
            400_000.0,
        )
        .unwrap();
        assert!(cut.rpm <= last, "{diameter}: {} after {last}", cut.rpm);
        last = cut.rpm;
    }
}

// ---------------------------------------------------------------- the crib

#[test]
fn the_wear_of_the_holes_changes_at_ninety_and_at_one_hundred_percent() {
    let mut t = tool("h", Kind::Drill, 1.0);
    t.max_holes = 100;
    for (holes, soon, replace) in [
        (0, false, false),
        (89, false, false),
        (90, true, false),
        (99, true, false),
        (100, false, true),
        (500, false, true),
    ] {
        t.holes = holes;
        match t.wear() {
            Wear::Good(f) => assert!(!soon && !replace && f < 0.9, "{holes}"),
            Wear::Soon(f) => assert!(soon && (0.9..1.0).contains(&f), "{holes}"),
            Wear::Replace => assert!(replace, "{holes}"),
        }
    }
}

#[test]
fn what_is_left_never_goes_below_zero_and_a_limit_that_is_not_set_has_no_number() {
    let mut t = tool("r", Kind::Drill, 1.0);
    let none = t.remaining();
    assert!(none.hours.is_none() && none.holes.is_none());
    t.max_hours = 10.0;
    t.max_holes = 100;
    t.hours = 4.0;
    t.holes = 30;
    let left = t.remaining();
    assert_eq!(left.hours, Some(6.0));
    assert_eq!(left.holes, Some(70));
    t.hours = 50.0;
    t.holes = u64::MAX;
    let over = t.remaining();
    assert_eq!(over.hours, Some(0.0));
    assert_eq!(over.holes, Some(0));
}

#[test]
fn replacing_a_tool_zeroes_the_counters_and_keeps_everything_else() {
    let mut crib = Crib::default();
    let mut t = tool("worn", Kind::EndMill, 2.0);
    t.hours = 3.5;
    t.meters = 120.0;
    t.holes = 9;
    t.max_hours = 4.0;
    t.max_holes = 10;
    t.extra
        .insert("note".into(), serde_json::Value::String("kept".into()));
    let id = crib.add(t).unwrap();
    assert_eq!(crib.replace_tool(id).unwrap(), (3.5, 120.0, 9));
    let after = crib.get(id).unwrap();
    assert_eq!((after.hours, after.meters, after.holes), (0.0, 0.0, 0));
    assert_eq!((after.max_hours, after.max_holes), (4.0, 10));
    assert_eq!(after.name, "worn");
    assert!(after.extra.contains_key("note"));
    assert!(crib.replace_tool(id + 1).unwrap_err().contains("no tool"));
}

#[test]
fn the_last_number_that_fits_is_given_once_and_then_the_crib_says_there_is_none_left() {
    let mut crib = Crib {
        next_id: u32::MAX,
        ..Crib::default()
    };
    assert_eq!(crib.add(tool("a", Kind::Drill, 1.0)).unwrap(), u32::MAX);
    let why = crib.add(tool("b", Kind::Drill, 1.0)).unwrap_err();
    assert!(why.contains("too many tools"), "{why}");
    assert_eq!(crib.tools.len(), 1);
    let mut sheets = Crib {
        next_sheet_id: u32::MAX,
        ..Crib::default()
    };
    assert_eq!(
        sheets.add_sheet(Sheet::new("a", "steel", 0.1)).unwrap(),
        u32::MAX
    );
    let why = sheets.add_sheet(Sheet::new("b", "steel", 0.1)).unwrap_err();
    assert!(why.contains("too many sheets"), "{why}");
    assert_eq!(sheets.sheets.len(), 1);
}

#[test]
fn a_failed_update_leaves_the_crib_as_it_was_and_a_tool_may_keep_its_own_name() {
    let mut crib = Crib::default();
    let a = crib.add(tool("alpha", Kind::Drill, 1.0)).unwrap();
    let b = crib.add(tool("beta", Kind::Drill, 2.0)).unwrap();
    let before = crib.clone();
    let mut clash = crib.get(b).unwrap().clone();
    clash.name = "ALPHA".into();
    assert!(crib.update(clash).unwrap_err().contains("already"));
    let mut bad = crib.get(b).unwrap().clone();
    bad.diameter_mm = 0.0;
    assert!(crib.update(bad).unwrap_err().contains("diameter_mm"));
    let mut ghost = crib.get(b).unwrap().clone();
    ghost.id = 99;
    assert!(crib.update(ghost).unwrap_err().contains("no tool 99"));
    assert_eq!(crib, before);
    let mut same = crib.get(a).unwrap().clone();
    same.name = "Alpha".into();
    same.diameter_mm = 1.5;
    crib.update(same).unwrap();
    assert_eq!(crib.get(a).unwrap().name, "Alpha");
    assert_eq!(crib.next_id, before.next_id);
}

#[test]
fn a_list_is_ordered_the_same_way_every_time_and_ties_go_by_number() {
    let mut crib = Crib::default();
    for (name, kind) in [
        ("bravo", Kind::Drill),
        ("Alpha", Kind::EndMill),
        ("charlie", Kind::Drill),
        ("delta", Kind::Drill),
    ] {
        crib.add(tool(name, kind, 1.0)).unwrap();
    }
    crib.tools[2].max_holes = 1;
    crib.tools[2].holes = 1;
    let ids = |s: &Selection| crib.select(s).iter().map(|t| t.id).collect::<Vec<u32>>();
    assert_eq!(ids(&Selection::default()), vec![1, 2, 3, 4]);
    let by_name = Selection {
        sort: Sort::Name,
        ..Selection::default()
    };
    assert_eq!(ids(&by_name), vec![2, 1, 3, 4]);
    let by_wear = Selection {
        sort: Sort::Wear,
        ..Selection::default()
    };
    assert_eq!(ids(&by_wear), vec![3, 1, 2, 4]);
    let worn_drills = Selection {
        kind: Some(Kind::Drill),
        worn_only: true,
        sort: Sort::Wear,
    };
    assert_eq!(ids(&worn_drills), vec![3]);
    let none = Selection {
        kind: Some(Kind::Laser),
        ..Selection::default()
    };
    assert!(ids(&none).is_empty());
}

#[test]
fn the_summary_has_one_row_for_each_kind_in_use_in_the_order_of_the_kinds_and_adds_without_wrapping(
) {
    let mut crib = Crib::default();
    crib.add(tool("n", Kind::Nozzle, 0.4)).unwrap();
    let mut d1 = tool("d1", Kind::Drill, 1.0);
    d1.holes = u64::MAX;
    d1.hours = 1.5;
    let mut d2 = tool("d2", Kind::Drill, 1.0);
    d2.holes = 5;
    d2.hours = 2.0;
    d2.max_holes = 5;
    crib.add(d1).unwrap();
    crib.add(d2).unwrap();
    let rows = crib.summary();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].kind, Kind::Drill);
    assert_eq!(rows[1].kind, Kind::Nozzle);
    assert_eq!(rows[0].tools, 2);
    assert_eq!(rows[0].holes, u64::MAX);
    assert_eq!(rows[0].hours, 3.5);
    assert_eq!(rows[0].to_replace, 1);
    assert!(Crib::default().summary().is_empty());
}

#[test]
fn a_use_that_cannot_be_added_leaves_the_counters_where_they_were() {
    let mut crib = Crib::default();
    let id = crib.add(tool("t", Kind::Drill, 1.0)).unwrap();
    crib.record_use(id, 1.0, 2.0, 3).unwrap();
    for (h, m) in [
        (f64::NAN, 1.0),
        (1.0, -0.5),
        (f64::INFINITY, 0.0),
        (-1.0, 0.0),
    ] {
        assert!(crib.record_use(id, h, m, 1).is_err());
    }
    assert!(crib.record_use(id + 1, 1.0, 1.0, 1).is_err());
    let t = crib.get(id).unwrap();
    assert_eq!((t.hours, t.meters, t.holes), (1.0, 2.0, 3));
    crib.record_use(id, f64::MAX, 0.0, 0).unwrap();
    let why = crib.record_use(id, f64::MAX, 0.0, 0).unwrap_err();
    assert!(why.contains("grow past"), "{why}");
    assert_eq!(crib.get(id).unwrap().hours, f64::MAX);
}

// ---------------------------------------------------------------- the check

#[test]
fn a_table_of_needs_is_read_with_the_number_of_each_row() {
    let text = format!("{NEEDS_HEADER}\ndrill 0.8,,0.8\nrouter,end-mill,\nfoil,SHEET,0.1\n");
    let needs = parse_needs(&text, 10).unwrap();
    assert_eq!(needs.len(), 3);
    assert_eq!(needs[0].row, 2);
    assert_eq!(needs[0].kind, NeedKind::AnyTool);
    assert_eq!(needs[0].size_mm, Some(0.8));
    assert_eq!(needs[1].kind, NeedKind::Tool(Kind::EndMill));
    assert_eq!(needs[1].size_mm, None);
    assert_eq!(needs[2].row, 4);
    assert_eq!(needs[2].kind, NeedKind::Sheet);
    let with_mark = format!("\u{feff}{NEEDS_HEADER}\n");
    assert!(parse_needs(&with_mark, 10).unwrap().is_empty());
}

#[test]
fn a_table_of_needs_that_is_wrong_names_its_row() {
    let head = format!("{NEEDS_HEADER}\n");
    for (rows, at) in [
        ("ok,,\nbad,,0\n", "row 3"),
        ("bad,,-1\n", "row 2"),
        ("bad,,nan\n", "row 2"),
        ("bad,,inf\n", "row 2"),
        ("bad,,abc\n", "row 2"),
        ("ok,,1\nx,saw,1\n", "row 3"),
        ("only,two\n", "row 2"),
        (",,1\n", "row 2"),
    ] {
        let why = parse_needs(&format!("{head}{rows}"), 10).unwrap_err();
        assert!(why.contains(at), "{rows:?}: {why}");
    }
    let long = format!("{head}{},,1\n", "n".repeat(81));
    assert!(parse_needs(&long, 10)
        .unwrap_err()
        .contains("80 characters"));
    let edge = format!("{head}{},,1\n", "n".repeat(80));
    assert!(parse_needs(&edge, 10).is_ok());
    assert!(parse_needs("", 10).is_err());
    assert!(parse_needs("name,kind\n", 10)
        .unwrap_err()
        .contains(NEEDS_HEADER));
}

#[test]
fn the_rows_of_a_table_of_needs_are_limited_and_the_limit_itself_is_allowed() {
    let rows = format!("{NEEDS_HEADER}\na,,\nb,,\nc,,\n");
    assert_eq!(parse_needs(&rows, 3).unwrap().len(), 3);
    assert!(parse_needs(&rows, 2)
        .unwrap_err()
        .contains("more than the 2 allowed"));
}

#[test]
fn nothing_to_check_in_an_empty_crib_is_a_pass_with_a_report_that_does_not_change() {
    let report = check(&Crib::default(), &[], 0.01);
    assert!(report.passed());
    assert_eq!(
        report.text(),
        "0 tools and sheets and 0 needs looked at, 0 problems, 0 warnings\nCHECK=PASS\n"
    );
}

fn need(row: usize, name: &str, kind: NeedKind, size: Option<f64>) -> Need {
    Need {
        row,
        name: name.into(),
        kind,
        size_mm: size,
    }
}

#[test]
fn the_size_of_a_need_may_be_off_by_the_tolerance_and_not_a_hair_more() {
    let mut crib = Crib::default();
    crib.add(tool("bit", Kind::Drill, 1.0)).unwrap();
    for (size, passes) in [
        (1.0, true),
        (1.5, true),
        (0.5, true),
        (1.6, false),
        (0.4, false),
    ] {
        let report = check(&crib, &[need(2, "BIT", NeedKind::AnyTool, Some(size))], 0.5);
        assert_eq!(report.passed(), passes, "{size}");
    }
    let report = check(
        &crib,
        &[need(7, "bit", NeedKind::Tool(Kind::EndMill), None)],
        0.5,
    );
    assert!(report.problems[0].contains("need on row 7"));
    assert!(report.problems[0].contains("is a drill, not a end-mill"));
}

#[test]
fn a_sheet_needs_its_kerf_from_itself_or_from_a_laser_that_is_thin_enough() {
    let mut crib = Crib::default();
    crib.add_sheet(Sheet::new("foil", "steel", 0.12)).unwrap();
    let report = check(&crib, &[], 0.01);
    assert!(!report.passed());
    assert!(report.problems[0].contains("no kerf_mm"));
    crib.add(tool("fat", Kind::Laser, 1.5)).unwrap();
    assert!(!check(&crib, &[], 0.01).passed());
    crib.add(tool("thin", Kind::Laser, 1.0)).unwrap();
    assert!(check(&crib, &[], 0.01).passed());
    let mut own = Crib::default();
    let mut s = Sheet::new("foil", "steel", 0.12);
    s.kerf_mm = 0.05;
    own.add_sheet(s).unwrap();
    assert!(check(&own, &[], 0.01).passed());
}

#[test]
fn a_need_for_a_sheet_says_which_thickness_the_crib_has_and_a_tool_name_for_a_sheet_gives_a_hint() {
    let mut crib = Crib::default();
    let mut s = Sheet::new("foil", "steel", 0.12);
    s.kerf_mm = 0.05;
    crib.add_sheet(s).unwrap();
    let ok = check(&crib, &[need(2, "FOIL", NeedKind::Sheet, Some(0.12))], 0.01);
    assert!(ok.passed());
    let thick = check(&crib, &[need(2, "foil", NeedKind::Sheet, Some(0.2))], 0.01);
    assert!(thick.problems[0].contains("0.12 mm thick, not 0.2 mm"));
    let missing = check(&crib, &[need(3, "gold", NeedKind::Sheet, None)], 0.01);
    assert!(missing.problems[0].contains("there is no sheet called 'gold'"));
    let hint = check(&crib, &[need(4, "foil", NeedKind::AnyTool, None)], 0.01);
    assert!(hint.problems[0].contains("write the kind sheet"));
    let plain = check(&crib, &[need(5, "nothing", NeedKind::AnyTool, None)], 0.01);
    assert!(!plain.problems[0].contains("write the kind sheet"));
}

#[test]
fn a_crib_that_disagrees_with_itself_fails_with_the_warnings_before_the_problems() {
    let mut crib = Crib::default();
    let a = crib.add(tool("a", Kind::Drill, 1.0)).unwrap();
    let mut worn = tool("worn", Kind::Drill, 1.0);
    worn.max_holes = 1;
    worn.holes = 1;
    crib.add(worn).unwrap();
    let mut twin = tool("A2", Kind::Drill, 1.0);
    twin.id = a;
    crib.tools.push(twin);
    crib.tools.push({
        let mut broken = tool("broken", Kind::Drill, 1.0);
        broken.id = 77;
        broken.diameter_mm = 0.0;
        broken
    });
    let needs = [need(2, "worn", NeedKind::AnyTool, None)];
    let report = check(&crib, &needs, 0.01);
    assert!(!report.passed());
    assert_eq!(report.warnings.len(), 1);
    assert_eq!(report.problems.len(), 2);
    assert!(report.problems.iter().any(|p| p.contains("used twice")));
    assert!(report.problems.iter().any(|p| p.contains("diameter_mm")));
    let text = report.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].starts_with("warning: "));
    assert!(lines[1].starts_with("problem: ") && lines[2].starts_with("problem: "));
    assert_eq!(*lines.last().unwrap(), "CHECK=FAIL");
    assert!(
        lines[3].starts_with("4 tools and sheets and 1 needs looked at, 2 problems, 1 warnings")
    );
}

#[test]
fn a_worn_tool_that_a_job_needs_is_a_warning_and_the_check_still_passes() {
    let mut crib = Crib::default();
    let mut worn = tool("worn", Kind::EndMill, 2.0);
    worn.max_hours = 1.0;
    worn.hours = 1.0;
    crib.add(worn).unwrap();
    let report = check(
        &crib,
        &[need(2, "worn", NeedKind::Tool(Kind::EndMill), Some(2.0))],
        0.01,
    );
    assert!(report.passed());
    assert_eq!(report.warnings.len(), 1);
    assert!(report.text().ends_with("CHECK=PASS\n"));
    assert!(report.text().starts_with("warning: need on row 2"));
}

// ---------------------------------------------------------------- the sheets

#[test]
fn the_limits_of_a_sheet_are_inclusive_at_the_top_and_open_at_zero_for_the_thickness() {
    fn ok(edit: impl Fn(&mut Sheet)) -> Result<(), String> {
        let mut s = Sheet::new("s", "steel", 1.0);
        edit(&mut s);
        s.validate()
    }
    assert!(ok(|s| s.thickness_mm = 50.0).is_ok());
    assert!(ok(|s| s.thickness_mm = 50.01)
        .unwrap_err()
        .contains("thickness_mm"));
    assert!(ok(|s| s.thickness_mm = 0.0)
        .unwrap_err()
        .contains("thickness_mm"));
    assert!(ok(|s| s.reduction_mm = 0.5).is_ok());
    assert!(ok(|s| s.reduction_mm = 0.51)
        .unwrap_err()
        .contains("reduction_mm"));
    assert!(ok(|s| s.kerf_mm = 1.0).is_ok());
    assert!(ok(|s| s.kerf_mm = -0.1).unwrap_err().contains("kerf_mm"));
    assert!(ok(|s| s.min_aspect_ratio = 10.0).is_ok());
    assert!(ok(|s| s.min_area_ratio = 10.5)
        .unwrap_err()
        .contains("min_area_ratio"));
    assert!(ok(|s| s.thickness_mm = f64::NAN).is_err());
    assert!(ok(|s| s.name = String::new()).is_err());
    assert!(ok(|s| s.material = "a\tb".into()).is_err());
}

#[test]
fn the_ratios_of_an_aperture_are_exact_and_a_size_that_makes_no_sense_is_refused() {
    let s = Sheet::new("s", "steel", 1.0);
    let (aspect, area) = s.ratios(2.0, 4.0).unwrap();
    assert_eq!(aspect, 2.0);
    assert!(close(area, 8.0 / 12.0));
    assert_eq!(s.ratios(4.0, 2.0).unwrap(), s.ratios(2.0, 4.0).unwrap());
    for (w, l) in [
        (0.0, 1.0),
        (1.0, -1.0),
        (f64::NAN, 1.0),
        (1.0, f64::INFINITY),
    ] {
        assert!(s.ratios(w, l).is_err(), "{w} x {l}");
    }
}

#[test]
fn the_reduction_of_an_aperture_never_leaves_it_with_no_size() {
    let mut s = Sheet::new("s", "steel", 1.0);
    s.reduction_mm = 0.1;
    let (w, l) = s.reduced(1.0, 0.3).unwrap();
    assert!(close(w, 0.8) && close(l, 0.1));
    assert!(s.reduced(1.0, 0.2).unwrap_err().contains("no size"));
    assert!(s.reduced(1.0, 0.1).unwrap_err().contains("no size"));
    assert!(s.reduced(0.0, 1.0).is_err());
}

#[test]
fn an_aperture_at_the_exact_limit_of_a_sheet_can_be_cut_and_a_hair_under_cannot() {
    let mut s = Sheet::new("foil", "steel", 0.1);
    assert!(
        s.check_aperture(0.001, 0.001).is_ok(),
        "no limit is not applied"
    );
    s.min_aspect_ratio = 1.0;
    assert!(s.check_aperture(0.1, 5.0).is_ok());
    assert!(s.check_aperture(5.0, 0.1).is_ok());
    let why = s.check_aperture(0.09, 5.0).unwrap_err();
    assert!(
        why.contains("aspect ratio") && why.contains("'foil'"),
        "{why}"
    );
    s.min_aspect_ratio = 0.0;
    s.min_area_ratio = 0.66;
    assert!(s.check_aperture(1.0, 1.0).is_ok());
    assert!(s
        .check_aperture(0.1, 0.1)
        .unwrap_err()
        .contains("area ratio"));
}

// ---------------------------------------------------------------- the tables and the document

#[test]
fn a_seeded_crib_with_gaps_in_its_numbers_reads_back_from_its_table_of_tools() {
    for seed in [1, 2, 3, 99, 2024] {
        let crib = random_crib(seed, 24);
        let text = csv::to_csv(&crib);
        let back = csv::from_csv(&text).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        assert_eq!(back.tools, crib.tools, "seed {seed}");
        assert_eq!(csv::to_csv(&back), text, "seed {seed}");
    }
}

#[test]
fn a_seeded_crib_reads_back_from_its_table_of_sheets_and_the_table_does_not_change() {
    for seed in [5, 6, 7, 31337] {
        let crib = random_crib(seed, 18);
        let text = csv::to_sheets_csv(&crib);
        let back = csv::from_sheets_csv(&text).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        assert_eq!(back.sheets, crib.sheets, "seed {seed}");
        assert_eq!(csv::to_sheets_csv(&back), text, "seed {seed}");
    }
}

#[test]
fn a_seeded_crib_reads_back_from_json_whole_and_the_document_is_the_same_text_again() {
    for seed in [11, 12, 13, 424_242] {
        let crib = random_crib(seed, 30);
        let json = store::to_json(&crib);
        let back = store::from_json(&json).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        assert_eq!(back, crib, "seed {seed}");
        assert_eq!(store::to_json(&back), json, "seed {seed}");
    }
}

#[test]
fn the_format_of_the_document_is_two_only_when_the_crib_has_had_a_sheet() {
    let version = |crib: &Crib| {
        let v: serde_json::Value = serde_json::from_str(&store::to_json(crib)).unwrap();
        v["schema_version"].as_u64().unwrap()
    };
    let mut crib = Crib::default();
    crib.add(tool("a", Kind::Drill, 1.0)).unwrap();
    assert_eq!(version(&crib), SCHEMA_VERSION);
    crib.add_sheet(Sheet::new("s", "steel", 0.1)).unwrap();
    assert_eq!(version(&crib), SHEETS_SCHEMA_VERSION);
    assert!(store::to_json(&Crib::default()).ends_with('\n'));
}

#[test]
fn a_document_cut_at_any_point_is_refused_and_never_panics() {
    let json = store::to_json(&random_crib(77, 6));
    let json = json.trim_end();
    let mut cuts = 0;
    for i in 0..json.len() {
        if json.is_char_boundary(i) {
            assert!(store::from_json(&json[..i]).is_err(), "cut at {i}");
            cuts += 1;
        }
    }
    assert!(cuts > 100);
    assert!(store::from_json(json).is_ok());
}

#[test]
fn a_document_with_a_mark_in_front_or_no_format_is_read_and_a_newer_format_is_refused() {
    assert!(store::from_json("\u{feff}{\"tools\": []}")
        .unwrap()
        .tools
        .is_empty());
    let newer = format!(
        "{{\"schema_version\": {}, \"tools\": []}}",
        SHEETS_SCHEMA_VERSION + 1
    );
    assert!(store::from_json(&newer)
        .unwrap_err()
        .contains("this program reads up to"));
    let zero =
        "{\"tools\": [{\"id\": 0, \"name\": \"a\", \"kind\": \"drill\", \"diameter_mm\": 1.0}]}";
    assert!(store::from_json(zero).unwrap_err().contains("tool 1"));
    let nameless = "{\"tools\": [{\"id\": 1, \"kind\": \"drill\", \"diameter_mm\": 1.0}]}";
    assert!(store::from_json(nameless).unwrap_err().contains("no name"));
}

#[test]
fn a_table_of_tools_that_is_wrong_says_so_and_the_other_table_is_recognised() {
    assert!(csv::from_csv("").unwrap_err().contains("empty"));
    assert!(csv::from_csv("id,name\n")
        .unwrap_err()
        .contains("must be exactly"));
    assert!(csv::from_csv(csv::SHEET_HEADER)
        .unwrap_err()
        .contains("table of sheets"));
    assert!(csv::from_sheets_csv(csv::HEADER)
        .unwrap_err()
        .contains("table of tools"));
    let empty = csv::from_csv(&format!("{}\n", csv::HEADER)).unwrap();
    assert!(empty.tools.is_empty() && empty.next_id == 0);
    let short = format!("{}\n1,a,drill\n", csv::HEADER);
    assert!(csv::from_csv(&short)
        .unwrap_err()
        .contains("row 2: it has 3 fields, not 14"));
    assert_eq!(csv::to_csv(&Crib::default()), format!("{}\n", csv::HEADER));
    assert_eq!(
        csv::to_sheets_csv(&Crib::default()),
        format!("{}\n", csv::SHEET_HEADER)
    );
}

#[test]
fn a_table_of_tools_with_a_number_or_a_name_used_twice_is_refused_at_the_second_row() {
    let row = |id: u32, name: &str| format!("{id},{name},drill,1,2,0,,3.175,carbide,0,0,0,0,0\n");
    let head = format!("{}\n", csv::HEADER);
    let same_id = format!("{head}{}{}", row(1, "a"), row(1, "b"));
    assert!(csv::from_csv(&same_id).unwrap_err().contains("row 3"));
    let zero_id = format!("{head}{}", row(0, "a"));
    assert!(csv::from_csv(&zero_id).unwrap_err().contains("row 2"));
    let same_name = format!("{head}{}{}", row(1, "Name"), row(2, "nAME"));
    assert!(csv::from_csv(&same_name)
        .unwrap_err()
        .contains("used twice"));
    let big = format!("{head}{}", row(1, "a").replace("drill,1,2", "drill,1,9"));
    assert!(csv::from_csv(&big).unwrap_err().contains("flutes"));
    let rows = format!("{head}{}{}{}", row(1, "a"), row(2, "b"), row(3, "c"));
    assert_eq!(csv::from_csv_limited(&rows, 3).unwrap().next_id, 4);
    assert!(csv::from_csv_limited(&rows, 2)
        .unwrap_err()
        .contains("more than the 2 allowed"));
}

// ---------------------------------------------------------------- the files and the settings

#[test]
fn a_file_of_exactly_the_limit_is_read_and_one_byte_more_is_refused_with_its_code() {
    let dir = scratch("limit");
    let file = dir.join("a.txt");
    fs::write(&file, "12345").unwrap();
    assert_eq!(read_text_limited(&file, 5).unwrap(), "12345");
    assert!(read_text_limited(&file, 4)
        .unwrap_err()
        .starts_with(TOO_LARGE));
    assert_eq!(
        read_text_limited(&file, 0).unwrap_err().split(':').next(),
        Some(TOO_LARGE)
    );
    fs::write(&file, "").unwrap();
    assert_eq!(read_text_limited(&file, 0).unwrap(), "");
    fs::write(&file, [0xffu8, 0xfe, 0x00]).unwrap();
    assert!(read_text_limited(&file, 10)
        .unwrap_err()
        .starts_with(NOT_TEXT));
    let missing = dir.join("none.txt");
    assert!(read_text_limited(&missing, 10)
        .unwrap_err()
        .starts_with(UNREADABLE));
}

#[test]
fn an_atomic_write_replaces_the_file_whole_and_leaves_no_temporary_file() {
    let dir = scratch("atomic");
    let file = dir.join("crib.json");
    write_atomic(&file, b"first").unwrap();
    write_atomic(&file, b"second and longer").unwrap();
    write_atomic(&file, b"3").unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "3");
    let names: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["crib.json".to_string()]);
    assert!(write_atomic(&dir.join("no_folder").join("x"), b"x").is_err());
}

#[test]
fn a_save_keeps_the_crib_it_replaced_as_a_backup_that_reads() {
    let dir = scratch("backup");
    let file = dir.join("crib.json");
    let first = random_crib(3, 5);
    let second = random_crib(4, 7);
    store::save(&first, &file).unwrap();
    assert!(!dir.join("crib.json.bak").exists());
    store::save(&second, &file).unwrap();
    assert_eq!(store::load(&file).unwrap(), second);
    assert_eq!(store::load(&dir.join("crib.json.bak")).unwrap(), first);
    fs::write(&file, "{ damaged").unwrap();
    assert_eq!(
        store::restore_from_backup(&file, 1).unwrap(),
        first.tools.len()
    );
    assert_eq!(store::load(&file).unwrap(), first);
    assert_eq!(
        fs::read_to_string(dir.join("crib.json.damaged")).unwrap(),
        "{ damaged"
    );
}

#[test]
fn the_settings_accept_each_limit_and_refuse_the_value_next_to_it() {
    for (line, ok) in [
        ("window_width = 720", true),
        ("window_width = 719", false),
        ("window_width = 7680", true),
        ("window_width = 7681", false),
        ("window_height = 480", true),
        ("window_height = 479", false),
        ("max_csv_rows = 1", true),
        ("max_csv_rows = 0", false),
        ("max_csv_rows = 100000", true),
        ("max_csv_rows = 100001", false),
        ("max_file_mib = 1024", true),
        ("max_file_mib = 1025", false),
        ("default_max_rpm = 1000", true),
        ("default_max_rpm = 999", false),
        ("check_tolerance_mm = 0.0001", true),
        ("check_tolerance_mm = 0", false),
        ("check_tolerance_mm = nan", false),
        ("window_width = 1e3", false),
        ("window_width = -1", false),
    ] {
        assert_eq!(Settings::parse(line).is_ok(), ok, "{line}");
    }
}

#[test]
fn a_mistake_in_the_settings_names_the_line_and_the_key() {
    let why = Settings::parse("# one\n\nwindow_width = 900\nwindow_height = tall\n").unwrap_err();
    assert!(
        why.contains("line 4") && why.contains("window_height"),
        "{why}"
    );
    let why = Settings::parse("language = klingon").unwrap_err();
    assert!(why.contains("line 1") && why.contains("klingon"), "{why}");
    let why = Settings::parse("surface_speed_steel = 100").unwrap_err();
    assert!(why.contains("names no material"), "{why}");
    let why = Settings::parse("working_min_mm = 0.6").unwrap_err();
    assert!(why.contains("working_min_mm"), "{why}");
}

#[test]
fn the_settings_read_the_language_in_any_case_and_a_material_by_the_end_of_its_key() {
    assert_eq!(Settings::parse("language = AUTO").unwrap().language, None);
    assert_eq!(
        Settings::parse("language = \"Zh\"").unwrap().language,
        Some(Lang::Zh)
    );
    let s = Settings::parse("surface_speed_brass = 321\nchip_fraction_acrylic = 0.05\n").unwrap();
    assert_eq!(s.params.surface_speed_of(Material::Brass), 321.0);
    assert_eq!(s.params.chip_fraction_of(Material::Acrylic), 0.05);
    assert_eq!(
        s.params.surface_speed_of(Material::Wood),
        CalcParams::default().surface_speed_of(Material::Wood)
    );
    assert_eq!(
        Settings::parse("something_else = 5\nno equals\n").unwrap(),
        Settings::default()
    );
    let dir = scratch("cfg");
    assert_eq!(Settings::load(&dir).unwrap(), Settings::default());
    fs::write(dir.join("tools.cfg"), "max_csv_rows = 7\n").unwrap();
    assert_eq!(Settings::load(&dir).unwrap().max_csv_rows, 7);
}

// ---------------------------------------------------------------- the eight languages

#[test]
fn the_eight_languages_have_their_own_code_and_name_and_come_back_from_their_code() {
    assert_eq!(Lang::ALL.len(), 8);
    let codes: Vec<&str> = Lang::ALL.iter().map(|l| l.code()).collect();
    assert_eq!(codes, vec!["en", "es", "de", "fr", "it", "ja", "zh", "pt"]);
    for (i, a) in Lang::ALL.iter().enumerate() {
        assert_eq!(Lang::from_code(a.code()), Some(*a));
        assert_eq!(Lang::from_code(&a.code().to_uppercase()), Some(*a));
        assert!(!a.name().is_empty());
        for b in &Lang::ALL[i + 1..] {
            assert_ne!(a.name(), b.name());
        }
    }
    assert_eq!(Lang::from_code("zh_CN.UTF-8"), Some(Lang::Zh));
    assert_eq!(Lang::from_code("pt-PT"), Some(Lang::Pt));
    assert_eq!(Lang::from_code("e"), None);
    assert_eq!(Lang::from_code("日本語"), None);
}

#[test]
fn no_sentence_starts_or_ends_with_a_blank_in_any_language() {
    assert!(Msg::ALL.len() > 20);
    for m in Msg::ALL {
        for l in Lang::ALL {
            let text = m.tr(l);
            assert_eq!(text, text.trim(), "{m:?} in {}", l.code());
        }
    }
}

#[test]
fn the_japanese_and_the_chinese_are_written_in_their_own_script_and_the_others_are_not_the_english()
{
    let total = Msg::ALL.len();
    let own_script = |lang: Lang| Msg::ALL.iter().filter(|m| !m.tr(lang).is_ascii()).count();
    assert!(
        own_script(Lang::Ja) * 10 >= total * 8,
        "ja {}",
        own_script(Lang::Ja)
    );
    assert!(
        own_script(Lang::Zh) * 10 >= total * 8,
        "zh {}",
        own_script(Lang::Zh)
    );
    for lang in [Lang::Es, Lang::De, Lang::Fr, Lang::It, Lang::Pt] {
        let own = Msg::ALL
            .iter()
            .filter(|m| m.tr(lang) != m.tr(Lang::En))
            .count();
        assert!(own * 2 >= total, "{} {own}", lang.code());
    }
}

#[test]
fn filling_a_text_leaves_a_place_with_no_value_and_ignores_a_value_with_no_place() {
    assert_eq!(fill("{0}-{1}", &["a"]), "a-{1}");
    assert_eq!(fill("{0}{0}", &["ab"]), "abab");
    assert_eq!(fill("", &["a"]), "");
    assert_eq!(fill("no place", &[]), "no place");
    assert_eq!(fill("{2}", &["a", "b", "c"]), "c");
}
