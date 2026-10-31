//! Feeds and speeds to START from: a surface speed and a chip load per material, limited by what the spindle can turn. They are starting points to try on a scrap board, not a promise.

use crate::model::{Kind, Tool};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    Fr4,
    Aluminium,
    Acrylic,
    Wood,
    Brass,
}

impl Material {
    pub fn parse(text: &str) -> Result<Material, String> {
        match text.trim().to_ascii_lowercase().as_str() {
            "fr4" | "fr-4" => Ok(Material::Fr4),
            "aluminium" | "aluminum" => Ok(Material::Aluminium),
            "acrylic" => Ok(Material::Acrylic),
            "wood" => Ok(Material::Wood),
            "brass" => Ok(Material::Brass),
            other => Err(format!(
                "'{other}' is not a material (fr4, aluminium, acrylic, wood or brass)"
            )),
        }
    }

    /// Surface speed of a carbide tool in metres a minute.
    fn surface_speed(self) -> f64 {
        match self {
            Material::Fr4 => 100.0,
            Material::Aluminium => 200.0,
            Material::Acrylic => 150.0,
            Material::Wood => 250.0,
            Material::Brass => 120.0,
        }
    }

    /// Chip load as a fraction of the diameter of the tool, per tooth.
    fn chip_fraction(self) -> f64 {
        match self {
            Material::Fr4 => 0.015,
            Material::Aluminium => 0.02,
            Material::Acrylic => 0.04,
            Material::Wood => 0.03,
            Material::Brass => 0.02,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cut {
    pub rpm: f64,
    pub feed_mm_min: f64,
    pub plunge_mm_min: f64,
    /// The depth of one pass; a drill goes through in one.
    pub pass_depth_mm: f64,
    /// True when the spindle could not turn as fast as the material asks and the speed was lowered (the feed went down with it).
    pub rpm_limited: bool,
}

/// The cut for `tool` in `material` with a spindle of at most `max_rpm`.
pub fn feeds_speeds(tool: &Tool, material: Material, max_rpm: f64) -> Result<Cut, String> {
    tool.validate()?;
    if !max_rpm.is_finite() || max_rpm < 1000.0 {
        return Err(format!(
            "max_rpm is {max_rpm}: a spindle turns at least 1000 rpm"
        ));
    }
    if matches!(tool.kind, Kind::Laser | Kind::Nozzle) {
        return Err(format!(
            "a {} has no feeds and speeds of a cutting tool",
            tool.kind.key()
        ));
    }
    // a V-bit or an engraver cuts at its widest only at the full depth: use the diameter at the point it works from
    let working = if matches!(tool.kind, Kind::VBit | Kind::Engraver) {
        tool.diameter_mm.clamp(0.1, 0.5)
    } else {
        tool.diameter_mm
    };
    let wanted = material.surface_speed() * 1000.0 / (std::f64::consts::PI * working);
    let rpm = wanted.min(max_rpm);
    let feed = rpm * f64::from(tool.flutes) * material.chip_fraction() * tool.diameter_mm.max(0.1);
    let plunge = if tool.kind == Kind::Drill {
        rpm * 0.01 * tool.diameter_mm
    } else {
        feed * 0.3
    };
    let pass = if tool.kind == Kind::Drill {
        0.0
    } else {
        (tool.diameter_mm * 0.5).min(1.0)
    };
    Ok(Cut {
        rpm: rpm.round(),
        feed_mm_min: feed.round(),
        plunge_mm_min: plunge.round(),
        pass_depth_mm: pass,
        rpm_limited: wanted > max_rpm,
    })
}
