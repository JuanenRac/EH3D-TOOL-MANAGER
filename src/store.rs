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
    serde_json::to_string_pretty(&json!({"schema_version": SCHEMA_VERSION, "tools": tools}))
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

pub fn from_json(text: &str) -> Result<Crib, String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("the crib is not JSON: {e}"))?;
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
        t.id = item
            .get("id")
            .and_then(|n| n.as_u64())
            .and_then(|n| u32::try_from(n).ok())
            .unwrap_or(0);
        t.flutes =
            u32::try_from(item.get("flutes").and_then(|n| n.as_u64()).unwrap_or(2)).unwrap_or(0);
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
        t.holes = item.get("holes").and_then(|n| n.as_u64()).unwrap_or(0);
        t.max_hours = number(item, "max_hours", 0.0).map_err(fail)?;
        t.max_holes = item.get("max_holes").and_then(|n| n.as_u64()).unwrap_or(0);
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
    Ok(crib)
}

pub fn save(crib: &Crib, file: &Path) -> Result<(), String> {
    fs::write(file, to_json(crib)).map_err(|e| format!("{}: {e}", file.display()))
}

pub fn load(file: &Path) -> Result<Crib, String> {
    from_json(&fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?)
}
