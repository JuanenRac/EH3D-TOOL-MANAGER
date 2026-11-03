//! A tool, the crib that holds it and what the wear counters say.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Drill,
    EndMill,
    VBit,
    Engraver,
    Laser,
    Nozzle,
}

pub const KINDS: [Kind; 6] = [
    Kind::Drill,
    Kind::EndMill,
    Kind::VBit,
    Kind::Engraver,
    Kind::Laser,
    Kind::Nozzle,
];

impl Kind {
    pub fn key(self) -> &'static str {
        match self {
            Kind::Drill => "drill",
            Kind::EndMill => "end-mill",
            Kind::VBit => "v-bit",
            Kind::Engraver => "engraver",
            Kind::Laser => "laser",
            Kind::Nozzle => "nozzle",
        }
    }

    pub fn parse(text: &str) -> Result<Kind, String> {
        KINDS.into_iter().find(|k| k.key() == text.trim().to_ascii_lowercase()).ok_or_else(|| format!("'{text}' is not a kind of tool (drill, end-mill, v-bit, engraver, laser or nozzle)"))
    }
}

pub const MAX_DIAMETER_MM: f64 = 50.0;
pub const MAX_NAME: usize = 80;

#[derive(Debug, Clone, PartialEq)]
pub struct Tool {
    /// 0 until the crib gives the tool its number.
    pub id: u32,
    pub name: String,
    pub kind: Kind,
    pub diameter_mm: f64,
    pub flutes: u32,
    pub length_mm: f64,
    /// The included angle of a V-bit or an engraver; `None` for the other kinds.
    pub tip_angle_deg: Option<f64>,
    pub shank_mm: f64,
    pub material: String,
    pub hours: f64,
    pub meters: f64,
    pub holes: u64,
    /// The wear limits; 0 means no limit of that kind.
    pub max_hours: f64,
    pub max_holes: u64,
}

impl Tool {
    pub fn new(name: &str, kind: Kind, diameter_mm: f64) -> Tool {
        Tool {
            id: 0,
            name: name.trim().to_string(),
            kind,
            diameter_mm,
            flutes: 2,
            length_mm: 0.0,
            tip_angle_deg: None,
            shank_mm: 3.175,
            material: "carbide".into(),
            hours: 0.0,
            meters: 0.0,
            holes: 0,
            max_hours: 0.0,
            max_holes: 0,
        }
    }

    /// Every value must make sense; the message names the field.
    pub fn validate(&self) -> Result<(), String> {
        let positive = |name: &str, v: f64, max: f64| -> Result<(), String> {
            if v.is_finite() && v > 0.0 && v <= max {
                Ok(())
            } else {
                Err(format!(
                    "{name} is {v}: it must be more than 0 and at most {max}"
                ))
            }
        };
        let not_negative = |name: &str, v: f64| -> Result<(), String> {
            if v.is_finite() && v >= 0.0 {
                Ok(())
            } else {
                Err(format!(
                    "{name} is {v}: it cannot be negative or not a number"
                ))
            }
        };
        if self.name.is_empty()
            || self.name.chars().count() > MAX_NAME
            || self.name.chars().any(|c| c.is_control())
        {
            return Err(format!(
                "the name must have 1 to {MAX_NAME} characters and no control character"
            ));
        }
        positive("diameter_mm", self.diameter_mm, MAX_DIAMETER_MM)?;
        if !(1..=8).contains(&self.flutes) {
            return Err(format!("flutes is {}: it must be from 1 to 8", self.flutes));
        }
        not_negative("length_mm", self.length_mm)?;
        positive("shank_mm", self.shank_mm, 25.0)?;
        if let Some(a) = self.tip_angle_deg {
            positive("tip_angle_deg", a, 180.0)?;
        }
        if matches!(self.kind, Kind::VBit | Kind::Engraver) && self.tip_angle_deg.is_none() {
            return Err("a V-bit or an engraver needs its tip_angle_deg".into());
        }
        not_negative("hours", self.hours)?;
        not_negative("meters", self.meters)?;
        not_negative("max_hours", self.max_hours)
    }

    /// How worn the tool is: the largest of the fractions of its limits that are set.
    pub fn wear(&self) -> Wear {
        let mut fraction: f64 = 0.0;
        if self.max_hours > 0.0 {
            fraction = fraction.max(self.hours / self.max_hours);
        }
        if self.max_holes > 0 {
            fraction = fraction.max(self.holes as f64 / self.max_holes as f64);
        }
        if fraction >= 1.0 {
            Wear::Replace
        } else if fraction >= 0.9 {
            Wear::Soon(fraction)
        } else {
            Wear::Good(fraction)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Wear {
    /// Below 90 % of the limits (or no limit set); the fraction used.
    Good(f64),
    /// From 90 % of a limit: replace it soon.
    Soon(f64),
    /// A limit was reached: replace it.
    Replace,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Crib {
    pub tools: Vec<Tool>,
}

impl Crib {
    /// Adds a tool and returns the number the crib gives it (the smallest one above every number in use).
    pub fn add(&mut self, mut tool: Tool) -> Result<u32, String> {
        tool.validate()?;
        if self
            .tools
            .iter()
            .any(|t| t.name.eq_ignore_ascii_case(&tool.name))
        {
            return Err(format!("there is a tool called '{}' already", tool.name));
        }
        // a number that would not fit is an error, not a number that wraps round to 0 and makes the crib unreadable
        tool.id = self
            .tools
            .iter()
            .map(|t| t.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("too many tools: there is no number left for a new one")?;
        let id = tool.id;
        self.tools.push(tool);
        Ok(id)
    }

    pub fn get(&self, id: u32) -> Option<&Tool> {
        self.tools.iter().find(|t| t.id == id)
    }

    pub fn remove(&mut self, id: u32) -> Result<Tool, String> {
        let at = self
            .tools
            .iter()
            .position(|t| t.id == id)
            .ok_or_else(|| format!("there is no tool {id}"))?;
        Ok(self.tools.remove(at))
    }

    /// Adds what a job used to the counters of a tool.
    pub fn record_use(
        &mut self,
        id: u32,
        hours: f64,
        meters: f64,
        holes: u64,
    ) -> Result<Wear, String> {
        for (name, v) in [("hours", hours), ("meters", meters)] {
            if !v.is_finite() || v < 0.0 {
                return Err(format!(
                    "{name} is {v}: it cannot be negative or not a number"
                ));
            }
        }
        let tool = self
            .tools
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| format!("there is no tool {id}"))?;
        if !(tool.hours + hours).is_finite() || !(tool.meters + meters).is_finite() {
            return Err(format!(
                "tool {id}: the counters would grow past what a number can hold"
            ));
        }
        tool.hours += hours;
        tool.meters += meters;
        tool.holes = tool.holes.saturating_add(holes);
        Ok(tool.wear())
    }

    /// The tools that are worn or about to be, worst first.
    pub fn to_replace(&self) -> Vec<(&Tool, Wear)> {
        let mut v: Vec<(&Tool, Wear)> = self
            .tools
            .iter()
            .map(|t| (t, t.wear()))
            .filter(|(_, w)| !matches!(w, Wear::Good(_)))
            .collect();
        v.sort_by(|a, b| {
            rank(&b.1)
                .partial_cmp(&rank(&a.1))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        v
    }
}

fn rank(w: &Wear) -> f64 {
    match w {
        Wear::Good(f) | Wear::Soon(f) => *f,
        Wear::Replace => f64::MAX,
    }
}
