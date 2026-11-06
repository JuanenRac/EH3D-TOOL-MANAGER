//! Feeds and speeds to START from: a surface speed and a chip load per material, limited by what the spindle can turn. They are starting points to try on a scrap board, not a promise.
//! Every number that was chosen by hand (the surface speeds, the chip loads and the factors) is a field of `CalcParams`: its defaults are below and `tools.cfg` can change them.

use crate::model::{Kind, Tool};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    Fr4,
    Aluminium,
    Acrylic,
    Wood,
    Brass,
}

/// The materials, in the order of the arrays of `CalcParams`.
pub const MATERIALS: [Material; 5] = [
    Material::Fr4,
    Material::Aluminium,
    Material::Acrylic,
    Material::Wood,
    Material::Brass,
];

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

    pub fn key(self) -> &'static str {
        match self {
            Material::Fr4 => "fr4",
            Material::Aluminium => "aluminium",
            Material::Acrylic => "acrylic",
            Material::Wood => "wood",
            Material::Brass => "brass",
        }
    }

    /// The place of the material in the arrays of `CalcParams`.
    pub fn index(self) -> usize {
        match self {
            Material::Fr4 => 0,
            Material::Aluminium => 1,
            Material::Acrylic => 2,
            Material::Wood => 3,
            Material::Brass => 4,
        }
    }
}

/// The limits a value of `CalcParams` may have: (lowest, highest).
pub const SURFACE_SPEED_LIMITS: (f64, f64) = (1.0, 2000.0);
pub const CHIP_FRACTION_LIMITS: (f64, f64) = (0.0005, 0.5);
pub const WORKING_LIMITS: (f64, f64) = (0.01, 10.0);
pub const DRILL_PLUNGE_LIMITS: (f64, f64) = (0.0001, 1.0);
pub const PLUNGE_RATIO_LIMITS: (f64, f64) = (0.01, 1.0);
pub const PASS_FRACTION_LIMITS: (f64, f64) = (0.01, 2.0);
pub const MAX_PASS_LIMITS: (f64, f64) = (0.01, 25.0);
/// The included angle of a V-bit the width of the cut is worked out for, in degrees.
pub const V_BIT_ANGLE_LIMITS: (f64, f64) = (10.0, 170.0);
/// The width of the flat tip of a V-bit, in mm (0 is a sharp point).
pub const V_BIT_FLAT_LIMITS: (f64, f64) = (0.0, 5.0);
/// The deepest cut a width is worked out for, in mm.
pub const V_BIT_DEPTH_LIMITS: (f64, f64) = (0.0, 100.0);

/// The numbers the feeds and speeds are worked out with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalcParams {
    /// Surface speed of a carbide tool in metres a minute, per material (the order of `MATERIALS`).
    pub surface_speed: [f64; 5],
    /// Chip load as a fraction of the diameter of the tool, per tooth, per material.
    pub chip_fraction: [f64; 5],
    /// A V-bit or an engraver works at the diameter of its point: the diameter of the tool is kept between these two numbers (mm).
    pub working_min_mm: f64,
    pub working_max_mm: f64,
    /// The smallest diameter the chip load is multiplied by (mm).
    pub min_working_mm: f64,
    /// The plunge of a drill is its speed times this number times its diameter.
    pub drill_plunge: f64,
    /// The plunge of any other tool is its feed times this number.
    pub plunge_ratio: f64,
    /// The depth of a pass is the working diameter times this number, and at most `max_pass_mm`.
    pub pass_fraction: f64,
    pub max_pass_mm: f64,
    /// The included angle a V-bit is taken to have when the person gives none (the `vbit` command), in degrees.
    pub v_bit_angle_deg: f64,
    /// The width of the flat tip of a V-bit, in mm: the cut is this much wider than the cone alone makes it.
    pub v_bit_flat_mm: f64,
}

impl Default for CalcParams {
    fn default() -> Self {
        CalcParams {
            surface_speed: [100.0, 200.0, 150.0, 250.0, 120.0],
            chip_fraction: [0.015, 0.02, 0.04, 0.03, 0.02],
            working_min_mm: 0.1,
            working_max_mm: 0.5,
            min_working_mm: 0.1,
            drill_plunge: 0.01,
            plunge_ratio: 0.3,
            pass_fraction: 0.5,
            max_pass_mm: 1.0,
            v_bit_angle_deg: 60.0,
            v_bit_flat_mm: 0.1,
        }
    }
}

fn within(name: &str, v: f64, limits: (f64, f64)) -> Result<(), String> {
    if v.is_finite() && (limits.0..=limits.1).contains(&v) {
        Ok(())
    } else {
        Err(format!(
            "{name} is {v}: it must be from {} to {}",
            limits.0, limits.1
        ))
    }
}

impl CalcParams {
    pub fn surface_speed_of(&self, material: Material) -> f64 {
        self.surface_speed[material.index()]
    }

    pub fn chip_fraction_of(&self, material: Material) -> f64 {
        self.chip_fraction[material.index()]
    }

    /// Every number must make sense; the message names it.
    pub fn validate(&self) -> Result<(), String> {
        for m in MATERIALS {
            within(
                &format!("surface_speed_{}", m.key()),
                self.surface_speed_of(m),
                SURFACE_SPEED_LIMITS,
            )?;
            within(
                &format!("chip_fraction_{}", m.key()),
                self.chip_fraction_of(m),
                CHIP_FRACTION_LIMITS,
            )?;
        }
        within("working_min_mm", self.working_min_mm, WORKING_LIMITS)?;
        within("working_max_mm", self.working_max_mm, WORKING_LIMITS)?;
        if self.working_min_mm > self.working_max_mm {
            return Err(format!(
                "working_min_mm ({}) is more than working_max_mm ({})",
                self.working_min_mm, self.working_max_mm
            ));
        }
        within("min_working_mm", self.min_working_mm, WORKING_LIMITS)?;
        within("drill_plunge", self.drill_plunge, DRILL_PLUNGE_LIMITS)?;
        within("plunge_ratio", self.plunge_ratio, PLUNGE_RATIO_LIMITS)?;
        within("pass_fraction", self.pass_fraction, PASS_FRACTION_LIMITS)?;
        within("max_pass_mm", self.max_pass_mm, MAX_PASS_LIMITS)?;
        within("v_bit_angle_deg", self.v_bit_angle_deg, V_BIT_ANGLE_LIMITS)?;
        within("v_bit_flat_mm", self.v_bit_flat_mm, V_BIT_FLAT_LIMITS)
    }
}

/// The width of the cut of a V-bit at a depth: the cone opens by `2 * depth * tan(angle / 2)` and the flat tip adds its own width, so
/// `width = 2 * depth * tan(angle / 2) + flat`. The angle is the included angle of the bit (10 to 170 degrees), the depth is 0 or more (at most 100 mm) and the flat is from 0 to 5 mm.
pub fn v_bit_width_mm(depth_mm: f64, angle_deg: f64, flat_mm: f64) -> Result<f64, String> {
    within("depth_mm", depth_mm, V_BIT_DEPTH_LIMITS)?;
    within("angle_deg", angle_deg, V_BIT_ANGLE_LIMITS)?;
    within("flat_mm", flat_mm, V_BIT_FLAT_LIMITS)?;
    Ok(2.0 * depth_mm * (angle_deg.to_radians() / 2.0).tan() + flat_mm)
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

/// The numbers behind a cut, to show the formulas with their values and the conditions that applied.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trace {
    pub kind: Kind,
    pub diameter_mm: f64,
    pub flutes: u32,
    /// True for a V-bit or an engraver, which works at the diameter of its point.
    pub uses_point: bool,
    /// The diameter the speed is worked out with.
    pub working_mm: f64,
    pub surface_speed: f64,
    pub chip_fraction: f64,
    /// The speed the material asks for, before the spindle limits it.
    pub wanted_rpm: f64,
    pub max_rpm: f64,
    /// The speed after the limit, before it is rounded.
    pub rpm: f64,
    /// The diameter the chip load is multiplied by.
    pub chip_working_mm: f64,
    /// The feed before it is rounded.
    pub feed: f64,
    /// The depth of the cut a V-bit or an engraver was asked for, and the width that depth makes (see `v_bit_width_mm`); `None` when no depth was given.
    pub depth_mm: Option<f64>,
    pub v_width_mm: Option<f64>,
    /// The included angle of the tool, when it has one.
    pub tip_angle_deg: Option<f64>,
    pub params: CalcParams,
}

/// The cut for `tool` in `material` with a spindle of at most `max_rpm`, with the numbers of `CalcParams::default()`.
pub fn feeds_speeds(tool: &Tool, material: Material, max_rpm: f64) -> Result<Cut, String> {
    feeds_speeds_with(tool, material, max_rpm, &CalcParams::default())
}

/// The cut for `tool` in `material` with a spindle of at most `max_rpm`, worked out with `params`.
pub fn feeds_speeds_with(
    tool: &Tool,
    material: Material,
    max_rpm: f64,
    params: &CalcParams,
) -> Result<Cut, String> {
    feeds_speeds_traced(tool, material, max_rpm, params).map(|(cut, _)| cut)
}

/// The cut and the numbers behind it.
pub fn feeds_speeds_traced(
    tool: &Tool,
    material: Material,
    max_rpm: f64,
    params: &CalcParams,
) -> Result<(Cut, Trace), String> {
    feeds_speeds_traced_at(tool, material, max_rpm, params, None)
}

/// The cut and the numbers behind it, for a V-bit or an engraver that cuts to `depth_mm`: the diameter it works with is the width of the cut at that depth (see
/// `v_bit_width_mm`, with the angle of the tool and the flat of `params`), at most the diameter of the tool and not under 0.01 mm. With `None` the diameter of the point
/// is kept between `working_min_mm` and `working_max_mm`, as always. A depth for any other kind of tool is an error.
pub fn feeds_speeds_traced_at(
    tool: &Tool,
    material: Material,
    max_rpm: f64,
    params: &CalcParams,
    depth_mm: Option<f64>,
) -> Result<(Cut, Trace), String> {
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
    params.validate()?;
    // a V-bit or an engraver cuts at its widest only at the full depth: use the diameter at the point it works from
    let uses_point = matches!(tool.kind, Kind::VBit | Kind::Engraver);
    if depth_mm.is_some() && !uses_point {
        return Err(format!(
            "a cut depth is for a V-bit or an engraver, not for a {}",
            tool.kind.key()
        ));
    }
    let v_width = match (uses_point, depth_mm, tool.tip_angle_deg) {
        (true, Some(d), Some(a)) => Some(v_bit_width_mm(d, a, params.v_bit_flat_mm)?),
        _ => None,
    };
    let working = if let Some(w) = v_width {
        w.min(tool.diameter_mm).max(WORKING_LIMITS.0)
    } else if uses_point {
        tool.diameter_mm
            .clamp(params.working_min_mm, params.working_max_mm)
    } else {
        tool.diameter_mm
    };
    let surface_speed = params.surface_speed_of(material);
    let chip_fraction = params.chip_fraction_of(material);
    let wanted = surface_speed * 1000.0 / (std::f64::consts::PI * working);
    let rpm = wanted.min(max_rpm);
    // the chip load and the depth of a pass follow the diameter the tool works with, not the shank of a V-bit or an engraver
    let chip_working = working.max(params.min_working_mm);
    let feed = rpm * f64::from(tool.flutes) * chip_fraction * chip_working;
    let plunge = if tool.kind == Kind::Drill {
        rpm * params.drill_plunge * tool.diameter_mm
    } else {
        feed * params.plunge_ratio
    };
    let pass = if tool.kind == Kind::Drill {
        0.0
    } else {
        (working * params.pass_fraction).min(params.max_pass_mm)
    };
    let cut = Cut {
        rpm: rpm.round(),
        feed_mm_min: feed.round(),
        plunge_mm_min: plunge.round(),
        pass_depth_mm: pass,
        rpm_limited: wanted > max_rpm,
    };
    let trace = Trace {
        kind: tool.kind,
        diameter_mm: tool.diameter_mm,
        flutes: tool.flutes,
        uses_point,
        working_mm: working,
        surface_speed,
        chip_fraction,
        wanted_rpm: wanted,
        max_rpm,
        rpm,
        chip_working_mm: chip_working,
        feed,
        depth_mm,
        v_width_mm: v_width,
        tip_angle_deg: tool.tip_angle_deg,
        params: *params,
    };
    Ok((cut, trace))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_materials_are_in_the_order_of_the_arrays_and_read_back_by_their_key() {
        for (k, m) in MATERIALS.into_iter().enumerate() {
            assert_eq!(m.index(), k);
            assert_eq!(Material::parse(m.key()), Ok(m));
        }
    }

    #[test]
    fn the_defaults_give_the_numbers_the_program_always_gave() {
        let d = Tool::new("d", Kind::Drill, 0.8);
        let c = feeds_speeds(&d, Material::Fr4, 24000.0).unwrap();
        assert_eq!(
            (c.rpm, c.feed_mm_min, c.plunge_mm_min, c.pass_depth_mm),
            (24000.0, 576.0, 192.0, 0.0)
        );
        assert!(c.rpm_limited);
        let mut v = Tool::new("v", Kind::VBit, 3.175);
        v.tip_angle_deg = Some(30.0);
        let c = feeds_speeds(&v, Material::Aluminium, 24000.0).unwrap();
        assert_eq!(
            (c.rpm, c.feed_mm_min, c.plunge_mm_min, c.pass_depth_mm),
            (24000.0, 480.0, 144.0, 0.25)
        );
    }

    #[test]
    fn the_trace_holds_the_numbers_behind_the_cut() {
        let d = Tool::new("d", Kind::EndMill, 3.0);
        let (cut, trace) =
            feeds_speeds_traced(&d, Material::Wood, 100_000.0, &CalcParams::default()).unwrap();
        assert!(!trace.uses_point && !cut.rpm_limited);
        assert_eq!((trace.working_mm, trace.surface_speed), (3.0, 250.0));
        assert_eq!(trace.rpm.round(), cut.rpm);
        assert_eq!(trace.feed.round(), cut.feed_mm_min);
        assert_eq!(trace.params, CalcParams::default());
    }

    #[test]
    fn the_numbers_can_be_changed_and_a_bad_one_is_refused() {
        let d = Tool::new("d", Kind::Drill, 1.0);
        let mut p = CalcParams::default();
        p.surface_speed[Material::Wood.index()] = 100.0;
        let slow = feeds_speeds_with(&d, Material::Wood, 100_000.0, &p).unwrap();
        let normal =
            feeds_speeds_with(&d, Material::Wood, 100_000.0, &CalcParams::default()).unwrap();
        assert!(slow.rpm < normal.rpm);
        p.max_pass_mm = 0.0;
        let e = feeds_speeds_with(&d, Material::Wood, 100_000.0, &p).unwrap_err();
        assert!(e.contains("max_pass_mm"), "{e}");
        let q = CalcParams {
            working_min_mm: 2.0,
            ..CalcParams::default()
        };
        assert!(q.validate().unwrap_err().contains("working_min_mm"));
        let r = CalcParams {
            surface_speed: [f64::NAN; 5],
            ..CalcParams::default()
        };
        assert!(r.validate().unwrap_err().contains("surface_speed_fr4"));
    }

    #[test]
    fn the_width_of_a_v_bit_is_twice_the_depth_times_the_tangent_of_half_the_angle_plus_the_flat() {
        // 60 degrees at 0.5 mm: 2 x 0.5 x tan 30 = 0.57735, and the flat of 0.1 mm makes 0.67735
        let w = v_bit_width_mm(0.5, 60.0, 0.1).unwrap();
        assert!((w - 0.677_350).abs() < 1e-5, "{w}");
        // 90 degrees: the width is twice the depth; a sharp point adds nothing; no depth leaves only the flat
        assert!((v_bit_width_mm(1.0, 90.0, 0.0).unwrap() - 2.0).abs() < 1e-12);
        assert_eq!(v_bit_width_mm(0.0, 60.0, 0.1).unwrap(), 0.1);
        // 120 degrees at 0.2 mm: 2 x 0.2 x tan 60 = 0.69282
        assert!((v_bit_width_mm(0.2, 120.0, 0.0).unwrap() - 0.692_820).abs() < 1e-5);
        // the defaults of tools.cfg are the ones of the example in the documents
        let d = CalcParams::default();
        assert_eq!((d.v_bit_angle_deg, d.v_bit_flat_mm), (60.0, 0.1));
    }

    #[test]
    fn the_width_of_a_v_bit_refuses_what_makes_no_sense_and_names_it() {
        for (depth, angle, flat, name) in [
            (-0.1, 60.0, 0.1, "depth_mm"),
            (100.1, 60.0, 0.1, "depth_mm"),
            (f64::NAN, 60.0, 0.1, "depth_mm"),
            (0.5, 9.9, 0.1, "angle_deg"),
            (0.5, 170.1, 0.1, "angle_deg"),
            (0.5, f64::INFINITY, 0.1, "angle_deg"),
            (0.5, 60.0, -0.01, "flat_mm"),
            (0.5, 60.0, 5.1, "flat_mm"),
        ] {
            let e = v_bit_width_mm(depth, angle, flat).unwrap_err();
            assert!(e.contains(name), "{e}");
        }
        assert!(v_bit_width_mm(0.0, 10.0, 0.0).is_ok());
        assert!(v_bit_width_mm(100.0, 170.0, 5.0).is_ok());
        let bad = CalcParams {
            v_bit_angle_deg: 5.0,
            ..CalcParams::default()
        };
        assert!(bad.validate().unwrap_err().contains("v_bit_angle_deg"));
        let bad = CalcParams {
            v_bit_flat_mm: 9.0,
            ..CalcParams::default()
        };
        assert!(bad.validate().unwrap_err().contains("v_bit_flat_mm"));
    }

    #[test]
    fn a_v_bit_with_a_depth_works_at_the_width_of_the_cut_and_without_one_as_always() {
        let mut v = Tool::new("v", Kind::VBit, 3.175);
        v.tip_angle_deg = Some(60.0);
        let p = CalcParams::default();
        // no depth: the numbers of always (the point is kept between 0.1 and 0.5 mm, so 0.5 mm)
        let (_, plain) = feeds_speeds_traced(&v, Material::Aluminium, 100_000.0, &p).unwrap();
        assert_eq!(
            (plain.working_mm, plain.v_width_mm, plain.depth_mm),
            (0.5, None, None)
        );
        // 0.5 mm deep: the width 0.67735 mm is the diameter it works with
        let (cut, t) =
            feeds_speeds_traced_at(&v, Material::Aluminium, 100_000.0, &p, Some(0.5)).unwrap();
        assert!((t.working_mm - 0.677_350).abs() < 1e-5, "{}", t.working_mm);
        assert_eq!((t.depth_mm, t.tip_angle_deg), (Some(0.5), Some(60.0)));
        assert_eq!(t.v_width_mm, Some(t.working_mm));
        // speed = 200 x 1000 / (pi x 0.67735) = about 93987 rpm, which the spindle of 100000 rpm can turn
        assert!(!cut.rpm_limited);
        assert!((cut.rpm - 93_987.0).abs() <= 2.0, "{}", cut.rpm);
        // a deep cut cannot be wider than the bit: 3.175 mm at most
        let (_, deep) =
            feeds_speeds_traced_at(&v, Material::Aluminium, 100_000.0, &p, Some(20.0)).unwrap();
        assert_eq!(deep.working_mm, 3.175);
        // depth 0 and a sharp point would be a diameter of 0: the working diameter is never under 0.01 mm
        let sharp = CalcParams {
            v_bit_flat_mm: 0.0,
            ..CalcParams::default()
        };
        let (_, zero) =
            feeds_speeds_traced_at(&v, Material::Aluminium, 100_000.0, &sharp, Some(0.0)).unwrap();
        assert_eq!(zero.working_mm, 0.01);
        // a depth for a tool that is not a V-bit or an engraver is an error
        let d = Tool::new("d", Kind::Drill, 1.0);
        let e = feeds_speeds_traced_at(&d, Material::Fr4, 24000.0, &p, Some(0.5)).unwrap_err();
        assert!(e.contains("V-bit") && e.contains("drill"), "{e}");
        // an angle the width cannot be worked out for is refused with the name of the value
        v.tip_angle_deg = Some(5.0);
        let e = feeds_speeds_traced_at(&v, Material::Fr4, 24000.0, &p, Some(0.5)).unwrap_err();
        assert!(e.contains("angle_deg"), "{e}");
    }
}
