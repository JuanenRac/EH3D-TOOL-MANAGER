//! The cutting profile of a tool in a material, read from the machining database of EH3D-COMMON (`eh3d_materials::machining`). It is a reference that is only read: nothing from the
//! database is copied into the crib, and no figure is made up here. Every figure of the database is a starting point that nobody has tried on a machine unless the point says so, and the
//! text of a profile says so in its first line. This module is built only with the feature `common-machining`.

use crate::model::{Kind, Tool};
use eh3d_materials::machining::{CncOp, MachiningDatabase, Tool as CutterKind};

/// The first line of the text of a profile whose figures were not tried on a machine.
pub const NOT_VERIFIED: &str = "NOT VERIFIED: these figures are starting points of a table; nobody has tried them on a machine. Try them on a scrap piece.";

/// What the database says a tool can start from in a material.
#[derive(Debug, Clone, PartialEq)]
pub struct CutProfile {
    pub material_id: String,
    /// What the router does (`contour`, `engrave` or `drill`).
    pub operation: &'static str,
    pub rpm: f64,
    pub feed_mm_min: f64,
    pub plunge_mm_min: Option<f64>,
    pub depth_per_pass_mm: f64,
    /// True when the figures lie on a line between two points of the table.
    pub interpolated: bool,
    /// True only when every point the figures come from was tried on a machine.
    pub verified: bool,
}

/// The database that comes with EH3D-COMMON.
pub fn database() -> Result<MachiningDatabase, String> {
    MachiningDatabase::embedded().map_err(|e| e.to_string())
}

/// The identifiers of the materials of the database, in its order.
pub fn material_ids(db: &MachiningDatabase) -> Vec<String> {
    db.materials().iter().map(|w| w.id.clone()).collect()
}

/// The operation and the kind of cutter of the database that a tool of the crib is: a drill drills, an end mill cuts out and a V-bit or an engraver engraves. A laser and a nozzle
/// are not cutters of this table.
fn cutter(tool: &Tool) -> Result<(CncOp, CutterKind, &'static str), String> {
    match tool.kind {
        Kind::Drill => Ok((CncOp::Drill, CutterKind::Drill, "drill")),
        Kind::EndMill => Ok((CncOp::Contour, CutterKind::EndMill, "contour")),
        Kind::VBit | Kind::Engraver => Ok((CncOp::Engrave, CutterKind::VBit, "engrave")),
        Kind::Laser | Kind::Nozzle => Err(format!(
            "the tool '{}' is a {}: the machining database has starting points for a router, not for this kind of tool",
            tool.name,
            tool.kind.key()
        )),
    }
}

/// The profile of `tool` in the material called `material_id`. A material the database does not have, a diameter outside the diameters it knows and a kind of tool it has no points
/// for are errors that say so; the figures are never worked out outside what the table knows.
pub fn cut_profile(
    db: &MachiningDatabase,
    tool: &Tool,
    material_id: &str,
) -> Result<CutProfile, String> {
    let (op, kind, operation) = cutter(tool)?;
    let start = db
        .cnc_start(material_id, op, kind, tool.diameter_mm)
        .map_err(|e| format!("{material_id}: {e}"))?;
    Ok(CutProfile {
        material_id: material_id.to_string(),
        operation,
        rpm: start.rpm,
        feed_mm_min: start.feed_mm_min,
        plunge_mm_min: start.plunge_mm_min,
        depth_per_pass_mm: start.depth_per_pass_mm,
        interpolated: start.interpolated,
        verified: start.verified,
    })
}

/// The profile as lines of text for a person; a profile that was not tried on a machine starts with `NOT_VERIFIED`.
pub fn describe(tool: &Tool, p: &CutProfile) -> String {
    let mut out = String::new();
    if !p.verified {
        out.push_str(NOT_VERIFIED);
        out.push('\n');
    }
    out.push_str(&format!(
        "tool {} '{}' ({}, {} mm) in {}: {}\n  spindle  {} rpm\n  feed     {} mm/min\n",
        tool.id,
        tool.name,
        tool.kind.key(),
        tool.diameter_mm,
        p.material_id,
        p.operation,
        p.rpm,
        p.feed_mm_min
    ));
    if let Some(plunge) = p.plunge_mm_min {
        out.push_str(&format!("  plunge   {plunge} mm/min\n"));
    }
    out.push_str(&format!("  pass     {} mm\n", p.depth_per_pass_mm));
    if p.interpolated {
        out.push_str("  the figures lie on a line between two points of the table\n");
    }
    out
}
