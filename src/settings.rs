//! The settings that a person may change, read from the lines of `tools.cfg`: the language and the size of the window, the file of the crib, the speed of the spindle the calculator
//! starts with and every number the feeds and speeds are worked out with. A key that is not here is left alone; a value that makes no sense is an error that names the line, never a
//! value that only looks right.

use crate::calc::{
    CalcParams, Material, CHIP_FRACTION_LIMITS, DRILL_PLUNGE_LIMITS, MAX_PASS_LIMITS,
    PASS_FRACTION_LIMITS, PLUNGE_RATIO_LIMITS, SURFACE_SPEED_LIMITS, V_BIT_ANGLE_LIMITS,
    V_BIT_FLAT_LIMITS, WORKING_LIMITS,
};
use crate::files;
use crate::lang::Lang;
use std::path::{Path, PathBuf};

/// The name of the settings file, in the folder of the program.
pub const CONFIG_FILE: &str = "tools.cfg";
/// The values used when the file does not say otherwise.
pub const DEFAULT_WINDOW_WIDTH: u32 = 1200;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 760;
pub const DEFAULT_MAX_RPM: f64 = 24000.0;
/// The largest crib or CSV file that is read, in MiB.
pub const DEFAULT_MAX_FILE_MIB: u64 = 16;
/// The largest `tools.cfg` that is read, in bytes.
pub const MAX_CONFIG_BYTES: u64 = 1_048_576;
/// The limits a value of the file may have.
pub const MIN_WINDOW: (u32, u32) = (720, 480);
pub const MAX_WINDOW: (u32, u32) = (7680, 4320);
pub const MAX_RPM_LIMITS: (f64, f64) = (1000.0, 400_000.0);
pub const MAX_FILE_MIB_LIMITS: (u32, u32) = (1, 1024);
/// How long a change waits for another program that is changing the same crib, in seconds: the default and the limits of `lock_wait_s`.
pub const DEFAULT_LOCK_WAIT_S: u32 = 5;
pub const LOCK_WAIT_S_LIMITS: (u32, u32) = (0, 60);
/// The most rows read from a CSV table: the default and the limits of `max_csv_rows`.
pub const DEFAULT_MAX_CSV_ROWS: u32 = 10_000;
pub const MAX_CSV_ROWS_LIMITS: (u32, u32) = (1, 100_000);
/// How far a size asked of a tool or a sheet may be from the one in the crib before `check` says it is another one, in mm: the default and the limits of `check_tolerance_mm`.
pub const DEFAULT_CHECK_TOLERANCE_MM: f64 = 0.01;
pub const CHECK_TOLERANCE_LIMITS: (f64, f64) = (0.0001, 1.0);

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// The language of the window; `None` follows the system.
    pub language: Option<Lang>,
    pub window_width: u32,
    pub window_height: u32,
    /// The file of the crib the window opens; `None` is `crib.json` in the folder of the program.
    pub crib_file: Option<PathBuf>,
    /// The speed of the spindle the calculator starts with.
    pub default_max_rpm: f64,
    /// The largest crib or CSV file that is read, in MiB; a bigger one is refused with the code `file_too_large`.
    pub max_file_mib: u64,
    /// How long a change waits for another program that is changing the same crib, in seconds, before it is refused with the code `file_locked`.
    pub lock_wait_s: u32,
    /// The most rows read from a CSV table; a table with more is refused.
    pub max_csv_rows: usize,
    /// How far a size asked of a tool or a sheet may be from the one in the crib in `check`, in mm.
    pub check_tolerance_mm: f64,
    /// The link to the EH3D server (`server_address`, `server_token`, `server_machine`, `server_timeout_s`, `server_retries`, `server_mdns`, `server_max_answer_mib`): off until
    /// `server_address` is set.
    pub server: eh3d_serverclient::Settings,
    pub params: CalcParams,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: None,
            window_width: DEFAULT_WINDOW_WIDTH,
            window_height: DEFAULT_WINDOW_HEIGHT,
            crib_file: None,
            default_max_rpm: DEFAULT_MAX_RPM,
            max_file_mib: DEFAULT_MAX_FILE_MIB,
            lock_wait_s: DEFAULT_LOCK_WAIT_S,
            max_csv_rows: DEFAULT_MAX_CSV_ROWS as usize,
            check_tolerance_mm: DEFAULT_CHECK_TOLERANCE_MM,
            server: eh3d_serverclient::Settings::default(),
            params: CalcParams::default(),
        }
    }
}

/// The folder where the program keeps its files (`tools.cfg`, `crib.json`): the variable `EH3D_TOOLS_DIR`, or the application data folder.
pub fn data_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("EH3D_TOOLS_DIR") {
        return PathBuf::from(d);
    }
    if let Some(d) = std::env::var_os("APPDATA") {
        return PathBuf::from(d).join("eh3d-tools");
    }
    if let Some(d) = std::env::var_os("HOME") {
        return PathBuf::from(d).join(".config").join("eh3d-tools");
    }
    PathBuf::from(".")
}

/// A whole number between `low` and `high`, or the reason it is not.
fn whole(line: usize, key: &str, value: &str, limits: (u32, u32)) -> Result<u32, String> {
    match value.parse::<u32>() {
        Ok(n) if (limits.0..=limits.1).contains(&n) => Ok(n),
        _ => Err(format!(
            "{CONFIG_FILE} line {line}: {key} is '{value}', a whole number from {} to {} is needed",
            limits.0, limits.1
        )),
    }
}

/// A number between the limits, or the reason it is not.
fn float(line: usize, key: &str, value: &str, limits: (f64, f64)) -> Result<f64, String> {
    match value.parse::<f64>() {
        Ok(n) if n.is_finite() && (limits.0..=limits.1).contains(&n) => Ok(n),
        _ => Err(format!(
            "{CONFIG_FILE} line {line}: {key} is '{value}', a number from {} to {} is needed",
            limits.0, limits.1
        )),
    }
}

/// The material that ends a key such as `surface_speed_wood`.
fn material(line: usize, key: &str, name: &str) -> Result<Material, String> {
    Material::parse(name).map_err(|_| {
        format!(
            "{CONFIG_FILE} line {line}: {key} names no material (fr4, aluminium, acrylic, wood or brass)"
        )
    })
}

impl Settings {
    /// Reads the text of the file. Empty lines and lines that start with `#` are skipped; `key = value`, with the value optionally in quotes.
    pub fn parse(text: &str) -> Result<Settings, String> {
        let mut s = Settings::default();
        for (n, raw) in text.trim_start_matches('\u{feff}').lines().enumerate() {
            let line = n + 1;
            let raw = raw.trim();
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let Some((key, value)) = raw.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim().trim_matches('"'));
            let p = &mut s.params;
            match key {
                "language" => {
                    let v = value.to_ascii_lowercase();
                    s.language = if v == "auto" {
                        None
                    } else {
                        Some(
                            Lang::ALL
                                .into_iter()
                                .find(|l| l.code() == v)
                                .ok_or_else(|| {
                                    format!(
                                        "{CONFIG_FILE} line {line}: language is '{value}', one of auto, en, es, de, fr, it, ja, zh or pt is needed"
                                    )
                                })?,
                        )
                    };
                }
                "window_width" => {
                    s.window_width = whole(line, key, value, (MIN_WINDOW.0, MAX_WINDOW.0))?;
                }
                "window_height" => {
                    s.window_height = whole(line, key, value, (MIN_WINDOW.1, MAX_WINDOW.1))?;
                }
                "crib_file" => {
                    s.crib_file = if value.is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(value))
                    };
                }
                "default_max_rpm" => {
                    s.default_max_rpm = float(line, key, value, MAX_RPM_LIMITS)?;
                }
                "max_file_mib" => {
                    s.max_file_mib = u64::from(whole(line, key, value, MAX_FILE_MIB_LIMITS)?);
                }
                "lock_wait_s" => {
                    s.lock_wait_s = whole(line, key, value, LOCK_WAIT_S_LIMITS)?;
                }
                "max_csv_rows" => {
                    s.max_csv_rows = whole(line, key, value, MAX_CSV_ROWS_LIMITS)? as usize;
                }
                "check_tolerance_mm" => {
                    s.check_tolerance_mm = float(line, key, value, CHECK_TOLERANCE_LIMITS)?;
                }
                k if crate::server::is_server_key(k) => {
                    s.server
                        .set(k, value)
                        .map_err(|e| format!("{CONFIG_FILE} line {line}: {e}"))?;
                }
                "working_min_mm" => p.working_min_mm = float(line, key, value, WORKING_LIMITS)?,
                "working_max_mm" => p.working_max_mm = float(line, key, value, WORKING_LIMITS)?,
                "min_working_mm" => p.min_working_mm = float(line, key, value, WORKING_LIMITS)?,
                "drill_plunge" => p.drill_plunge = float(line, key, value, DRILL_PLUNGE_LIMITS)?,
                "plunge_ratio" => p.plunge_ratio = float(line, key, value, PLUNGE_RATIO_LIMITS)?,
                "pass_fraction" => {
                    p.pass_fraction = float(line, key, value, PASS_FRACTION_LIMITS)?;
                }
                "max_pass_mm" => p.max_pass_mm = float(line, key, value, MAX_PASS_LIMITS)?,
                "v_bit_angle_deg" => {
                    p.v_bit_angle_deg = float(line, key, value, V_BIT_ANGLE_LIMITS)?;
                }
                "v_bit_flat_mm" => p.v_bit_flat_mm = float(line, key, value, V_BIT_FLAT_LIMITS)?,
                _ => {
                    if let Some(name) = key.strip_prefix("surface_speed_") {
                        let m = material(line, key, name)?;
                        p.surface_speed[m.index()] = float(line, key, value, SURFACE_SPEED_LIMITS)?;
                    } else if let Some(name) = key.strip_prefix("chip_fraction_") {
                        let m = material(line, key, name)?;
                        p.chip_fraction[m.index()] = float(line, key, value, CHIP_FRACTION_LIMITS)?;
                    }
                }
            }
        }
        // the numbers that depend on each other (the lowest working diameter is not above the highest) are checked once all are read
        s.params
            .validate()
            .map_err(|e| format!("{CONFIG_FILE}: {e}"))?;
        Ok(s)
    }

    /// Reads `tools.cfg` in `dir`; a file that is not there means the defaults.
    pub fn load(dir: &Path) -> Result<Settings, String> {
        let file = dir.join(CONFIG_FILE);
        if !file.exists() {
            return Ok(Settings::default());
        }
        Settings::parse(&files::read_text_limited(&file, MAX_CONFIG_BYTES)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_file_gives_the_defaults() {
        assert_eq!(Settings::parse("").unwrap(), Settings::default());
        assert_eq!(Settings::default().params, CalcParams::default());
    }

    #[test]
    fn the_keys_are_read_and_the_other_lines_are_left_alone() {
        let s = Settings::parse(
            "\u{feff}# comment\nlanguage = \"PT\"\nwindow_width = 1400\nwindow_height=900\ncrib_file = C:\\shop\\crib.json\ndefault_max_rpm = 30000\nsurface_speed_wood = 300\nchip_fraction_fr4 = 0.02\nplunge_ratio = 0.25\nunknown = 1\nno equals sign\n",
        )
        .unwrap();
        assert_eq!(s.language, Some(Lang::Pt));
        assert_eq!((s.window_width, s.window_height), (1400, 900));
        assert_eq!(s.crib_file, Some(PathBuf::from("C:\\shop\\crib.json")));
        assert_eq!(s.default_max_rpm, 30000.0);
        assert_eq!(s.params.surface_speed_of(Material::Wood), 300.0);
        assert_eq!(s.params.surface_speed_of(Material::Fr4), 100.0);
        assert_eq!(s.params.chip_fraction_of(Material::Fr4), 0.02);
        assert_eq!(s.params.plunge_ratio, 0.25);
        assert_eq!(Settings::parse("language = auto").unwrap().language, None);
        assert_eq!(Settings::parse("crib_file = ").unwrap().crib_file, None);
    }

    #[test]
    fn a_value_that_makes_no_sense_names_its_line() {
        for (text, key) in [
            ("\nwindow_width = 100", "window_width"),
            ("window_height = tall", "window_height"),
            ("language = klingon", "language"),
            ("default_max_rpm = 10", "default_max_rpm"),
            ("default_max_rpm = NaN", "default_max_rpm"),
            ("surface_speed_brass = 0", "surface_speed_brass"),
            ("chip_fraction_wood = 3", "chip_fraction_wood"),
            ("surface_speed_steel = 100", "surface_speed_steel"),
            ("drill_plunge = -1", "drill_plunge"),
            ("max_pass_mm = 1000", "max_pass_mm"),
            ("v_bit_angle_deg = 5", "v_bit_angle_deg"),
            ("v_bit_angle_deg = 171", "v_bit_angle_deg"),
            ("v_bit_angle_deg = sharp", "v_bit_angle_deg"),
            ("v_bit_flat_mm = -0.1", "v_bit_flat_mm"),
            ("v_bit_flat_mm = 6", "v_bit_flat_mm"),
        ] {
            let e = Settings::parse(text).unwrap_err();
            assert!(e.contains(key) && e.contains("line"), "{e}");
        }
        assert!(Settings::parse("\nwindow_width = 99999")
            .unwrap_err()
            .contains("line 2"));
    }

    #[test]
    fn numbers_that_depend_on_each_other_are_checked_together() {
        let e = Settings::parse("working_min_mm = 2\nworking_max_mm = 1").unwrap_err();
        assert!(
            e.contains("working_min_mm") && e.contains("working_max_mm"),
            "{e}"
        );
    }

    #[test]
    fn a_missing_file_is_the_defaults_and_a_file_is_read() {
        let dir = std::env::temp_dir().join(format!("eh3d-tools-settings-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(Settings::load(&dir).unwrap(), Settings::default());
        std::fs::write(dir.join(CONFIG_FILE), "default_max_rpm = 18000\n").unwrap();
        assert_eq!(Settings::load(&dir).unwrap().default_max_rpm, 18000.0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_angle_and_the_flat_of_the_v_bit_are_read_and_have_the_defaults_without_them() {
        let d = Settings::parse("").unwrap().params;
        assert_eq!((d.v_bit_angle_deg, d.v_bit_flat_mm), (60.0, 0.1));
        let s = Settings::parse(
            "v_bit_angle_deg = 90
v_bit_flat_mm = 0.05
",
        )
        .unwrap();
        assert_eq!(
            (s.params.v_bit_angle_deg, s.params.v_bit_flat_mm),
            (90.0, 0.05)
        );
        // the limits are the ones of the library, both ends included
        assert!(Settings::parse(
            "v_bit_angle_deg = 10
v_bit_flat_mm = 0"
        )
        .is_ok());
        assert!(Settings::parse(
            "v_bit_angle_deg = 170
v_bit_flat_mm = 5"
        )
        .is_ok());
        let e = Settings::parse(
            "

v_bit_angle_deg = 175",
        )
        .unwrap_err();
        assert!(
            e.contains("line 3") && e.contains("10") && e.contains("170"),
            "{e}"
        );
    }

    #[test]
    fn the_limit_of_the_files_is_read_and_has_its_limits() {
        assert_eq!(Settings::default().max_file_mib, 16);
        assert_eq!(
            Settings::parse("max_file_mib = 64").unwrap().max_file_mib,
            64
        );
        assert!(Settings::parse(
            "max_file_mib = 1
max_file_mib = 1024"
        )
        .is_ok());
        for bad in [
            "max_file_mib = 0",
            "max_file_mib = 1025",
            "max_file_mib = big",
        ] {
            let e = Settings::parse(bad).unwrap_err();
            assert!(e.contains("max_file_mib") && e.contains("1 to 1024"), "{e}");
        }
    }

    #[test]
    fn the_sample_file_has_the_default_values() {
        let sample = include_str!("../tools.cfg.example");
        assert_eq!(Settings::parse(sample).unwrap(), Settings::default());
        assert!(sample.contains("max_file_mib = 16"));
        assert!(sample.contains("lock_wait_s = 5"));
        assert!(sample.contains("max_csv_rows = 10000"));
        assert!(sample.contains("check_tolerance_mm = 0.01"));
    }

    #[test]
    fn the_wait_for_the_lock_is_read_and_has_its_limits() {
        assert_eq!(Settings::default().lock_wait_s, 5);
        for good in [0, 60] {
            let s = Settings::parse(&format!("lock_wait_s = {good}")).unwrap();
            assert_eq!(s.lock_wait_s, good);
        }
        for bad in ["lock_wait_s = 61", "lock_wait_s = -1", "lock_wait_s = long"] {
            let e = Settings::parse(bad).unwrap_err();
            assert!(e.contains("lock_wait_s") && e.contains("0 to 60"), "{e}");
        }
    }

    #[test]
    fn the_rows_and_the_tolerance_of_the_check_are_read_and_have_their_limits() {
        let s = Settings::parse("max_csv_rows = 50\ncheck_tolerance_mm = 0.5").unwrap();
        assert_eq!((s.max_csv_rows, s.check_tolerance_mm), (50, 0.5));
        for bad in [
            "max_csv_rows = 0",
            "max_csv_rows = 100001",
            "max_csv_rows = many",
            "check_tolerance_mm = 0",
            "check_tolerance_mm = 2",
        ] {
            let e = Settings::parse(bad).unwrap_err();
            assert!(
                e.contains("line 1") && e.contains(bad.split(' ').next().unwrap()),
                "{e}"
            );
        }
    }
}
