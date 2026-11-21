//! A sheet of the material a stencil is cut from: what the program that cuts a stencil needs to know about the sheet, kept in the crib beside the tools. A sheet has a material, a
//! thickness, how much every aperture is made smaller than the pad it prints, the kerf of its cutting beam (or 0 to take the kerf from a laser tool of the crib) and the least
//! aspect ratio and area ratio an aperture may have in it (0 for no limit). No figure here is a default of the program: a sheet holds what the person wrote, and a limit that is 0 is
//! not applied.

use crate::model::MAX_NAME;

/// The word the table uses for the kind of a sheet.
pub const KIND_KEY: &str = "sheet";
/// The thickest sheet a crib may hold, in mm.
pub const MAX_THICKNESS_MM: f64 = 50.0;
/// The most an aperture may be made smaller on each side, in mm.
pub const MAX_REDUCTION_MM: f64 = 0.5;
/// The widest kerf a sheet may have, in mm (the largest a stencil program takes).
pub const MAX_KERF_MM: f64 = 1.0;
/// The largest aspect ratio or area ratio a limit may be.
pub const MAX_RATIO: f64 = 10.0;

/// A sheet of stencil material.
#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    /// 0 until the crib gives the sheet its number (the numbers of sheets are their own, apart from those of tools).
    pub id: u32,
    pub name: String,
    pub material: String,
    pub thickness_mm: f64,
    /// How much smaller than the pad each aperture is made, on each side, in mm; 0 is not smaller.
    pub reduction_mm: f64,
    /// The width of the cut of the beam in this sheet, in mm; 0 takes the kerf from a laser tool of the crib.
    pub kerf_mm: f64,
    /// The least aspect ratio (the narrow side of the aperture over the thickness) an aperture may have; 0 is no limit.
    pub min_aspect_ratio: f64,
    /// The least area ratio (the area of the opening over the area of its walls) an aperture may have; 0 is no limit.
    pub min_area_ratio: f64,
    /// The keys of the file that this program does not know: kept and written back as they were.
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Sheet {
    /// A sheet with no reduction, no kerf of its own and no limit of ratios.
    pub fn new(name: &str, material: &str, thickness_mm: f64) -> Sheet {
        Sheet {
            id: 0,
            name: name.trim().to_string(),
            material: material.trim().to_string(),
            thickness_mm,
            reduction_mm: 0.0,
            kerf_mm: 0.0,
            min_aspect_ratio: 0.0,
            min_area_ratio: 0.0,
            extra: serde_json::Map::new(),
        }
    }

    /// Every value must make sense; the message names the field.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty()
            || self.name.chars().count() > MAX_NAME
            || self.name.chars().any(|c| c.is_control())
        {
            return Err(format!(
                "the name must have 1 to {MAX_NAME} characters and no control character"
            ));
        }
        if self.material.chars().count() > MAX_NAME || self.material.chars().any(|c| c.is_control())
        {
            return Err(format!(
                "the material must have at most {MAX_NAME} characters and no control character"
            ));
        }
        let within = |name: &str, v: f64, low_open: bool, max: f64| -> Result<(), String> {
            let ok = v.is_finite() && v <= max && if low_open { v > 0.0 } else { v >= 0.0 };
            if ok {
                Ok(())
            } else if low_open {
                Err(format!(
                    "{name} is {v}: it must be more than 0 and at most {max}"
                ))
            } else {
                Err(format!("{name} is {v}: it must be from 0 to {max}"))
            }
        };
        within("thickness_mm", self.thickness_mm, true, MAX_THICKNESS_MM)?;
        within("reduction_mm", self.reduction_mm, false, MAX_REDUCTION_MM)?;
        within("kerf_mm", self.kerf_mm, false, MAX_KERF_MM)?;
        within("min_aspect_ratio", self.min_aspect_ratio, false, MAX_RATIO)?;
        within("min_area_ratio", self.min_area_ratio, false, MAX_RATIO)
    }

    /// The aspect ratio and the area ratio of a rectangular aperture of `width_mm` by `length_mm` in this sheet: the narrow side over the thickness, and the area of the opening
    /// over the area of its walls, `w * l / (2 * (w + l) * t)`.
    pub fn ratios(&self, width_mm: f64, length_mm: f64) -> Result<(f64, f64), String> {
        for (name, v) in [("width", width_mm), ("length", length_mm)] {
            if !v.is_finite() || v <= 0.0 {
                return Err(format!(
                    "the {name} of the aperture is {v}: it must be more than 0"
                ));
            }
        }
        let t = self.thickness_mm;
        Ok((
            width_mm.min(length_mm) / t,
            width_mm * length_mm / (2.0 * (width_mm + length_mm) * t),
        ))
    }

    /// The size of the aperture after the reduction, on each side; an aperture that the reduction would leave with no size is an error.
    pub fn reduced(&self, width_mm: f64, length_mm: f64) -> Result<(f64, f64), String> {
        self.ratios(width_mm, length_mm)?;
        let (w, l) = (
            width_mm - 2.0 * self.reduction_mm,
            length_mm - 2.0 * self.reduction_mm,
        );
        if w <= 0.0 || l <= 0.0 {
            return Err(format!(
                "an aperture of {width_mm} x {length_mm} mm is left with no size by a reduction of {} mm on each side",
                self.reduction_mm
            ));
        }
        Ok((w, l))
    }

    /// Whether an aperture of `width_mm` by `length_mm` (the size it will have after the reduction) can be cut in this sheet: a limit that is 0 is not applied, and the message
    /// names the limit that is not met.
    pub fn check_aperture(&self, width_mm: f64, length_mm: f64) -> Result<(), String> {
        let (aspect, area) = self.ratios(width_mm, length_mm)?;
        if self.min_aspect_ratio > 0.0 && aspect < self.min_aspect_ratio {
            return Err(format!(
                "the aspect ratio of {width_mm} x {length_mm} mm is {aspect:.3}, below the {} that the sheet '{}' asks for",
                self.min_aspect_ratio, self.name
            ));
        }
        if self.min_area_ratio > 0.0 && area < self.min_area_ratio {
            return Err(format!(
                "the area ratio of {width_mm} x {length_mm} mm is {area:.3}, below the {} that the sheet '{}' asks for",
                self.min_area_ratio, self.name
            ));
        }
        Ok(())
    }
}
