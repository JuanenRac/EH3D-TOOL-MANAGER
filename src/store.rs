//! The crib as a JSON document (`{"schema_version":1,"tools":[...]}`), read with care.

use crate::model::{Crib, Kind, Tool};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub const SCHEMA_VERSION: u64 = 1;
pub const MAX_TOOLS: usize = 10_000;

pub fn to_json(crib: &Crib) -> String {
    let tools: Vec<Value> = crib
        .tools
        .iter()
        .map(|t| {
            json!({"id": t.id, "name": t.name, "kind": t.kind.key(), "diameter_mm": t.diameter_mm, "flutes": t.flutes, "length_mm": t.length_mm,
                   "tip_angle_deg": t.tip_angle_deg, "shank_mm": t.shank_mm, "material": t.material, "hours": t.hours, "meters": t.meters,
                   "holes": t.holes, "max_hours": t.max_hours, "max_holes": t.max_holes})
        })
        .collect();
    serde_json::to_string_pretty(
        &json!({"schema_version": SCHEMA_VERSION, "next_id": crib.next_id, "tools": tools}),
    )
    .unwrap_or_default()
        + "\n"
}

fn number(v: &Value, key: &str, default: f64) -> Result<f64, String> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(x) => x
            .as_f64()
            .filter(|n| n.is_finite())
            .ok_or_else(|| format!("'{key}' is not a number")),
    }
}

/// Reads a whole, non-negative counter; a missing or null field is the default,
/// anything else that is not a whole number is refused naming the field.
fn whole(item: &Value, key: &str, default: u64) -> Result<u64, String> {
    match item.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(v) => v
            .as_u64()
            .ok_or_else(|| format!("{key} must be a whole number of 0 or more, not {v}")),
    }
}

pub fn from_json(text: &str) -> Result<Crib, String> {
    // a file saved with a byte order mark (PowerShell does) is still JSON
    let v: Value = serde_json::from_str(text.trim_start_matches('\u{feff}'))
        .map_err(|e| format!("the crib is not JSON: {e}"))?;
    let schema = v
        .get("schema_version")
        .and_then(|s| s.as_u64())
        .unwrap_or(0);
    if schema > SCHEMA_VERSION {
        return Err(format!(
            "the crib has format {schema}; this program reads up to {SCHEMA_VERSION}"
        ));
    }
    let list = v
        .get("tools")
        .and_then(|t| t.as_array())
        .ok_or("the crib has no list of tools")?;
    if list.len() > MAX_TOOLS {
        return Err(format!(
            "the crib has {} tools: more than the {MAX_TOOLS} allowed",
            list.len()
        ));
    }
    let mut crib = Crib::default();
    for (i, item) in list.iter().enumerate() {
        let fail = |e: String| format!("tool {}: {e}", i + 1);
        let name = item
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| fail("it has no name".into()))?;
        let kind =
            Kind::parse(item.get("kind").and_then(|k| k.as_str()).unwrap_or("")).map_err(fail)?;
        let mut t = Tool::new(name, kind, number(item, "diameter_mm", 0.0).map_err(fail)?);
        t.id = match item.get("id") {
            None | Some(Value::Null) => 0,
            Some(v) => v
                .as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| {
                    fail(format!(
                        "the id must be a whole number from 1 to {}, not {v}",
                        u32::MAX
                    ))
                })?,
        };
        t.flutes = u32::try_from(whole(item, "flutes", 2).map_err(fail)?)
            .map_err(|_| fail("flutes is too large".into()))?;
        t.length_mm = number(item, "length_mm", 0.0).map_err(fail)?;
        t.tip_angle_deg = match item.get("tip_angle_deg") {
            None | Some(Value::Null) => None,
            Some(_) => Some(number(item, "tip_angle_deg", 0.0).map_err(fail)?),
        };
        t.shank_mm = number(item, "shank_mm", 3.175).map_err(fail)?;
        t.material = item
            .get("material")
            .and_then(|m| m.as_str())
            .unwrap_or("carbide")
            .to_string();
        t.hours = number(item, "hours", 0.0).map_err(fail)?;
        t.meters = number(item, "meters", 0.0).map_err(fail)?;
        t.holes = whole(item, "holes", 0).map_err(fail)?;
        t.max_hours = number(item, "max_hours", 0.0).map_err(fail)?;
        t.max_holes = whole(item, "max_holes", 0).map_err(fail)?;
        t.validate().map_err(fail)?;
        if t.id == 0 || crib.tools.iter().any(|o| o.id == t.id) {
            return Err(fail(format!(
                "the number {} is missing or used twice",
                t.id
            )));
        }
        if crib
            .tools
            .iter()
            .any(|o| o.name.eq_ignore_ascii_case(&t.name))
        {
            return Err(fail(format!("the name '{}' is used twice", t.name)));
        }
        crib.tools.push(t);
    }
    // the next number is at least the one the file keeps and always above every number in use
    let kept = match v.get("next_id") {
        None | Some(Value::Null) => 0,
        Some(n) => n
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| format!("next_id must be a whole number, not {n}"))?,
    };
    crib.next_id = match crib.tools.iter().map(|t| t.id).max() {
        Some(used) => kept.max(used.saturating_add(1)),
        None => kept,
    };
    Ok(crib)
}

/// Writes the crib to a temporary file beside the real one and renames it over the real one, so that a cut or a full disk leaves the old crib whole.
pub fn save(crib: &Crib, file: &Path) -> Result<(), String> {
    let mut tmp = file.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp);
    let fail = |e: std::io::Error| format!("{}: {e}", file.display());
    if let Err(e) = fs::write(&tmp, to_json(crib)) {
        let _ = fs::remove_file(&tmp);
        return Err(fail(e));
    }
    fs::rename(&tmp, file).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        fail(e)
    })
}

pub fn load(file: &Path) -> Result<Crib, String> {
    from_json(&fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?)
}
