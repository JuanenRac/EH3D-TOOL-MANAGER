//! The check of the crib: that the tools and the sheets a job needs are in it, and that the crib agrees with itself. A job is described by a table of needs (one row for each tool or
//! sheet, by name and size) so that this program depends on no program that cuts. A check that is not passed is not an error of the program: the command ends with the code 2
//! (`VERIFICATION_FAILED`), apart from the code 1 of a file that cannot be read.

use crate::csv;
use crate::model::{Crib, Kind, Wear, MAX_NAME};
use crate::sheet::{KIND_KEY, MAX_KERF_MM};

/// The header of the table of needs.
pub const NEEDS_HEADER: &str = "name,kind,size_mm";
/// The code a check that is not passed starts its message with; the command line ends with the exit code 2 for it.
pub const VERIFICATION_FAILED: &str = "verification_failed";

/// What kind of thing a job needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedKind {
    /// A tool of any kind (the column `kind` is empty).
    AnyTool,
    /// A tool of this kind.
    Tool(Kind),
    /// A sheet.
    Sheet,
}

/// One row of the table of needs.
#[derive(Debug, Clone, PartialEq)]
pub struct Need {
    /// The row of the table (the header is row 1).
    pub row: usize,
    pub name: String,
    pub kind: NeedKind,
    /// The diameter of a tool or the thickness of a sheet; `None` for any.
    pub size_mm: Option<f64>,
}

/// Reads a table of needs (`name,kind,size_mm`; `kind` is a kind of tool or `sheet`, empty for any tool, and `size_mm` may be empty); an error names the row. At most `max_rows`
/// rows are read.
pub fn parse_needs(text: &str, max_rows: usize) -> Result<Vec<Need>, String> {
    let (head, rows) = csv::table(text, max_rows)?;
    if head.join(",") != NEEDS_HEADER {
        return Err(format!("the first row must be exactly: {NEEDS_HEADER}"));
    }
    let mut needs = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let n = i + 2;
        if row.len() != 3 {
            return Err(format!("row {n}: it has {} fields, not 3", row.len()));
        }
        let name = row[0].trim().to_string();
        if name.is_empty()
            || name.chars().count() > MAX_NAME
            || name.chars().any(|c| c.is_control())
        {
            return Err(format!(
                "row {n}: the name must have 1 to {MAX_NAME} characters and no control character"
            ));
        }
        let kind = match row[1].trim() {
            "" => NeedKind::AnyTool,
            k if k.eq_ignore_ascii_case(KIND_KEY) => NeedKind::Sheet,
            k => NeedKind::Tool(Kind::parse(k).map_err(|e| format!("row {n}: {e}"))?),
        };
        let size_mm = if row[2].trim().is_empty() {
            None
        } else {
            Some(
                row[2]
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|v| v.is_finite() && *v > 0.0)
                    .ok_or_else(|| {
                        format!(
                            "row {n}: size_mm '{}' must be a number above 0 or empty",
                            row[2]
                        )
                    })?,
            )
        };
        needs.push(Need {
            row: n,
            name,
            kind,
            size_mm,
        });
    }
    Ok(needs)
}

/// What a check found.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Report {
    /// Tools and sheets of the crib that were looked at.
    pub looked_at: usize,
    /// Rows of needs that were looked at.
    pub needs: usize,
    /// What makes the check not pass.
    pub problems: Vec<String>,
    /// What deserves a look and does not stop the check.
    pub warnings: Vec<String>,
}

impl Report {
    /// True when there is no problem.
    pub fn passed(&self) -> bool {
        self.problems.is_empty()
    }

    /// The report as lines of text, the last one saying `CHECK=PASS` or `CHECK=FAIL`.
    pub fn text(&self) -> String {
        let mut out = String::new();
        for w in &self.warnings {
            out.push_str(&format!("warning: {w}\n"));
        }
        for p in &self.problems {
            out.push_str(&format!("problem: {p}\n"));
        }
        out.push_str(&format!(
            "{} tools and sheets and {} needs looked at, {} problems, {} warnings\nCHECK={}\n",
            self.looked_at,
            self.needs,
            self.problems.len(),
            self.warnings.len(),
            if self.passed() { "PASS" } else { "FAIL" }
        ));
        out
    }
}

/// Checks the crib against itself and against the needs of a job. `tolerance_mm` is how far a size may be from the one asked (`check_tolerance_mm` of `tools.cfg`).
pub fn check(crib: &Crib, needs: &[Need], tolerance_mm: f64) -> Report {
    let mut r = Report {
        looked_at: crib.tools.len() + crib.sheets.len(),
        needs: needs.len(),
        ..Report::default()
    };
    for (i, t) in crib.tools.iter().enumerate() {
        if let Err(e) = t.validate() {
            r.problems.push(format!("tool {} '{}': {e}", t.id, t.name));
        }
        if crib.tools[..i]
            .iter()
            .any(|o| o.id == t.id || o.name.eq_ignore_ascii_case(&t.name))
        {
            r.problems.push(format!(
                "tool {} '{}': its number or its name is used twice",
                t.id, t.name
            ));
        }
    }
    let has_laser = crib
        .tools
        .iter()
        .any(|t| t.kind == Kind::Laser && t.diameter_mm > 0.0 && t.diameter_mm <= MAX_KERF_MM);
    for (i, s) in crib.sheets.iter().enumerate() {
        if let Err(e) = s.validate() {
            r.problems.push(format!("sheet {} '{}': {e}", s.id, s.name));
        }
        if crib.sheets[..i]
            .iter()
            .any(|o| o.id == s.id || o.name.eq_ignore_ascii_case(&s.name))
        {
            r.problems.push(format!(
                "sheet {} '{}': its number or its name is used twice",
                s.id, s.name
            ));
        }
        if s.kerf_mm == 0.0 && !has_laser {
            r.problems.push(format!(
                "sheet {} '{}': it has no kerf_mm and the crib has no laser tool of up to {MAX_KERF_MM} mm to take the kerf from",
                s.id, s.name
            ));
        }
    }
    for need in needs {
        let at = format!("need on row {}", need.row);
        match need.kind {
            NeedKind::Sheet => {
                let found: Vec<_> = crib
                    .sheets
                    .iter()
                    .filter(|s| s.name.eq_ignore_ascii_case(&need.name))
                    .collect();
                let Some(s) = found.first() else {
                    r.problems
                        .push(format!("{at}: there is no sheet called '{}'", need.name));
                    continue;
                };
                if let Some(size) = need.size_mm {
                    if (s.thickness_mm - size).abs() > tolerance_mm {
                        r.problems.push(format!(
                            "{at}: the sheet '{}' is {} mm thick, not {size} mm",
                            s.name, s.thickness_mm
                        ));
                    }
                }
            }
            NeedKind::AnyTool | NeedKind::Tool(_) => {
                let Some(t) = crib
                    .tools
                    .iter()
                    .find(|t| t.name.eq_ignore_ascii_case(&need.name))
                else {
                    let hint = if crib
                        .sheets
                        .iter()
                        .any(|s| s.name.eq_ignore_ascii_case(&need.name))
                    {
                        " (there is a sheet with that name: write the kind sheet)"
                    } else {
                        ""
                    };
                    r.problems.push(format!(
                        "{at}: there is no tool called '{}'{hint}",
                        need.name
                    ));
                    continue;
                };
                if let NeedKind::Tool(k) = need.kind {
                    if t.kind != k {
                        r.problems.push(format!(
                            "{at}: the tool '{}' is a {}, not a {}",
                            t.name,
                            t.kind.key(),
                            k.key()
                        ));
                    }
                }
                if let Some(size) = need.size_mm {
                    if (t.diameter_mm - size).abs() > tolerance_mm {
                        r.problems.push(format!(
                            "{at}: the tool '{}' has a diameter of {} mm, not {size} mm",
                            t.name, t.diameter_mm
                        ));
                    }
                }
                if t.wear() == Wear::Replace {
                    r.warnings.push(format!(
                        "{at}: the tool {} '{}' reached a wear limit and is to be replaced",
                        t.id, t.name
                    ));
                }
            }
        }
    }
    r
}
