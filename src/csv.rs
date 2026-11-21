//! The crib as a CSV table, to open in a spreadsheet: a header row, commas, and double quotes around a text that has a comma, a quote or a line break.

use crate::model::{Crib, Kind, Tool};
use crate::sheet::{Sheet, KIND_KEY};

pub const HEADER: &str = "id,name,kind,diameter_mm,flutes,length_mm,tip_angle_deg,shank_mm,material,hours,meters,holes,max_hours,max_holes";

/// The header of the table of the sheets of stencil material (`kind` is always `sheet`). The table of the tools keeps its own header, so that a program that reads the
/// tools reads them as it always did; a program that wants the sheets reads this table, by the names of its columns.
pub const SHEET_HEADER: &str =
    "id,name,kind,material,thickness_mm,reduction_mm,kerf_mm,min_aspect_ratio,min_area_ratio";

/// The most rows read from a table when nothing else is said (`max_csv_rows` of `tools.cfg`).
pub const DEFAULT_MAX_ROWS: usize = 10_000;

/// A text that a spreadsheet would run as a formula when it opens the file starts with one of these.
const FORMULA_START: [char; 6] = ['=', '+', '-', '@', '\t', '\r'];

/// True when the text, after any quote marks in front, starts like a formula: such a text carries one more quote mark in the table, so that reading it back takes exactly one away.
fn is_marked(text: &str) -> bool {
    text.trim_start_matches('\'').starts_with(FORMULA_START)
}

/// The text as it goes in the table: one that starts like a formula (`=1+1`), or like a formula that already has quote marks in front (`'=1+1`), gets one more quote mark in front, so that
/// a spreadsheet shows it as text and `unformula` gives back exactly what was written.
fn neutralise(text: &str) -> String {
    if is_marked(text) {
        format!("'{text}")
    } else {
        text.to_string()
    }
}

fn unformula(text: &str) -> String {
    match text.strip_prefix('\'') {
        Some(rest) if is_marked(rest) => rest.to_string(),
        _ => text.to_string(),
    }
}

fn quote(text: &str) -> String {
    let text = neutralise(text);
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text
    }
}

pub fn to_csv(crib: &Crib) -> String {
    let mut out = String::from(HEADER);
    out.push('\n');
    for t in &crib.tools {
        let angle = t.tip_angle_deg.map(|a| a.to_string()).unwrap_or_default();
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            t.id,
            quote(&t.name),
            t.kind.key(),
            t.diameter_mm,
            t.flutes,
            t.length_mm,
            angle,
            t.shank_mm,
            quote(&t.material),
            t.hours,
            t.meters,
            t.holes,
            t.max_hours,
            t.max_holes
        ));
    }
    out
}

/// Splits the text in records of fields, honouring the quotes. An unclosed quote is an error.
pub fn parse_records(text: &str) -> Result<Vec<Vec<String>>, String> {
    let mut records = Vec::new();
    let mut record = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    // true right after the quote that closes a field: only a comma or the end of the row may follow
    let mut closed = false;
    // the row of the table the quote that is open began in (the header is row 1)
    let mut quote_row = 1;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    quoted = false;
                    closed = true;
                }
            } else {
                field.push(c);
            }
        } else {
            if closed {
                if !matches!(c, ',' | '\n' | '\r') {
                    return Err(format!(
                        "row {}: there is '{c}' after the closing quote of a field",
                        records.len() + 1
                    ));
                }
                closed = false;
            }
            match c {
                '"' if field.is_empty() => {
                    quoted = true;
                    quote_row = records.len() + 1;
                }
                ',' => record.push(std::mem::take(&mut field)),
                '\r' => {}
                '\n' => {
                    record.push(std::mem::take(&mut field));
                    records.push(std::mem::take(&mut record));
                }
                _ => field.push(c),
            }
        }
    }
    if quoted {
        return Err(format!(
            "row {quote_row}: a quote is opened and never closed"
        ));
    }
    if !field.is_empty() || !record.is_empty() {
        record.push(field);
        records.push(record);
    }
    records.retain(|r| !(r.len() == 1 && r[0].trim().is_empty()));
    Ok(records)
}

/// Reads a crib from CSV; every row is checked like a tool of the JSON document and the message names the row. At most `DEFAULT_MAX_ROWS` rows are read.
pub fn from_csv(text: &str) -> Result<Crib, String> {
    from_csv_limited(text, DEFAULT_MAX_ROWS)
}

/// The rows and the header of a table, refused when it has more rows than `max_rows`.
pub(crate) fn table(
    text: &str,
    max_rows: usize,
) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let mut records = parse_records(text.trim_start_matches('\u{feff}'))?;
    if records.is_empty() {
        return Err("the CSV is empty".into());
    }
    let head = records.remove(0);
    if records.len() > max_rows {
        return Err(format!(
            "the table has {} rows: more than the {max_rows} allowed (max_csv_rows in tools.cfg)",
            records.len()
        ));
    }
    Ok((head, records))
}

/// Like `from_csv`, with the most rows given (`max_csv_rows` of `tools.cfg`).
pub fn from_csv_limited(text: &str, max_rows: usize) -> Result<Crib, String> {
    let (head, rows) = table(text, max_rows)?;
    let rows = &rows[..];
    if head.join(",") != HEADER {
        if head.join(",") == SHEET_HEADER {
            return Err("this is a table of sheets, not of tools: read it with the sheets commands (import-sheets-csv)".into());
        }
        return Err(format!("the first row must be exactly: {HEADER}"));
    }
    let mut crib = Crib::default();
    for (i, row) in rows.iter().enumerate() {
        let n = i + 2;
        if row.len() != 14 {
            return Err(format!("row {n}: it has {} fields, not 14", row.len()));
        }
        let num = |k: usize, name: &str| -> Result<f64, String> {
            if row[k].trim().is_empty() {
                return Ok(0.0);
            }
            row[k]
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| format!("row {n}: {name} '{}' is not a number", row[k]))
        };
        let int = |k: usize, name: &str| -> Result<u64, String> {
            row[k]
                .trim()
                .parse::<u64>()
                .map_err(|_| format!("row {n}: {name} '{}' is not a whole number", row[k]))
        };
        let kind = Kind::parse(&row[2]).map_err(|e| format!("row {n}: {e}"))?;
        let mut t = Tool::new(&unformula(&row[1]), kind, num(3, "diameter_mm")?);
        t.id = u32::try_from(int(0, "id")?).map_err(|_| format!("row {n}: the id is too large"))?;
        t.flutes = u32::try_from(int(4, "flutes")?).map_err(|_| {
            format!(
                "row {n}: flutes '{}' is too large, it must be from 1 to 8",
                row[4]
            )
        })?;
        t.length_mm = num(5, "length_mm")?;
        t.tip_angle_deg = if row[6].trim().is_empty() {
            None
        } else {
            Some(num(6, "tip_angle_deg")?)
        };
        t.shank_mm = num(7, "shank_mm")?;
        t.material = unformula(&row[8]);
        t.hours = num(9, "hours")?;
        t.meters = num(10, "meters")?;
        t.holes = int(11, "holes")?;
        t.max_hours = num(12, "max_hours")?;
        t.max_holes = int(13, "max_holes")?;
        t.validate().map_err(|e| format!("row {n}: {e}"))?;
        if t.id == 0 || crib.tools.iter().any(|o| o.id == t.id) {
            return Err(format!("row {n}: the id {} is missing or used twice", t.id));
        }
        if crib
            .tools
            .iter()
            .any(|o| o.name.eq_ignore_ascii_case(&t.name))
        {
            return Err(format!("row {n}: the name '{}' is used twice", t.name));
        }
        crib.tools.push(t);
    }
    if let Some(used) = crib.tools.iter().map(|t| t.id).max() {
        crib.next_id = used.saturating_add(1);
    }
    Ok(crib)
}

/// The sheets of the crib as a table (`SHEET_HEADER`); a crib with no sheet gives the header only.
pub fn to_sheets_csv(crib: &Crib) -> String {
    let mut out = String::from(SHEET_HEADER);
    out.push('\n');
    for s in &crib.sheets {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            s.id,
            quote(&s.name),
            KIND_KEY,
            quote(&s.material),
            s.thickness_mm,
            s.reduction_mm,
            s.kerf_mm,
            s.min_aspect_ratio,
            s.min_area_ratio
        ));
    }
    out
}

/// Reads a table of sheets into a crib that has only those sheets; every row is checked like a sheet of the JSON document and the message names the row.
pub fn from_sheets_csv(text: &str) -> Result<Crib, String> {
    from_sheets_csv_limited(text, DEFAULT_MAX_ROWS)
}

/// Like `from_sheets_csv`, with the most rows given (`max_csv_rows` of `tools.cfg`).
pub fn from_sheets_csv_limited(text: &str, max_rows: usize) -> Result<Crib, String> {
    let (head, rows) = table(text, max_rows)?;
    if head.join(",") != SHEET_HEADER {
        if head.join(",") == HEADER {
            return Err("this is a table of tools, not of sheets: read it with import-csv".into());
        }
        return Err(format!("the first row must be exactly: {SHEET_HEADER}"));
    }
    let mut crib = Crib::default();
    for (i, row) in rows.iter().enumerate() {
        let n = i + 2;
        if row.len() != 9 {
            return Err(format!("row {n}: it has {} fields, not 9", row.len()));
        }
        let num = |k: usize, name: &str| -> Result<f64, String> {
            if row[k].trim().is_empty() {
                return Ok(0.0);
            }
            row[k]
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| format!("row {n}: {name} '{}' is not a number", row[k]))
        };
        if !row[2].trim().eq_ignore_ascii_case(KIND_KEY) {
            return Err(format!(
                "row {n}: the kind '{}' is not '{KIND_KEY}'",
                row[2]
            ));
        }
        let mut s = Sheet::new(
            &unformula(&row[1]),
            &unformula(&row[3]),
            num(4, "thickness_mm")?,
        );
        s.id = row[0]
            .trim()
            .parse::<u32>()
            .map_err(|_| format!("row {n}: the id '{}' is not a whole number", row[0]))?;
        s.reduction_mm = num(5, "reduction_mm")?;
        s.kerf_mm = num(6, "kerf_mm")?;
        s.min_aspect_ratio = num(7, "min_aspect_ratio")?;
        s.min_area_ratio = num(8, "min_area_ratio")?;
        s.validate().map_err(|e| format!("row {n}: {e}"))?;
        if s.id == 0 || crib.sheets.iter().any(|o| o.id == s.id) {
            return Err(format!("row {n}: the id {} is missing or used twice", s.id));
        }
        if crib
            .sheets
            .iter()
            .any(|o| o.name.eq_ignore_ascii_case(&s.name))
        {
            return Err(format!("row {n}: the name '{}' is used twice", s.name));
        }
        crib.sheets.push(s);
    }
    if let Some(used) = crib.sheets.iter().map(|s| s.id).max() {
        crib.next_sheet_id = used.saturating_add(1);
    }
    Ok(crib)
}
