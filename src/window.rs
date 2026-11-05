//! The state of the window, without any screen: the crib, the form of a tool, the filters, the calculator and what was last said to the person. The screen (`gui`, built with the
//! `gui` feature) only draws this and passes on what the person does, so everything here is tested as plain code. Every check is the one the command line makes (the same functions
//! of the library) and an error is the message of the library.

use crate::calc::{feeds_speeds_traced, CalcParams, Cut, Material, Trace};
use crate::lang::{fill, Lang, Msg};
use crate::model::{Crib, Kind, Tool, Wear};
use crate::settings::Settings;
use crate::{csv, store};
use std::path::PathBuf;

/// The name of the crib file the window opens when `tools.cfg` names no other one, in the folder of the program.
pub const DEFAULT_CRIB_FILE: &str = "crib.json";
/// The name of the table the window offers for the CSV, in the folder of the program.
pub const DEFAULT_CSV_FILE: &str = "crib.csv";
/// More holes than this in one use is not a count of holes (the command line refuses it too).
pub const MAX_HOLES_PER_USE: u64 = 1_000_000_000_000;

/// A number as the person reads it: up to 2 decimals, 4 below 1, without zeros at the end.
pub fn fmt_num(v: f64) -> String {
    let decimals = if v.abs() < 1.0 { 4 } else { 2 };
    let text = format!("{v:.decimals$}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// The word of a kind of tool.
pub fn kind_msg(kind: Kind) -> Msg {
    match kind {
        Kind::Drill => Msg::KindDrill,
        Kind::EndMill => Msg::KindEndMill,
        Kind::VBit => Msg::KindVBit,
        Kind::Engraver => Msg::KindEngraver,
        Kind::Laser => Msg::KindLaser,
        Kind::Nozzle => Msg::KindNozzle,
    }
}

/// The word of a material.
pub fn material_msg(material: Material) -> Msg {
    match material {
        Material::Fr4 => Msg::MatFr4,
        Material::Aluminium => Msg::MatAluminium,
        Material::Acrylic => Msg::MatAcrylic,
        Material::Wood => Msg::MatWood,
        Material::Brass => Msg::MatBrass,
    }
}

/// How worn a tool is, in words.
pub fn wear_text(wear: Wear, lang: Lang) -> String {
    let percent = |f: f64| format!("{:.0}", f * 100.0);
    match wear {
        Wear::Good(f) => fill(Msg::WearGood.tr(lang), &[percent(f).as_str()]),
        Wear::Soon(f) => fill(Msg::WearSoonText.tr(lang), &[percent(f).as_str()]),
        Wear::Replace => Msg::WearReplaceText.tr(lang).to_string(),
    }
}

/// A finite number, or an error that names the field. A blank text is `blank` when there is a value for it, an error otherwise.
fn number(name: &str, text: &str, blank: Option<f64>) -> Result<f64, String> {
    let t = text.trim();
    if t.is_empty() {
        return blank.ok_or_else(|| format!("{name} is empty: a number is needed"));
    }
    t.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(|| format!("{name} '{text}' is not a number"))
}

/// A whole number of 0 or more, or an error that names the field.
fn whole(name: &str, text: &str, blank: Option<u64>) -> Result<u64, String> {
    let t = text.trim();
    if t.is_empty() {
        return blank.ok_or_else(|| format!("{name} is empty: a whole number is needed"));
    }
    t.parse::<u64>()
        .map_err(|_| format!("{name} '{text}' is not a whole number of 0 or more"))
}

/// The number of flutes written in a text.
fn flutes_of(text: &str) -> Result<u32, String> {
    u32::try_from(whole("flutes", text, None)?)
        .map_err(|_| format!("flutes '{text}' is too large, it must be from 1 to 8"))
}

/// The tip angle written in a text: a blank one is no angle.
fn angle_of(text: &str) -> Result<Option<f64>, String> {
    if text.trim().is_empty() {
        Ok(None)
    } else {
        number("tip_angle_deg", text, None).map(Some)
    }
}

/// The form of a tool, as texts: what the person typed, checked only when it is applied.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    /// 0 for a tool that is not in the crib yet; the number of the tool that is being edited otherwise.
    pub id: u32,
    pub name: String,
    pub kind: Kind,
    pub diameter_mm: String,
    pub flutes: String,
    pub length_mm: String,
    pub tip_angle_deg: String,
    pub shank_mm: String,
    pub material: String,
    pub hours: String,
    pub meters: String,
    pub holes: String,
    pub max_hours: String,
    pub max_holes: String,
}

impl Default for Draft {
    fn default() -> Self {
        Draft {
            id: 0,
            name: String::new(),
            kind: Kind::Drill,
            diameter_mm: String::new(),
            flutes: "2".into(),
            length_mm: "0".into(),
            tip_angle_deg: String::new(),
            shank_mm: "3.175".into(),
            material: "carbide".into(),
            hours: "0".into(),
            meters: "0".into(),
            holes: "0".into(),
            max_hours: "0".into(),
            max_holes: "0".into(),
        }
    }
}

impl Draft {
    /// The form filled with the values of a tool of the crib.
    pub fn from_tool(t: &Tool) -> Draft {
        Draft {
            id: t.id,
            name: t.name.clone(),
            kind: t.kind,
            diameter_mm: t.diameter_mm.to_string(),
            flutes: t.flutes.to_string(),
            length_mm: t.length_mm.to_string(),
            tip_angle_deg: t.tip_angle_deg.map(|a| a.to_string()).unwrap_or_default(),
            shank_mm: t.shank_mm.to_string(),
            material: t.material.clone(),
            hours: t.hours.to_string(),
            meters: t.meters.to_string(),
            holes: t.holes.to_string(),
            max_hours: t.max_hours.to_string(),
            max_holes: t.max_holes.to_string(),
        }
    }

    /// The tool the form describes, with every check of the library; the error names the field.
    pub fn to_tool(&self) -> Result<Tool, String> {
        let mut t = Tool::new(
            &self.name,
            self.kind,
            number("diameter_mm", &self.diameter_mm, None)?,
        );
        t.id = self.id;
        t.flutes = flutes_of(&self.flutes)?;
        t.length_mm = number("length_mm", &self.length_mm, Some(0.0))?;
        t.tip_angle_deg = angle_of(&self.tip_angle_deg)?;
        t.shank_mm = number("shank_mm", &self.shank_mm, Some(3.175))?;
        t.material = self.material.trim().to_string();
        t.hours = number("hours", &self.hours, Some(0.0))?;
        t.meters = number("meters", &self.meters, Some(0.0))?;
        t.holes = whole("holes", &self.holes, Some(0))?;
        t.max_hours = number("max_hours", &self.max_hours, Some(0.0))?;
        t.max_holes = whole("max_holes", &self.max_holes, Some(0))?;
        t.validate()?;
        Ok(t)
    }
}

/// Which tools the wear filter lets through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WearFilter {
    All,
    /// Worn, or from 90 % of a limit.
    Worn,
    /// A limit was reached.
    Replace,
}

impl WearFilter {
    pub const ALL: [WearFilter; 3] = [WearFilter::All, WearFilter::Worn, WearFilter::Replace];

    pub fn msg(self) -> Msg {
        match self {
            WearFilter::All => Msg::WearAll,
            WearFilter::Worn => Msg::WearWorn,
            WearFilter::Replace => Msg::WearReplaceOnly,
        }
    }
}

/// What the table is sorted by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Id,
    Name,
    Kind,
    Diameter,
    Wear,
}

impl SortKey {
    pub const ALL: [SortKey; 5] = [
        SortKey::Id,
        SortKey::Name,
        SortKey::Kind,
        SortKey::Diameter,
        SortKey::Wear,
    ];

    pub fn msg(self) -> Msg {
        match self {
            SortKey::Id => Msg::ColId,
            SortKey::Name => Msg::ColName,
            SortKey::Kind => Msg::KindLabel,
            SortKey::Diameter => Msg::ColDiameter,
            SortKey::Wear => Msg::WearLabel,
        }
    }
}

/// The two pages of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Tools,
    Calc,
}

/// What the window last said to the person.
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Ready,
    Opened {
        file: String,
        tools: usize,
    },
    Saved {
        file: String,
        tools: usize,
        backup: bool,
    },
    NewCrib,
    Added {
        name: String,
        id: u32,
    },
    Updated {
        name: String,
        id: u32,
    },
    Removed {
        name: String,
        id: u32,
    },
    Used {
        id: u32,
        wear: Wear,
    },
    Imported {
        file: String,
        tools: usize,
    },
    Exported {
        file: String,
        tools: usize,
    },
    /// The message of the library, as it is.
    Failed(String),
    BadSettings(String),
    NeedFile,
    NoSelection,
    Exists {
        file: String,
    },
}

impl Status {
    /// True for what went wrong (the window shows it in another colour).
    pub fn is_problem(&self) -> bool {
        matches!(
            self,
            Status::Failed(_)
                | Status::BadSettings(_)
                | Status::NeedFile
                | Status::NoSelection
                | Status::Exists { .. }
        )
    }

    /// The sentence, in the language of the person.
    pub fn text(&self, lang: Lang) -> String {
        let count = |v: usize| v.to_string();
        match self {
            Status::Ready => Msg::Ready.tr(lang).to_string(),
            Status::Opened { file, tools } => fill(
                Msg::StatusOpened.tr(lang),
                &[file.as_str(), count(*tools).as_str()],
            ),
            Status::Saved {
                file,
                tools,
                backup,
            } => {
                let msg = if *backup {
                    Msg::StatusSavedBackup
                } else {
                    Msg::StatusSaved
                };
                fill(msg.tr(lang), &[file.as_str(), count(*tools).as_str()])
            }
            Status::NewCrib => Msg::StatusNewCrib.tr(lang).to_string(),
            Status::Added { name, id } => fill(
                Msg::StatusAdded.tr(lang),
                &[name.as_str(), id.to_string().as_str()],
            ),
            Status::Updated { name, id } => fill(
                Msg::StatusUpdated.tr(lang),
                &[name.as_str(), id.to_string().as_str()],
            ),
            Status::Removed { name, id } => fill(
                Msg::StatusRemoved.tr(lang),
                &[name.as_str(), id.to_string().as_str()],
            ),
            Status::Used { id, wear } => fill(
                Msg::StatusUsed.tr(lang),
                &[id.to_string().as_str(), wear_text(*wear, lang).as_str()],
            ),
            Status::Imported { file, tools } => fill(
                Msg::StatusImported.tr(lang),
                &[file.as_str(), count(*tools).as_str()],
            ),
            Status::Exported { file, tools } => fill(
                Msg::StatusExported.tr(lang),
                &[file.as_str(), count(*tools).as_str()],
            ),
            Status::Failed(e) => fill(Msg::StatusFailed.tr(lang), &[e.as_str()]),
            Status::BadSettings(e) => fill(Msg::StatusBadSettings.tr(lang), &[e.as_str()]),
            Status::NeedFile => Msg::StatusNeedFile.tr(lang).to_string(),
            Status::NoSelection => Msg::StatusNoSelection.tr(lang).to_string(),
            Status::Exists { file } => fill(Msg::StatusExists.tr(lang), &[file.as_str()]),
        }
    }
}

/// What the calculator is asked: a tool as texts, the material and the spindle, and the numbers the cut is worked out with.
#[derive(Debug, Clone, PartialEq)]
pub struct CalcState {
    pub kind: Kind,
    pub diameter_mm: String,
    pub flutes: String,
    pub tip_angle_deg: String,
    pub material: Material,
    pub max_rpm: String,
    pub params: CalcParams,
}

/// A cut with the numbers behind it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Calculation {
    pub cut: Cut,
    pub trace: Trace,
    pub material: Material,
}

impl CalcState {
    /// The calculator as it starts: the drill of the example of the command line, and the numbers and the spindle of the settings.
    pub fn new(settings: &Settings) -> CalcState {
        CalcState {
            kind: Kind::Drill,
            diameter_mm: "0.8".into(),
            flutes: "2".into(),
            tip_angle_deg: String::new(),
            material: Material::Fr4,
            max_rpm: fmt_num(settings.default_max_rpm),
            params: settings.params,
        }
    }

    /// Takes the kind, the diameter, the flutes and the angle of a tool of the crib.
    pub fn take_tool(&mut self, t: &Tool) {
        self.kind = t.kind;
        self.diameter_mm = t.diameter_mm.to_string();
        self.flutes = t.flutes.to_string();
        self.tip_angle_deg = t.tip_angle_deg.map(|a| a.to_string()).unwrap_or_default();
    }

    /// The cut, or the reason there is none (the message of the library).
    pub fn compute(&self) -> Result<Calculation, String> {
        let mut t = Tool::new(
            "calc",
            self.kind,
            number("diameter_mm", &self.diameter_mm, None)?,
        );
        t.flutes = flutes_of(&self.flutes)?;
        t.tip_angle_deg = angle_of(&self.tip_angle_deg)?;
        let max_rpm = number("max_rpm", &self.max_rpm, None)?;
        let (cut, trace) = feeds_speeds_traced(&t, self.material, max_rpm, &self.params)?;
        Ok(Calculation {
            cut,
            trace,
            material: self.material,
        })
    }
}

impl Calculation {
    /// The formulas with their numbers and the conditions that applied, one line each, in the language of the person.
    pub fn explain(&self, lang: Lang) -> Vec<String> {
        let t = &self.trace;
        let p = &t.params;
        let n = fmt_num;
        let mut out = Vec::new();
        if t.uses_point {
            out.push(fill(
                Msg::FormPoint.tr(lang),
                &[
                    n(t.diameter_mm).as_str(),
                    n(p.working_min_mm).as_str(),
                    n(p.working_max_mm).as_str(),
                    n(t.working_mm).as_str(),
                ],
            ));
        }
        out.push(fill(
            Msg::FormRpm.tr(lang),
            &[
                n(t.surface_speed).as_str(),
                n(t.working_mm).as_str(),
                n(t.wanted_rpm).as_str(),
            ],
        ));
        let spindle = if self.cut.rpm_limited {
            Msg::CondSpindleLimit
        } else {
            Msg::CondSpindleFree
        };
        out.push(fill(
            spindle.tr(lang),
            &[n(t.max_rpm).as_str(), n(t.wanted_rpm).as_str()],
        ));
        out.push(fill(
            Msg::FormFeed.tr(lang),
            &[
                n(t.rpm).as_str(),
                t.flutes.to_string().as_str(),
                n(t.chip_fraction).as_str(),
                n(t.chip_working_mm).as_str(),
                n(self.cut.feed_mm_min).as_str(),
            ],
        ));
        if t.kind == Kind::Drill {
            out.push(fill(
                Msg::FormPlungeDrill.tr(lang),
                &[
                    n(p.drill_plunge).as_str(),
                    n(t.rpm).as_str(),
                    n(t.diameter_mm).as_str(),
                    n(self.cut.plunge_mm_min).as_str(),
                ],
            ));
            out.push(Msg::CondDrillPass.tr(lang).to_string());
        } else {
            out.push(fill(
                Msg::FormPlunge.tr(lang),
                &[
                    n(p.plunge_ratio).as_str(),
                    n(t.feed).as_str(),
                    n(self.cut.plunge_mm_min).as_str(),
                ],
            ));
            out.push(fill(
                Msg::FormPass.tr(lang),
                &[
                    n(p.pass_fraction).as_str(),
                    n(p.max_pass_mm).as_str(),
                    n(self.cut.pass_depth_mm).as_str(),
                ],
            ));
        }
        out
    }
}

/// The whole state of the window.
#[derive(Debug, Clone)]
pub struct Window {
    /// The folder of the program: `tools.cfg` and, by default, the crib are there.
    pub data_dir: PathBuf,
    pub settings: Settings,
    pub lang: Lang,
    pub tab: Tab,
    pub crib: Crib,
    /// The file of the crib and the table of the CSV, as the person wrote them.
    pub file_text: String,
    pub csv_text: String,
    /// Whether an export may replace a file that is there.
    pub overwrite_csv: bool,
    /// True when the crib has changes that are not saved.
    pub dirty: bool,
    /// True when the next save replaces a file whose tools were swapped for others (a new crib or an import): the file that is there is kept as `.bak`.
    backup_pending: bool,
    pub search: String,
    pub kind_filter: Option<Kind>,
    pub wear_filter: WearFilter,
    pub sort: SortKey,
    pub descending: bool,
    pub draft: Draft,
    pub selected: Option<u32>,
    pub use_hours: String,
    pub use_meters: String,
    pub use_holes: String,
    pub calc: CalcState,
    pub status: Status,
}

impl Window {
    /// The window for the program whose files are in `data_dir`: it reads `tools.cfg` there (a bad one gives the defaults and says why) and opens the crib file if it is there.
    pub fn new(data_dir: PathBuf) -> Window {
        let (settings, bad) = match Settings::load(&data_dir) {
            Ok(s) => (s, None),
            Err(e) => (Settings::default(), Some(e)),
        };
        let file = settings
            .crib_file
            .clone()
            .unwrap_or_else(|| data_dir.join(DEFAULT_CRIB_FILE));
        let mut w = Window {
            lang: settings.language.unwrap_or_else(Lang::from_env),
            tab: Tab::Tools,
            crib: Crib::default(),
            file_text: file.display().to_string(),
            csv_text: data_dir.join(DEFAULT_CSV_FILE).display().to_string(),
            overwrite_csv: false,
            dirty: false,
            backup_pending: false,
            search: String::new(),
            kind_filter: None,
            wear_filter: WearFilter::All,
            sort: SortKey::Id,
            descending: false,
            draft: Draft::default(),
            selected: None,
            use_hours: "0".into(),
            use_meters: "0".into(),
            use_holes: "0".into(),
            calc: CalcState::new(&settings),
            status: Status::Ready,
            data_dir,
            settings,
        };
        if file.is_file() {
            w.open();
        }
        if let Some(e) = bad {
            w.status = Status::BadSettings(e);
        }
        w
    }

    fn file_path(&self) -> Option<PathBuf> {
        let t = self.file_text.trim();
        if t.is_empty() {
            None
        } else {
            Some(PathBuf::from(t))
        }
    }

    fn csv_path(&self) -> Option<PathBuf> {
        let t = self.csv_text.trim();
        if t.is_empty() {
            None
        } else {
            Some(PathBuf::from(t))
        }
    }

    fn fail(&mut self, error: String) {
        self.status = Status::Failed(error);
    }

    /// The form and the choice start again (after another crib was loaded).
    fn forget_choice(&mut self) {
        self.draft = Draft::default();
        self.selected = None;
    }

    /// Reads the crib file.
    pub fn open(&mut self) {
        let Some(file) = self.file_path() else {
            self.status = Status::NeedFile;
            return;
        };
        match store::load(&file) {
            Ok(crib) => {
                self.crib = crib;
                self.forget_choice();
                self.dirty = false;
                self.backup_pending = false;
                self.status = Status::Opened {
                    file: file.display().to_string(),
                    tools: self.crib.tools.len(),
                };
            }
            Err(e) => self.fail(e),
        }
    }

    /// Writes the crib file (through a temporary file, as the command line does). A file that a new crib or an import replaces is kept beside it as `.bak`.
    pub fn save(&mut self) {
        let Some(file) = self.file_path() else {
            self.status = Status::NeedFile;
            return;
        };
        let backup = self.backup_pending && file.exists();
        if backup {
            let mut bak = file.as_os_str().to_owned();
            bak.push(".bak");
            if let Err(e) = std::fs::copy(&file, &bak) {
                self.fail(format!("{}.bak: {e}", file.display()));
                return;
            }
        }
        match store::save(&self.crib, &file) {
            Ok(()) => {
                self.dirty = false;
                self.backup_pending = false;
                self.status = Status::Saved {
                    file: file.display().to_string(),
                    tools: self.crib.tools.len(),
                    backup,
                };
            }
            Err(e) => self.fail(e),
        }
    }

    /// An empty crib; the file that is named is not touched until the person saves (and then it is kept as `.bak`).
    pub fn new_crib(&mut self) {
        self.crib = Crib::default();
        self.forget_choice();
        self.dirty = false;
        self.backup_pending = true;
        self.status = Status::NewCrib;
    }

    /// Replaces the crib with the tools of a CSV table, with the errors of the library. The numbers that were given are still not given again.
    pub fn import_csv(&mut self) {
        let Some(file) = self.csv_path() else {
            self.status = Status::NeedFile;
            return;
        };
        let text = match std::fs::read_to_string(&file) {
            Ok(t) => t,
            Err(e) => {
                self.fail(format!("{}: {e}", file.display()));
                return;
            }
        };
        match csv::from_csv(&text) {
            Ok(mut crib) => {
                crib.next_id = crib.next_id.max(self.crib.next_id);
                self.crib = crib;
                self.forget_choice();
                self.dirty = true;
                self.backup_pending = true;
                self.status = Status::Imported {
                    file: file.display().to_string(),
                    tools: self.crib.tools.len(),
                };
            }
            Err(e) => self.fail(e),
        }
    }

    /// Writes the crib as a CSV table; a file that is there is replaced only when the person said so.
    pub fn export_csv(&mut self) {
        let Some(file) = self.csv_path() else {
            self.status = Status::NeedFile;
            return;
        };
        if file.exists() && !self.overwrite_csv {
            self.status = Status::Exists {
                file: file.display().to_string(),
            };
            return;
        }
        match std::fs::write(&file, csv::to_csv(&self.crib)) {
            Ok(()) => {
                self.status = Status::Exported {
                    file: file.display().to_string(),
                    tools: self.crib.tools.len(),
                };
            }
            Err(e) => self.fail(format!("{}: {e}", file.display())),
        }
    }

    /// Adds the tool of the form to the crib, or changes the tool that the form is editing.
    pub fn apply_draft(&mut self) {
        let tool = match self.draft.to_tool() {
            Ok(t) => t,
            Err(e) => {
                self.fail(e);
                return;
            }
        };
        let name = tool.name.clone();
        if self.draft.id == 0 {
            match self.crib.add(tool) {
                Ok(id) => {
                    self.after_change(id);
                    self.status = Status::Added { name, id };
                }
                Err(e) => self.fail(e),
            }
        } else {
            let id = tool.id;
            match self.crib.update(tool) {
                Ok(()) => {
                    self.after_change(id);
                    self.status = Status::Updated { name, id };
                }
                Err(e) => self.fail(e),
            }
        }
    }

    fn after_change(&mut self, id: u32) {
        self.dirty = true;
        self.selected = Some(id);
        self.draft = Draft::default();
    }

    /// Puts a tool of the crib in the form.
    pub fn edit(&mut self, id: u32) {
        if let Some(draft) = self.crib.get(id).map(Draft::from_tool) {
            self.draft = draft;
            self.selected = Some(id);
        }
    }

    /// Empties the form (the tool being edited is left as it is).
    pub fn clear_form(&mut self) {
        self.draft = Draft::default();
    }

    /// Removes a tool from the crib; its number is not given to another one.
    pub fn remove(&mut self, id: u32) {
        match self.crib.remove(id) {
            Ok(t) => {
                self.dirty = true;
                if self.draft.id == id {
                    self.draft = Draft::default();
                }
                if self.selected == Some(id) {
                    self.selected = None;
                }
                self.status = Status::Removed { name: t.name, id };
            }
            Err(e) => self.fail(e),
        }
    }

    /// The tool the person chose in the table.
    pub fn selected_tool(&self) -> Option<&Tool> {
        self.selected.and_then(|id| self.crib.get(id))
    }

    fn parse_use(&self) -> Result<(f64, f64, u64), String> {
        let hours = number("hours", &self.use_hours, Some(0.0))?;
        let meters = number("meters", &self.use_meters, Some(0.0))?;
        let holes = whole("holes", &self.use_holes, Some(0))?;
        if holes > MAX_HOLES_PER_USE {
            return Err(format!("holes {holes} is not a count of holes"));
        }
        Ok((hours, meters, holes))
    }

    /// Adds what a job used to the counters of the chosen tool.
    pub fn record_use(&mut self) {
        let Some(id) = self.selected else {
            self.status = Status::NoSelection;
            return;
        };
        let (hours, meters, holes) = match self.parse_use() {
            Ok(v) => v,
            Err(e) => {
                self.fail(e);
                return;
            }
        };
        match self.crib.record_use(id, hours, meters, holes) {
            Ok(wear) => {
                self.dirty = true;
                if self.draft.id == id {
                    self.edit(id);
                }
                self.use_hours = "0".into();
                self.use_meters = "0".into();
                self.use_holes = "0".into();
                self.status = Status::Used { id, wear };
            }
            Err(e) => self.fail(e),
        }
    }

    fn passes(&self, t: &Tool, needle: &str) -> bool {
        if self.kind_filter.is_some_and(|k| t.kind != k) {
            return false;
        }
        let wear_ok = match self.wear_filter {
            WearFilter::All => true,
            WearFilter::Worn => !matches!(t.wear(), Wear::Good(_)),
            WearFilter::Replace => t.wear() == Wear::Replace,
        };
        wear_ok
            && (needle.is_empty()
                || t.name.to_lowercase().contains(needle)
                || t.material.to_lowercase().contains(needle)
                || t.kind.key().contains(needle)
                || t.id.to_string() == needle)
    }

    /// The places in `crib.tools` of the tools the filters let through, in the order of the table.
    pub fn visible(&self) -> Vec<usize> {
        let needle = self.search.trim().to_lowercase();
        let tools = &self.crib.tools;
        let mut rows: Vec<usize> = tools
            .iter()
            .enumerate()
            .filter(|(_, t)| self.passes(t, &needle))
            .map(|(i, _)| i)
            .collect();
        rows.sort_by(|&a, &b| {
            let (x, y) = (&tools[a], &tools[b]);
            let order = match self.sort {
                SortKey::Id => x.id.cmp(&y.id),
                SortKey::Name => x.name.to_lowercase().cmp(&y.name.to_lowercase()),
                SortKey::Kind => x.kind.key().cmp(y.kind.key()),
                SortKey::Diameter => x.diameter_mm.total_cmp(&y.diameter_mm),
                SortKey::Wear => x.wear().fraction().total_cmp(&y.wear().fraction()),
            };
            if self.descending {
                order.reverse()
            } else {
                order
            }
        });
        rows
    }

    /// Puts the kind, the diameter, the flutes and the angle of the chosen tool in the calculator.
    pub fn calc_from_selected(&mut self) {
        match self.selected.and_then(|id| self.crib.get(id)) {
            Some(t) => self.calc.take_tool(t),
            None => self.status = Status::NoSelection,
        }
    }

    /// The numbers of the calculation go back to the ones of `tools.cfg`.
    pub fn reset_params(&mut self) {
        self.calc.params = self.settings.params;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::feeds_speeds;

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("eh3d-tools-win-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn window(name: &str) -> Window {
        Window::new(temp(name))
    }

    fn add(w: &mut Window, name: &str, kind: Kind, diameter: &str) {
        w.draft.name = name.to_string();
        w.draft.kind = kind;
        w.draft.diameter_mm = diameter.to_string();
        if matches!(kind, Kind::VBit | Kind::Engraver) {
            w.draft.tip_angle_deg = "30".into();
        }
        w.apply_draft();
    }

    #[test]
    fn numbers_are_written_short() {
        assert_eq!(fmt_num(100.0), "100");
        assert_eq!(fmt_num(0.015), "0.015");
        assert_eq!(fmt_num(0.0005), "0.0005");
        assert_eq!(fmt_num(7957.7471), "7957.75");
        assert_eq!(fmt_num(0.0), "0");
        assert_eq!(fmt_num(24000.0), "24000");
    }

    #[test]
    fn the_form_makes_the_tool_the_library_would_accept_and_names_the_field_that_is_wrong() {
        let mut d = Draft {
            name: "  Drill 0.8 ".into(),
            diameter_mm: "0.8".into(),
            ..Draft::default()
        };
        let t = d.to_tool().unwrap();
        assert_eq!(t, Tool::new("Drill 0.8", Kind::Drill, 0.8));
        d.diameter_mm = "-1".into();
        assert!(d.to_tool().unwrap_err().contains("diameter_mm"));
        d.diameter_mm = "abc".into();
        assert!(d.to_tool().unwrap_err().contains("not a number"));
        d.diameter_mm = "".into();
        assert!(d.to_tool().unwrap_err().contains("diameter_mm"));
        d.diameter_mm = "0.8".into();
        d.flutes = "9".into();
        assert!(d.to_tool().unwrap_err().contains("flutes"));
        d.flutes = "2.5".into();
        assert!(d.to_tool().unwrap_err().contains("whole number"));
        d.flutes = "99999999999".into();
        assert!(d.to_tool().unwrap_err().contains("too large"));
        d.flutes = "2".into();
        d.holes = "-3".into();
        assert!(d.to_tool().unwrap_err().contains("holes"));
        d.holes = "0".into();
        d.kind = Kind::VBit;
        assert!(d.to_tool().unwrap_err().contains("tip_angle_deg"));
        d.tip_angle_deg = "30".into();
        assert_eq!(d.to_tool().unwrap().tip_angle_deg, Some(30.0));
        d.name = "".into();
        assert!(d.to_tool().unwrap_err().contains("name"));
    }

    #[test]
    fn the_form_of_a_tool_of_the_crib_gives_the_same_tool_back() {
        let mut t = Tool::new("V 30", Kind::VBit, 3.175);
        t.id = 4;
        t.tip_angle_deg = Some(30.0);
        t.hours = 1.5;
        t.holes = 12;
        t.max_hours = 10.0;
        assert_eq!(Draft::from_tool(&t).to_tool().unwrap(), t);
    }

    #[test]
    fn a_tool_is_added_with_the_next_number_and_a_name_cannot_repeat() {
        let mut w = window("add");
        add(&mut w, "Drill A", Kind::Drill, "0.8");
        assert_eq!(
            w.status,
            Status::Added {
                name: "Drill A".into(),
                id: 1
            }
        );
        assert!(w.dirty);
        assert_eq!(w.selected, Some(1));
        assert_eq!(w.draft, Draft::default());
        add(&mut w, "drill a", Kind::Drill, "1.0");
        assert!(matches!(&w.status, Status::Failed(e) if e.contains("called")));
        assert_eq!(w.crib.tools.len(), 1);
        add(&mut w, "Drill B", Kind::Drill, "99");
        assert!(matches!(&w.status, Status::Failed(e) if e.contains("diameter_mm")));
        assert!(w.status.is_problem());
    }

    #[test]
    fn the_number_of_a_removed_tool_is_not_given_again() {
        let mut w = window("ids");
        add(&mut w, "A", Kind::Drill, "0.8");
        add(&mut w, "B", Kind::Drill, "0.9");
        w.remove(2);
        assert_eq!(
            w.status,
            Status::Removed {
                name: "B".into(),
                id: 2
            }
        );
        add(&mut w, "C", Kind::Drill, "1.0");
        assert_eq!(w.crib.tools.last().unwrap().id, 3);
        w.remove(9);
        assert!(matches!(&w.status, Status::Failed(e) if e.contains("no tool 9")));
    }

    #[test]
    fn a_tool_is_edited_in_the_form_and_the_changes_are_applied_with_the_same_checks() {
        let mut w = window("edit");
        add(&mut w, "A", Kind::Drill, "0.8");
        add(&mut w, "B", Kind::EndMill, "3");
        w.edit(2);
        assert_eq!(w.draft.id, 2);
        assert_eq!(w.draft.name, "B");
        w.draft.name = "A".into();
        w.apply_draft();
        assert!(matches!(&w.status, Status::Failed(e) if e.contains("already")));
        assert_eq!(w.crib.get(2).unwrap().name, "B");
        w.draft.name = "Mill".into();
        w.draft.diameter_mm = "3.5".into();
        w.apply_draft();
        assert_eq!(
            w.status,
            Status::Updated {
                name: "Mill".into(),
                id: 2
            }
        );
        let t = w.crib.get(2).unwrap();
        assert_eq!((t.name.as_str(), t.diameter_mm), ("Mill", 3.5));
        assert_eq!(w.crib.tools.len(), 2);
        assert_eq!(w.crib.next_id, 3);
        w.edit(77);
        assert_eq!(w.draft, Draft::default());
    }

    #[test]
    fn removing_the_tool_that_is_edited_empties_the_form() {
        let mut w = window("remove");
        add(&mut w, "A", Kind::Drill, "0.8");
        w.edit(1);
        w.remove(1);
        assert_eq!(w.draft, Draft::default());
        assert_eq!(w.selected, None);
        assert!(w.selected_tool().is_none());
    }

    fn filled() -> Window {
        let mut w = window("filters");
        add(&mut w, "Drill A", Kind::Drill, "0.8");
        add(&mut w, "Mill B", Kind::EndMill, "3");
        add(&mut w, "Vee C", Kind::VBit, "3.175");
        w.draft = Draft {
            name: "Old D".into(),
            diameter_mm: "1".into(),
            hours: "9.5".into(),
            max_hours: "10".into(),
            ..Draft::default()
        };
        w.apply_draft();
        w.draft = Draft {
            name: "Dead E".into(),
            diameter_mm: "1.2".into(),
            hours: "10".into(),
            max_hours: "10".into(),
            ..Draft::default()
        };
        w.apply_draft();
        assert_eq!(w.crib.tools.len(), 5);
        w
    }

    #[test]
    fn the_search_looks_in_the_name_the_material_the_kind_and_the_number() {
        let mut w = filled();
        assert_eq!(w.visible(), vec![0, 1, 2, 3, 4]);
        w.search = " DRILL ".into();
        assert_eq!(w.visible(), vec![0, 3, 4]);
        w.search = "mill".into();
        assert_eq!(w.visible(), vec![1]);
        w.search = "3".into();
        assert_eq!(w.visible(), vec![2]);
        w.search = "carbide".into();
        assert_eq!(w.visible().len(), 5);
        w.search = "nothing like this".into();
        assert!(w.visible().is_empty());
    }

    #[test]
    fn the_filters_of_the_kind_and_of_the_wear_narrow_the_table() {
        let mut w = filled();
        w.kind_filter = Some(Kind::Drill);
        assert_eq!(w.visible(), vec![0, 3, 4]);
        w.kind_filter = None;
        w.wear_filter = WearFilter::Worn;
        assert_eq!(w.visible(), vec![3, 4]);
        w.wear_filter = WearFilter::Replace;
        assert_eq!(w.visible(), vec![4]);
        w.kind_filter = Some(Kind::VBit);
        assert!(w.visible().is_empty());
    }

    #[test]
    fn the_table_is_sorted_by_the_column_the_person_chose() {
        let mut w = filled();
        w.sort = SortKey::Diameter;
        w.descending = true;
        assert_eq!(w.visible(), vec![2, 1, 4, 3, 0]);
        w.sort = SortKey::Name;
        w.descending = false;
        assert_eq!(w.visible(), vec![4, 0, 1, 3, 2]);
        w.sort = SortKey::Wear;
        assert_eq!(w.visible(), vec![0, 1, 2, 3, 4]);
        w.descending = true;
        assert_eq!(w.visible(), vec![4, 3, 0, 1, 2]);
        w.sort = SortKey::Kind;
        w.descending = false;
        assert_eq!(w.visible(), vec![0, 3, 4, 1, 2]);
    }

    #[test]
    fn the_use_of_a_tool_is_added_to_its_counters_and_says_how_worn_it_is() {
        let mut w = window("use");
        w.record_use();
        assert_eq!(w.status, Status::NoSelection);
        w.draft = Draft {
            name: "A".into(),
            diameter_mm: "0.8".into(),
            max_hours: "10".into(),
            ..Draft::default()
        };
        w.apply_draft();
        w.use_hours = "9.5".into();
        w.use_holes = "40".into();
        w.record_use();
        assert!(matches!(
            w.status,
            Status::Used {
                id: 1,
                wear: Wear::Soon(_)
            }
        ));
        let t = w.crib.get(1).unwrap();
        assert_eq!((t.hours, t.holes), (9.5, 40));
        assert_eq!(w.use_hours, "0");
        w.use_hours = "-1".into();
        w.record_use();
        assert!(matches!(&w.status, Status::Failed(e) if e.contains("hours")));
        w.use_hours = "0".into();
        w.use_holes = "2000000000000".into();
        w.record_use();
        assert!(matches!(&w.status, Status::Failed(e) if e.contains("not a count")));
        assert_eq!(w.crib.get(1).unwrap().holes, 40);
    }

    #[test]
    fn the_crib_is_saved_and_opened_again_when_the_program_starts() {
        let mut w = window("save");
        let dir = w.data_dir.clone();
        w.save();
        assert!(matches!(
            &w.status,
            Status::Saved {
                tools: 0,
                backup: false,
                ..
            }
        ));
        add(&mut w, "A", Kind::Drill, "0.8");
        w.save();
        assert!(matches!(
            &w.status,
            Status::Saved {
                tools: 1,
                backup: false,
                ..
            }
        ));
        assert!(!w.dirty);
        let again = Window::new(dir);
        assert_eq!(again.crib.tools.len(), 1);
        assert!(matches!(&again.status, Status::Opened { tools: 1, .. }));
        assert!(!again.dirty);
        let mut w = again;
        w.file_text = String::new();
        w.save();
        assert_eq!(w.status, Status::NeedFile);
        w.open();
        assert_eq!(w.status, Status::NeedFile);
        w.file_text = w.data_dir.join("missing.json").display().to_string();
        w.open();
        assert!(w.status.is_problem());
    }

    #[test]
    fn a_new_crib_keeps_the_file_it_replaces_as_a_backup() {
        let mut w = window("newcrib");
        add(&mut w, "A", Kind::Drill, "0.8");
        w.save();
        w.new_crib();
        assert_eq!(w.status, Status::NewCrib);
        assert!(w.crib.tools.is_empty());
        w.save();
        assert!(matches!(&w.status, Status::Saved { backup: true, .. }));
        let bak = format!("{}.bak", w.file_text);
        let kept = store::load(std::path::Path::new(&bak)).unwrap();
        assert_eq!(kept.tools.len(), 1);
    }

    #[test]
    fn the_csv_is_imported_with_the_errors_of_the_library_and_exported_without_replacing() {
        let mut w = window("csv");
        let dir = w.data_dir.clone();
        add(&mut w, "A", Kind::Drill, "0.8");
        add(&mut w, "B", Kind::Drill, "0.9");
        w.save();
        let mut other = Crib::default();
        other
            .add(Tool::new("Imported", Kind::EndMill, 2.0))
            .unwrap();
        other.add(Tool::new("Second", Kind::Drill, 1.0)).unwrap();
        other.remove(2).unwrap();
        w.csv_text = dir.join("in.csv").display().to_string();
        std::fs::write(&w.csv_text, csv::to_csv(&other)).unwrap();
        w.import_csv();
        assert!(matches!(&w.status, Status::Imported { tools: 1, .. }));
        assert_eq!(w.crib.tools[0].name, "Imported");
        assert!(w.dirty);
        // the counter of numbers does not go down with the import: the numbers 1 and 2 were given before
        assert_eq!(w.crib.next_id, 3);
        w.save();
        assert!(matches!(&w.status, Status::Saved { backup: true, .. }));
        assert!(dir.join("crib.json.bak").is_file());
        std::fs::write(&w.csv_text, "id,name\n1,x\n").unwrap();
        w.import_csv();
        assert!(matches!(&w.status, Status::Failed(e) if e.contains("first row must be exactly")));
        assert_eq!(w.crib.tools[0].name, "Imported");
        w.csv_text = dir.join("missing.csv").display().to_string();
        w.import_csv();
        assert!(w.status.is_problem());

        w.csv_text = dir.join("out.csv").display().to_string();
        w.export_csv();
        assert!(matches!(&w.status, Status::Exported { tools: 1, .. }));
        let written = std::fs::read_to_string(dir.join("out.csv")).unwrap();
        assert!(written.starts_with(csv::HEADER));
        std::fs::write(dir.join("out.csv"), "mine").unwrap();
        w.export_csv();
        assert!(matches!(&w.status, Status::Exists { .. }));
        assert_eq!(
            std::fs::read_to_string(dir.join("out.csv")).unwrap(),
            "mine"
        );
        w.overwrite_csv = true;
        w.export_csv();
        assert!(matches!(&w.status, Status::Exported { .. }));
        assert!(std::fs::read_to_string(dir.join("out.csv"))
            .unwrap()
            .starts_with(csv::HEADER));
    }

    #[test]
    fn a_settings_file_that_is_wrong_is_told_and_the_defaults_are_used() {
        let dir = temp("badcfg");
        std::fs::write(dir.join("tools.cfg"), "default_max_rpm = 5\n").unwrap();
        let w = Window::new(dir);
        assert!(matches!(&w.status, Status::BadSettings(e) if e.contains("default_max_rpm")));
        assert_eq!(w.settings, Settings::default());
        assert!(w.status.is_problem());
    }

    #[test]
    fn the_settings_give_the_language_the_crib_file_and_the_numbers_of_the_calculator() {
        let dir = temp("cfg");
        let file = dir.join("mine.json");
        std::fs::write(
            dir.join("tools.cfg"),
            format!(
                "language = de\ncrib_file = {}\ndefault_max_rpm = 18000\nsurface_speed_fr4 = 80\n",
                file.display()
            ),
        )
        .unwrap();
        let w = Window::new(dir);
        assert_eq!(w.lang, Lang::De);
        assert_eq!(w.file_text, file.display().to_string());
        assert_eq!(w.calc.max_rpm, "18000");
        assert_eq!(w.calc.params.surface_speed_of(Material::Fr4), 80.0);
        assert_eq!(w.status, Status::Ready);
    }

    #[test]
    fn the_calculator_gives_the_numbers_of_the_command_line() {
        let w = window("calc");
        let c = w.calc.compute().unwrap();
        let t = Tool::new("x", Kind::Drill, 0.8);
        assert_eq!(c.cut, feeds_speeds(&t, Material::Fr4, 24000.0).unwrap());
        assert_eq!(c.cut.rpm, 39789.0_f64.min(24000.0));
        assert!(c.cut.rpm_limited);
    }

    #[test]
    fn the_calculator_lowers_the_speed_when_the_spindle_cannot_turn_so_fast_and_says_so() {
        let mut w = window("calc2");
        w.calc.diameter_mm = "3".into();
        w.calc.max_rpm = "24000".into();
        let c = w.calc.compute().unwrap();
        assert!(!c.cut.rpm_limited);
        assert_eq!(
            c.cut.rpm,
            (100.0_f64 * 1000.0 / (std::f64::consts::PI * 3.0)).round()
        );
        let free = c.explain(Lang::En).join("\n");
        assert!(free.contains("can turn that fast"), "{free}");
        assert!(!free.contains("lowered"), "{free}");
        w.calc.max_rpm = "5000".into();
        let c = w.calc.compute().unwrap();
        assert!(c.cut.rpm_limited);
        assert_eq!(c.cut.rpm, 5000.0);
        let lines = c.explain(Lang::En);
        assert!(
            lines.iter().any(|l| l.contains("lowered to 5000 rpm")),
            "{lines:?}"
        );
        assert!(lines.iter().any(|l| l.starts_with("speed = surface speed")));
        assert!(lines.iter().any(|l| l.starts_with("feed = ")));
        assert!(lines.iter().any(|l| l.contains("goes through in one pass")));
    }

    #[test]
    fn a_v_bit_works_at_its_point_and_the_explanation_says_which_diameter_was_used() {
        let mut w = window("calc3");
        w.calc.kind = Kind::VBit;
        w.calc.diameter_mm = "3.175".into();
        assert!(w.calc.compute().unwrap_err().contains("tip_angle_deg"));
        w.calc.tip_angle_deg = "30".into();
        let c = w.calc.compute().unwrap();
        assert!(c.trace.uses_point);
        assert_eq!(c.trace.working_mm, 0.5);
        let lines = c.explain(Lang::En);
        assert!(
            lines[0].contains("diameter of its point") && lines[0].contains("0.5 mm is used"),
            "{lines:?}"
        );
        assert!(lines.iter().any(|l| l.starts_with("depth of a pass")));
        assert!(lines.iter().any(|l| l.starts_with("plunge = feed")));
    }

    #[test]
    fn the_numbers_of_the_calculation_can_be_changed_and_put_back() {
        let mut w = window("calc4");
        let before = w.calc.compute().unwrap().cut;
        w.calc.params.surface_speed[Material::Fr4.index()] = 50.0;
        w.calc.max_rpm = "100000".into();
        let after = w.calc.compute().unwrap().cut;
        assert!(after.rpm < before.rpm);
        assert_eq!(
            after.rpm,
            (50.0_f64 * 1000.0 / (std::f64::consts::PI * 0.8)).round()
        );
        w.calc.params.chip_fraction[Material::Fr4.index()] = 0.0;
        assert!(w.calc.compute().unwrap_err().contains("chip_fraction_fr4"));
        w.reset_params();
        assert_eq!(w.calc.params, CalcParams::default());
    }

    #[test]
    fn the_calculator_refuses_what_the_command_line_refuses() {
        let mut w = window("calc5");
        w.calc.kind = Kind::Laser;
        assert!(w
            .calc
            .compute()
            .unwrap_err()
            .contains("no feeds and speeds"));
        w.calc.kind = Kind::Drill;
        w.calc.max_rpm = "500".into();
        assert!(w.calc.compute().unwrap_err().contains("at least 1000"));
        w.calc.max_rpm = "fast".into();
        assert!(w.calc.compute().unwrap_err().contains("max_rpm"));
        w.calc.max_rpm = "24000".into();
        w.calc.flutes = "0".into();
        assert!(w.calc.compute().unwrap_err().contains("flutes"));
    }

    #[test]
    fn a_tool_of_the_crib_fills_the_calculator() {
        let mut w = window("calc6");
        w.calc_from_selected();
        assert_eq!(w.status, Status::NoSelection);
        add(&mut w, "Vee", Kind::VBit, "3.175");
        w.calc_from_selected();
        assert_eq!(w.calc.kind, Kind::VBit);
        assert_eq!(w.calc.diameter_mm, "3.175");
        assert_eq!(w.calc.tip_angle_deg, "30");
        assert!(w.calc.compute().is_ok());
    }

    #[test]
    fn every_status_has_a_sentence_in_every_language() {
        let statuses = [
            Status::Ready,
            Status::Opened {
                file: "f".into(),
                tools: 2,
            },
            Status::Saved {
                file: "f".into(),
                tools: 2,
                backup: false,
            },
            Status::Saved {
                file: "f".into(),
                tools: 2,
                backup: true,
            },
            Status::NewCrib,
            Status::Added {
                name: "n".into(),
                id: 1,
            },
            Status::Updated {
                name: "n".into(),
                id: 1,
            },
            Status::Removed {
                name: "n".into(),
                id: 1,
            },
            Status::Used {
                id: 1,
                wear: Wear::Soon(0.95),
            },
            Status::Used {
                id: 1,
                wear: Wear::Replace,
            },
            Status::Imported {
                file: "f".into(),
                tools: 2,
            },
            Status::Exported {
                file: "f".into(),
                tools: 2,
            },
            Status::Failed("why".into()),
            Status::BadSettings("why".into()),
            Status::NeedFile,
            Status::NoSelection,
            Status::Exists { file: "f".into() },
        ];
        for s in &statuses {
            for l in Lang::ALL {
                let text = s.text(l);
                assert!(
                    !text.trim().is_empty() && !text.contains('{'),
                    "{s:?} in {}",
                    l.code()
                );
            }
        }
        assert_eq!(
            Status::Added {
                name: "Drill".into(),
                id: 7
            }
            .text(Lang::En),
            "Tool 7 added: Drill"
        );
        assert!(!Status::Ready.is_problem());
        assert_eq!(wear_text(Wear::Good(0.5), Lang::En), "good (50 %)");
    }

    #[test]
    fn the_words_of_the_kinds_the_materials_the_filters_and_the_columns_exist() {
        for k in crate::model::KINDS {
            assert!(!kind_msg(k).tr(Lang::Pt).is_empty());
        }
        for m in crate::calc::MATERIALS {
            assert!(!material_msg(m).tr(Lang::Ja).is_empty());
        }
        for f in WearFilter::ALL {
            assert!(!f.msg().tr(Lang::Fr).is_empty());
        }
        for s in SortKey::ALL {
            assert!(!s.msg().tr(Lang::It).is_empty());
        }
    }
}
