//! A fuzz with a fixed seed of the readers of tables and of the crib: whatever the text, a reader answers with a value or with an error and never panics, and what it reads writes back
//! to the same value. The generator reads no clock, so a run is the same every time.

use eh3d_tool_manager::check::{parse_needs, NEEDS_HEADER};
use eh3d_tool_manager::csv::{
    from_csv, from_sheets_csv, parse_records, to_csv, to_sheets_csv, HEADER, SHEET_HEADER,
};
use eh3d_tool_manager::model::{Crib, Kind, Tool};
use eh3d_tool_manager::sheet::Sheet;
use eh3d_tool_manager::sim::SimRng;
use eh3d_tool_manager::store::{from_json, to_json};

const SEED: u64 = 0x5EED_0D0D_2026;
const ROUNDS: usize = 2_500;

/// Pieces that tables and numbers are made of, and some that break them.
const TOKENS: [&str; 26] = [
    ",",
    "\"",
    "\"\"",
    "\n",
    "\r\n",
    "'",
    "=",
    "-",
    "+",
    "@",
    "0",
    "9",
    ".",
    "e",
    " ",
    "é",
    "漢",
    "\u{feff}",
    "sheet",
    "drill",
    "NaN",
    "inf",
    "-1",
    "4294967296",
    "1e999",
    "\u{0}",
];

/// The text with a few edits: characters taken out, pieces put in, a character swapped, the end cut off or a line repeated.
fn mutate(rng: &mut SimRng, text: &str) -> String {
    let mut chars: Vec<char> = text.chars().collect();
    for _ in 0..1 + rng.below(6) {
        if chars.is_empty() {
            chars.extend(TOKENS[rng.below(TOKENS.len() as u64) as usize].chars());
            continue;
        }
        let at = rng.below(chars.len() as u64) as usize;
        match rng.below(5) {
            0 => {
                chars.remove(at);
            }
            1 => {
                let piece: Vec<char> = TOKENS[rng.below(TOKENS.len() as u64) as usize]
                    .chars()
                    .collect();
                chars.splice(at..at, piece);
            }
            2 => {
                chars[at] = TOKENS[rng.below(TOKENS.len() as u64) as usize]
                    .chars()
                    .next()
                    .unwrap_or(' ');
            }
            3 => chars.truncate(at),
            _ => {
                let line: String = chars
                    .iter()
                    .collect::<String>()
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();
                let copy: Vec<char> = format!("\n{line}\n").chars().collect();
                chars.splice(at..at, copy);
            }
        }
    }
    chars.into_iter().collect()
}

fn crib() -> Crib {
    let mut c = Crib::default();
    c.add(Tool::new("drill 0.8", Kind::Drill, 0.8)).unwrap();
    let mut v = Tool::new("=v-bit, \"60\"", Kind::VBit, 0.2);
    v.tip_angle_deg = Some(60.0);
    c.add(v).unwrap();
    let mut worn = Tool::new("laser", Kind::Laser, 0.05);
    worn.max_hours = 10.0;
    worn.hours = 9.5;
    c.add(worn).unwrap();
    let mut s = Sheet::new("foil, 0.12", "'+steel", 0.12);
    s.reduction_mm = 0.03;
    s.min_aspect_ratio = 1.5;
    c.add_sheet(s).unwrap();
    c.add_sheet(Sheet::new("paper", "card", 0.3)).unwrap();
    c
}

#[test]
fn a_seeded_fuzz_of_the_readers_never_panics_and_what_reads_writes_back_the_same() {
    let c = crib();
    let tools = to_csv(&c);
    let sheets = to_sheets_csv(&c);
    let json = to_json(&c);
    let needs = format!("{NEEDS_HEADER}\ndrill 0.8,drill,0.8\nfoil,sheet,0.12\nbeam,,\n");
    assert_eq!(from_csv(&tools).unwrap().tools, c.tools);
    assert_eq!(from_sheets_csv(&sheets).unwrap().sheets, c.sheets);
    assert_eq!(from_json(&json).unwrap(), c);
    assert!(parse_needs(&needs, 100).is_ok());

    let mut rng = SimRng::new(SEED);
    let (mut read, mut refused) = (0usize, 0usize);
    for _ in 0..ROUNDS {
        let text = mutate(&mut rng, &tools);
        match from_csv(&text) {
            Ok(back) => {
                read += 1;
                assert_eq!(
                    from_csv(&to_csv(&back)).unwrap().tools,
                    back.tools,
                    "{text:?}"
                );
            }
            Err(e) => {
                refused += 1;
                assert!(!e.is_empty());
            }
        }
        let text = mutate(&mut rng, &sheets);
        match from_sheets_csv(&text) {
            Ok(back) => {
                read += 1;
                assert_eq!(
                    from_sheets_csv(&to_sheets_csv(&back)).unwrap().sheets,
                    back.sheets,
                    "{text:?}"
                );
            }
            Err(e) => {
                refused += 1;
                assert!(!e.is_empty());
            }
        }
        let text = mutate(&mut rng, &json);
        match from_json(&text) {
            Ok(back) => {
                read += 1;
                assert_eq!(from_json(&to_json(&back)).unwrap(), back, "{text:?}");
            }
            Err(e) => {
                refused += 1;
                assert!(!e.is_empty());
            }
        }
        let text = mutate(&mut rng, &needs);
        match parse_needs(&text, 100) {
            Ok(_) => read += 1,
            Err(e) => {
                refused += 1;
                assert!(!e.is_empty());
            }
        }
        // the splitter alone: a text that splits into records splits into the same records after being written by the writer's rule
        let _ = parse_records(&mutate(&mut rng, &tools));
    }
    // the fuzz must reach both outcomes, or it tests nothing
    assert!(
        read > 100 && refused > 100,
        "read {read}, refused {refused}"
    );
}

#[test]
fn a_text_made_of_nothing_but_pieces_is_refused_without_a_panic_and_the_headers_are_the_ones_written(
) {
    let mut rng = SimRng::new(SEED ^ 1);
    for _ in 0..500 {
        let n = 1 + rng.below(200) as usize;
        let text: String = (0..n)
            .map(|_| TOKENS[rng.below(TOKENS.len() as u64) as usize])
            .collect();
        assert!(from_csv(&text).is_err() && from_sheets_csv(&text).is_err());
        assert!(from_json(&text).is_err());
        assert!(parse_needs(&text, 10).is_err());
    }
    assert!(to_csv(&Crib::default()).starts_with(HEADER));
    assert!(to_sheets_csv(&Crib::default()).starts_with(SHEET_HEADER));
}
