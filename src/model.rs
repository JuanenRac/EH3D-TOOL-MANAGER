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
    /// The keys of the file that this program does not know (written by another tool or by a later version of the same schema): kept and written back as they were.
    pub extra: serde_json::Map<String, serde_json::Value>,
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
            extra: serde_json::Map::new(),
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
        if self.material.chars().count() > MAX_NAME || self.material.chars().any(|c| c.is_control())
        {
            return Err(format!(
                "the material must have at most {MAX_NAME} characters and no control character"
            ));
        }
        not_negative("hours", self.hours)?;
        not_negative("meters", self.meters)?;
        not_negative("max_hours", self.max_hours)
    }

    /// What is left before each limit that is set (`None` for a limit that is not set); a limit that was passed leaves nothing, never a negative number.
    pub fn remaining(&self) -> Remaining {
        let hours = if self.max_hours > 0.0 {
            Some((self.max_hours - self.hours).max(0.0))
        } else {
            None
        };
        let holes = if self.max_holes > 0 {
            Some(self.max_holes.saturating_sub(self.holes))
        } else {
            None
        };
        Remaining { hours, holes }
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

/// What is left of a tool before its limits (see `Tool::remaining`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Remaining {
    /// Hours before the limit of hours; `None` when the tool has no such limit.
    pub hours: Option<f64>,
    /// Holes before the limit of holes; `None` when the tool has no such limit.
    pub holes: Option<u64>,
}

/// The order of a list of tools (see `Crib::select`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    /// By number, the order of the crib.
    #[default]
    Id,
    /// By name, without regard to upper and lower case.
    Name,
    /// The most worn first (a tool to replace before one that is about to be, and one with no limit last).
    Wear,
}

impl Sort {
    /// `id`, `name` or `wear`; anything else is an error that names the three.
    pub fn parse(text: &str) -> Result<Sort, String> {
        match text.trim().to_ascii_lowercase().as_str() {
            "id" => Ok(Sort::Id),
            "name" => Ok(Sort::Name),
            "wear" => Ok(Sort::Wear),
            _ => Err(format!(
                "'{text}' is not a way to sort the tools (id, name or wear)"
            )),
        }
    }
}

/// Which tools a list shows and in what order (see `Crib::select`); the default shows every tool by number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Selection {
    /// Only the tools of this kind.
    pub kind: Option<Kind>,
    /// Only the tools that are worn or about to be.
    pub worn_only: bool,
    /// The order.
    pub sort: Sort,
}

/// What the tools of one kind add up to (see `Crib::summary`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KindSummary {
    /// The kind.
    pub kind: Kind,
    /// How many tools of the kind there are.
    pub tools: usize,
    /// Hours they ran in all.
    pub hours: f64,
    /// Metres they travelled in all.
    pub meters: f64,
    /// Holes they made in all (a sum that stops at the largest number instead of wrapping round).
    pub holes: u64,
    /// How many of them are worn or about to be.
    pub to_replace: usize,
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

impl Wear {
    /// The fraction of the limits that is used (1 or more when a limit was reached); a tool to replace counts as the largest number, to put it after the others.
    pub fn fraction(self) -> f64 {
        rank(&self)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Crib {
    pub tools: Vec<Tool>,
    /// The number the next tool gets at least: a number that was given is never given again, not even after its tool was removed. 0 until the crib has had a tool.
    pub next_id: u32,
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
        let used = self.tools.iter().map(|t| t.id).max().unwrap_or(0);
        tool.id = used
            .max(self.next_id.saturating_sub(1))
            .checked_add(1)
            .ok_or("too many tools: there is no number left for a new one")?;
        let id = tool.id;
        self.next_id = id.saturating_add(1);
        self.tools.push(tool);
        Ok(id)
    }

    /// Replaces the tool that has the number of `tool`: the same checks as `add`, and the number and the counter of numbers stay as they are.
    pub fn update(&mut self, tool: Tool) -> Result<(), String> {
        tool.validate()?;
        let at = self
            .tools
            .iter()
            .position(|t| t.id == tool.id)
            .ok_or_else(|| format!("there is no tool {}", tool.id))?;
        if self
            .tools
            .iter()
            .any(|t| t.id != tool.id && t.name.eq_ignore_ascii_case(&tool.name))
        {
            return Err(format!("there is a tool called '{}' already", tool.name));
        }
        self.tools[at] = tool;
        Ok(())
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
        let removed = self.tools.remove(at);
        // the number of a removed tool is not given to another one
        self.next_id = self.next_id.max(removed.id.saturating_add(1));
        Ok(removed)
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

    /// Puts a new tool in the place of the one that is worn: the counters go back to zero and everything else (the number, the name, the limits and the keys the program
    /// does not know) stays. It gives back what the counters said before.
    pub fn replace_tool(&mut self, id: u32) -> Result<(f64, f64, u64), String> {
        let tool = self
            .tools
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| format!("there is no tool {id}"))?;
        let before = (tool.hours, tool.meters, tool.holes);
        tool.hours = 0.0;
        tool.meters = 0.0;
        tool.holes = 0;
        Ok(before)
    }

    /// The tools that a list shows: only the kind and the worn ones when asked, in the order asked (ties go by number, so the order never changes between two runs).
    pub fn select(&self, selection: &Selection) -> Vec<&Tool> {
        let mut v: Vec<&Tool> = self
            .tools
            .iter()
            .filter(|t| selection.kind.is_none_or(|k| t.kind == k))
            .filter(|t| !selection.worn_only || !matches!(t.wear(), Wear::Good(_)))
            .collect();
        match selection.sort {
            Sort::Id => v.sort_by_key(|t| t.id),
            Sort::Name => v.sort_by_key(|t| (t.name.to_lowercase(), t.id)),
            Sort::Wear => v.sort_by(|a, b| {
                rank(&b.wear())
                    .partial_cmp(&rank(&a.wear()))
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.id.cmp(&b.id))
            }),
        }
        v
    }

    /// What the tools add up to, one row for each kind that has a tool (in the order of `KINDS`).
    pub fn summary(&self) -> Vec<KindSummary> {
        KINDS
            .into_iter()
            .filter_map(|kind| {
                let of_kind: Vec<&Tool> = self.tools.iter().filter(|t| t.kind == kind).collect();
                if of_kind.is_empty() {
                    return None;
                }
                Some(KindSummary {
                    kind,
                    tools: of_kind.len(),
                    hours: of_kind.iter().map(|t| t.hours).sum(),
                    meters: of_kind.iter().map(|t| t.meters).sum(),
                    holes: of_kind
                        .iter()
                        .fold(0u64, |sum, t| sum.saturating_add(t.holes)),
                    to_replace: of_kind
                        .iter()
                        .filter(|t| !matches!(t.wear(), Wear::Good(_)))
                        .count(),
                })
            })
            .collect()
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
