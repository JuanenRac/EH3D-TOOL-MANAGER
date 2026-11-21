//! The crib as a JSON document (`{"schema_version":1,"tools":[...]}`), read with care. A crib that has sheets of stencil material is written as format 2 (the same document with
//! `sheets` and `next_sheet_id`), so that a program that only knows format 1 refuses it instead of dropping the sheets when it saves; a crib with no sheet is written as format 1,
//! byte for byte as before.

use crate::files;
use crate::model::{Crib, Kind, Tool};
use crate::settings::DEFAULT_MAX_FILE_MIB;
use crate::sheet::Sheet;
use serde_json::{json, Value};
use std::path::Path;

/// The format of a crib with tools only.
pub const SCHEMA_VERSION: u64 = 1;
/// The format of a crib that has sheets; the highest this program reads.
pub const SHEETS_SCHEMA_VERSION: u64 = 2;
pub const MAX_TOOLS: usize = 10_000;
/// The most sheets read from a crib.
pub const MAX_SHEETS: usize = 10_000;

/// The keys of a sheet that this program reads and writes; any other key of a sheet is kept in `Sheet::extra`.
const SHEET_KEYS: [&str; 9] = [
    "id",
    "name",
    "kind",
    "material",
    "thickness_mm",
    "reduction_mm",
    "kerf_mm",
    "min_aspect_ratio",
    "min_area_ratio",
];

/// The keys of a tool that this program reads and writes; any other key of a tool is kept in `Tool::extra`.
const KNOWN_KEYS: [&str; 14] = [
    "id",
    "name",
    "kind",
    "diameter_mm",
    "flutes",
    "length_mm",
    "tip_angle_deg",
    "shank_mm",
    "material",
    "hours",
    "meters",
    "holes",
    "max_hours",
    "max_holes",
];

pub fn to_json(crib: &Crib) -> String {
    let tools: Vec<Value> = crib
        .tools
        .iter()
        .map(|t| {
            let mut item = json!({"id": t.id, "name": t.name, "kind": t.kind.key(), "diameter_mm": t.diameter_mm, "flutes": t.flutes, "length_mm": t.length_mm,
                   "tip_angle_deg": t.tip_angle_deg, "shank_mm": t.shank_mm, "material": t.material, "hours": t.hours, "meters": t.meters,
                   "holes": t.holes, "max_hours": t.max_hours, "max_holes": t.max_holes});
            if let Some(object) = item.as_object_mut() {
                for (key, value) in &t.extra {
                    object.entry(key.clone()).or_insert_with(|| value.clone());
                }
            }
            item
        })
        .collect();
    let mut document =
        json!({"schema_version": SCHEMA_VERSION, "next_id": crib.next_id, "tools": tools});
    if !crib.sheets.is_empty() || crib.next_sheet_id > 0 {
        let sheets: Vec<Value> = crib.sheets.iter().map(sheet_to_json).collect();
        document["schema_version"] = json!(SHEETS_SCHEMA_VERSION);
        document["next_sheet_id"] = json!(crib.next_sheet_id);
        document["sheets"] = Value::Array(sheets);
    }
    serde_json::to_string_pretty(&document).unwrap_or_default() + "\n"
}

fn sheet_to_json(s: &Sheet) -> Value {
    let mut item = json!({"id": s.id, "name": s.name, "kind": crate::sheet::KIND_KEY, "material": s.material, "thickness_mm": s.thickness_mm,
           "reduction_mm": s.reduction_mm, "kerf_mm": s.kerf_mm, "min_aspect_ratio": s.min_aspect_ratio, "min_area_ratio": s.min_area_ratio});
    if let Some(object) = item.as_object_mut() {
        for (key, value) in &s.extra {
            object.entry(key.clone()).or_insert_with(|| value.clone());
        }
    }
    item
}

fn sheet_from_json(item: &Value) -> Result<Sheet, String> {
    let name = item
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or("it has no name")?;
    if let Some(kind) = item.get("kind") {
        if kind.as_str().map(str::to_ascii_lowercase).as_deref() != Some(crate::sheet::KIND_KEY) {
            return Err(format!("the kind is {kind}, not \"sheet\""));
        }
    }
    let mut s = Sheet::new(
        name,
        item.get("material").and_then(|m| m.as_str()).unwrap_or(""),
        number(item, "thickness_mm", 0.0)?,
    );
    s.id = match item.get("id") {
        None | Some(Value::Null) => 0,
        Some(v) => v
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| {
                format!(
                    "the id must be a whole number from 1 to {}, not {v}",
                    u32::MAX
                )
            })?,
    };
    s.reduction_mm = number(item, "reduction_mm", 0.0)?;
    s.kerf_mm = number(item, "kerf_mm", 0.0)?;
    s.min_aspect_ratio = number(item, "min_aspect_ratio", 0.0)?;
    s.min_area_ratio = number(item, "min_area_ratio", 0.0)?;
    if let Some(object) = item.as_object() {
        s.extra = object
            .iter()
            .filter(|(key, _)| !SHEET_KEYS.contains(&key.as_str()))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
    }
    s.validate()?;
    Ok(s)
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
    if schema > SHEETS_SCHEMA_VERSION {
        return Err(format!(
            "the crib has format {schema}; this program reads up to {SHEETS_SCHEMA_VERSION}"
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
        if let Some(object) = item.as_object() {
            t.extra = object
                .iter()
                .filter(|(key, _)| !KNOWN_KEYS.contains(&key.as_str()))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
        }
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
    if let Some(list) = v.get("sheets") {
        let list = list
            .as_array()
            .ok_or("the sheets of the crib are not a list")?;
        if list.len() > MAX_SHEETS {
            return Err(format!(
                "the crib has {} sheets: more than the {MAX_SHEETS} allowed",
                list.len()
            ));
        }
        for (i, item) in list.iter().enumerate() {
            let fail = |e: String| format!("sheet {}: {e}", i + 1);
            let s = sheet_from_json(item).map_err(fail)?;
            if s.id == 0 || crib.sheets.iter().any(|o| o.id == s.id) {
                return Err(format!(
                    "sheet {}: the number {} is missing or used twice",
                    i + 1,
                    s.id
                ));
            }
            if crib
                .sheets
                .iter()
                .any(|o| o.name.eq_ignore_ascii_case(&s.name))
            {
                return Err(format!(
                    "sheet {}: the name '{}' is used twice",
                    i + 1,
                    s.name
                ));
            }
            crib.sheets.push(s);
        }
    }
    let kept_sheets = match v.get("next_sheet_id") {
        None | Some(Value::Null) => 0,
        Some(n) => n
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| format!("next_sheet_id must be a whole number, not {n}"))?,
    };
    crib.next_sheet_id = match crib.sheets.iter().map(|s| s.id).max() {
        Some(used) => kept_sheets.max(used.saturating_add(1)),
        None => kept_sheets,
    };
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
/// The file that was there before is kept beside it as `<file>.bak` (best effort: a copy that fails does not stop the save).
pub fn save(crib: &Crib, file: &Path) -> Result<(), String> {
    if file.is_file() {
        let mut backup = file.as_os_str().to_owned();
        backup.push(".bak");
        let _ = std::fs::copy(file, std::path::PathBuf::from(backup));
    }
    files::write_atomic(file, to_json(crib).as_bytes())
}

/// The file with `suffix` added to its name (`crib.json.bak`).
fn beside(file: &Path, suffix: &str) -> std::path::PathBuf {
    let mut name = file.as_os_str().to_owned();
    name.push(suffix);
    std::path::PathBuf::from(name)
}

/// Puts back the crib that `save` kept as `<file>.bak`, on request and never by itself. The backup must be a crib that reads; the file it replaces (damaged or not) is kept as
/// `<file>.damaged`, and the backup itself stays where it is. Returns how many tools the restored crib has.
pub fn restore_from_backup(file: &Path, max_mib: u64) -> Result<usize, String> {
    let backup = beside(file, ".bak");
    let crib = load_with_limit(&backup, max_mib)?;
    if file.is_file() {
        std::fs::copy(file, beside(file, ".damaged"))
            .map_err(|e| format!("{}: {e}", beside(file, ".damaged").display()))?;
    }
    files::write_atomic(file, to_json(&crib).as_bytes())?;
    Ok(crib.tools.len())
}

/// The crib of a file that is not larger than `max_mib` MiB (`max_file_mib` of `tools.cfg`).
pub fn load_with_limit(file: &Path, max_mib: u64) -> Result<Crib, String> {
    from_json(&files::read_text_limited(file, files::mib(max_mib))?)
}

/// The crib of a file, with the size limit that `tools.cfg` has by default.
pub fn load(file: &Path) -> Result<Crib, String> {
    load_with_limit(file, DEFAULT_MAX_FILE_MIB)
}
