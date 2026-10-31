//! The crib as a CSV table, to open in a spreadsheet: a header row, commas, and double quotes around a text that has a comma, a quote or a line break.

use crate::model::{Crib, Kind, Tool};

pub const HEADER: &str = "id,name,kind,diameter_mm,flutes,length_mm,tip_angle_deg,shank_mm,material,hours,meters,holes,max_hours,max_holes";

fn quote(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_string()
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
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    quoted = false;
                }
            } else {
                field.push(c);
            }
        } else {
            match c {
                '"' if field.is_empty() => quoted = true,
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
        return Err("a quote is opened and never closed".into());
    }
    if !field.is_empty() || !record.is_empty() {
        record.push(field);
        records.push(record);
    }
    records.retain(|r| !(r.len() == 1 && r[0].trim().is_empty()));
    Ok(records)
}

/// Reads a crib from CSV; every row is checked like a tool of the JSON document and the message names the row.
pub fn from_csv(text: &str) -> Result<Crib, String> {
    let records = parse_records(text.trim_start_matches('\u{feff}'))?;
    let (head, rows) = records.split_first().ok_or("the CSV is empty")?;
    if head.join(",") != HEADER {
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
        let mut t = Tool::new(&row[1], kind, num(3, "diameter_mm")?);
        t.id = u32::try_from(int(0, "id")?).map_err(|_| format!("row {n}: the id is too large"))?;
        t.flutes = u32::try_from(int(4, "flutes")?).unwrap_or(0);
        t.length_mm = num(5, "length_mm")?;
        t.tip_angle_deg = if row[6].trim().is_empty() {
            None
        } else {
            Some(num(6, "tip_angle_deg")?)
        };
        t.shank_mm = num(7, "shank_mm")?;
        t.material = row[8].clone();
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
    Ok(crib)
}
