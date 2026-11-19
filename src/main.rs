// EH3D-TOOL-MANAGER - command line.
// Copyright (C) 2026 JuanenRac (Electro Hobby 3D). GPL-3.0-or-later.
#![forbid(unsafe_code)]

mod cli_args;

use eh3d_tool_manager::calc::{feeds_speeds_traced_at, v_bit_width_mm, Material};
use eh3d_tool_manager::model::{Crib, Kind, Selection, Sort, Tool, Wear};
use eh3d_tool_manager::settings::{data_dir, Settings};
use eh3d_tool_manager::{csv, files, store};
use std::env;
use std::path::Path;
use std::process::ExitCode;

const HELP: &str = "eh3d-tools: the tool crib of the EH3D Fabrication Suite

  eh3d-tools init FILE.json
  eh3d-tools add FILE.json NAME KIND DIAMETER_MM [--flutes N] [--angle DEG] [--max-hours H] [--max-holes N]
  eh3d-tools list FILE.json [--kind KIND] [--worn] [--sort id|name|wear]
  eh3d-tools show FILE.json ID              every value of a tool and what is left before its limits
  eh3d-tools use FILE.json ID [--hours H] [--meters M] [--holes N]
  eh3d-tools replace FILE.json ID           a new tool in the place of a worn one: the counters go back to zero
  eh3d-tools remove FILE.json ID            take a tool out of the crib (its number is never given again)
  eh3d-tools wear FILE.json                 the tools that are worn or about to be
  eh3d-tools stats FILE.json                what the tools add up to, by kind
  eh3d-tools export-csv FILE.json
  eh3d-tools import-csv FILE.json TABLE.csv
  eh3d-tools restore FILE.json               put back FILE.json.bak (kept by the last save); the file it replaces is kept as FILE.json.damaged
  eh3d-tools calc KIND DIAMETER_MM MATERIAL MAX_RPM [--flutes N] [--angle DEG] [--depth MM]
  eh3d-tools vbit --depth MM [--angle DEG] [--flat MM]     the width of the cut of a V-bit at a depth

KIND: drill, end-mill, v-bit, engraver, laser, nozzle. MATERIAL: fr4, aluminium, acrylic, wood, brass.
vbit works out 2 x depth x tan(angle / 2) + flat (the angle is 10 to 170 degrees, 60 and the flat 0.1 mm by default, both set in tools.cfg); calc --depth uses that width as the diameter a V-bit or an engraver works with.
The feeds and speeds are starting points to try on a scrap board, not a promise. The numbers they are worked out with can be changed in tools.cfg (see tools.cfg.example).";

fn flag(args: &mut Vec<String>, name: &str) -> Result<Option<f64>, String> {
    cli_args::flag(args, name)?
        .map(|value| cli_args::num(&value, name))
        .transpose()
}

/// A flag that takes a whole number of 0 or more (`--flutes 2.9` or `--max-holes -5` are errors).
fn whole_flag(args: &mut Vec<String>, name: &str) -> Result<Option<u64>, String> {
    cli_args::flag(args, name)?
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| format!("{name} '{value}' is not a whole number of 0 or more"))
        })
        .transpose()
}

/// One line of a list of tools.
fn line(t: &Tool) -> String {
    format!(
        "{:>3}  {:<24} {:<9} {:>6.3} mm  {:>2} flutes  {:>7.1} h  {:>7} holes  {}",
        t.id,
        t.name,
        t.kind.key(),
        t.diameter_mm,
        t.flutes,
        t.hours,
        t.holes,
        describe(t.wear())
    )
}

/// Every value of a tool, one to a line, and what is left before each of its limits.
fn detail(t: &Tool) -> String {
    let left = t.remaining();
    let hours = match left.hours {
        Some(h) => format!("{} h ({h:.1} h left)", t.max_hours),
        None => "no limit".to_string(),
    };
    let holes = match left.holes {
        Some(h) => format!("{} ({h} left)", t.max_holes),
        None => "no limit".to_string(),
    };
    let angle = t
        .tip_angle_deg
        .map(|a| format!("  tip angle   {a} degrees\n"))
        .unwrap_or_default();
    format!(
        "tool {}: {}\n  kind        {}\n  diameter    {} mm\n  flutes      {}\n  length      {} mm\n{angle}  shank       {} mm\n  material    {}\n  used        {:.1} h, {:.1} m, {} holes\n  hour limit  {hours}\n  hole limit  {holes}\n  wear        {}",
        t.id,
        t.name,
        t.kind.key(),
        t.diameter_mm,
        t.flutes,
        t.length_mm,
        t.shank_mm,
        t.material,
        t.hours,
        t.meters,
        t.holes,
        describe(t.wear())
    )
}

/// The number of a tool given as a word of the command line.
fn tool_number(text: &str) -> Result<u32, String> {
    text.parse::<u32>()
        .map_err(|_| format!("'{text}' is not a tool number"))
}

fn describe(w: Wear) -> String {
    match w {
        Wear::Good(f) => format!("good ({:.0} %)", f * 100.0),
        Wear::Soon(f) => format!("replace soon ({:.0} %)", f * 100.0),
        Wear::Replace => "REPLACE".to_string(),
    }
}

/// The largest crib or CSV file that is read: `max_file_mib` of `tools.cfg`.
fn max_mib() -> Result<u64, String> {
    Ok(Settings::load(&data_dir())?.max_file_mib)
}

fn run(mut args: Vec<String>) -> Result<(), String> {
    if args.is_empty() || matches!(args[0].as_str(), "help" | "--help" | "-h") {
        println!("{HELP}");
        return Ok(());
    }
    let cmd = args.remove(0);
    let need = |args: &Vec<String>, n: usize, what: &str| -> Result<(), String> {
        if args.len() < n {
            Err(format!("{cmd} needs {what}"))
        } else {
            Ok(())
        }
    };
    match cmd.as_str() {
        "init" => {
            need(&args, 1, "a file")?;
            cli_args::finish(&args, 1)?;
            if Path::new(&args[0]).exists() {
                return Err(format!(
                    "{}: already exists; nothing was overwritten",
                    args[0]
                ));
            }
            store::save(&Crib::default(), Path::new(&args[0]))
        }
        "add" => {
            let flutes = whole_flag(&mut args, "--flutes")?;
            let angle = flag(&mut args, "--angle")?;
            let max_hours = flag(&mut args, "--max-hours")?;
            let max_holes = whole_flag(&mut args, "--max-holes")?;
            need(&args, 4, "a file, a name, a kind and a diameter")?;
            cli_args::finish(&args, 4)?;
            let file = Path::new(&args[0]);
            let mut crib = store::load_with_limit(file, max_mib()?)?;
            let diameter = args[3]
                .parse::<f64>()
                .map_err(|_| format!("the diameter '{}' is not a number", args[3]))?;
            let mut t = Tool::new(&args[1], Kind::parse(&args[2])?, diameter);
            if let Some(f) = flutes {
                t.flutes = u32::try_from(f).map_err(|_| format!("--flutes {f} is too large"))?;
            }
            t.tip_angle_deg = angle;
            t.max_hours = max_hours.unwrap_or(0.0);
            t.max_holes = max_holes.unwrap_or(0);
            let id = crib.add(t)?;
            store::save(&crib, file)?;
            println!("tool {id} added");
            Ok(())
        }
        "list" => {
            let selection = Selection {
                kind: cli_args::flag(&mut args, "--kind")?
                    .map(|k| Kind::parse(&k))
                    .transpose()?,
                worn_only: cli_args::switch(&mut args, "--worn"),
                sort: cli_args::flag(&mut args, "--sort")?
                    .map(|s| Sort::parse(&s))
                    .transpose()?
                    .unwrap_or_default(),
            };
            need(&args, 1, "a file")?;
            cli_args::finish(&args, 1)?;
            let crib = store::load_with_limit(Path::new(&args[0]), max_mib()?)?;
            let shown = crib.select(&selection);
            for t in &shown {
                println!("{}", line(t));
            }
            if shown.is_empty() && (selection.kind.is_some() || selection.worn_only) {
                println!("no tool matches");
            }
            Ok(())
        }
        "show" => {
            need(&args, 2, "a file and a tool number")?;
            cli_args::finish(&args, 2)?;
            let crib = store::load_with_limit(Path::new(&args[0]), max_mib()?)?;
            let id = tool_number(&args[1])?;
            let tool = crib
                .get(id)
                .ok_or_else(|| format!("there is no tool {id}"))?;
            println!("{}", detail(tool));
            Ok(())
        }
        "replace" => {
            need(&args, 2, "a file and a tool number")?;
            cli_args::finish(&args, 2)?;
            let file = Path::new(&args[0]);
            let mut crib = store::load_with_limit(file, max_mib()?)?;
            let id = tool_number(&args[1])?;
            let (hours, meters, holes) = crib.replace_tool(id)?;
            store::save(&crib, file)?;
            println!(
                "tool {id}: counters back to zero (they said {hours:.1} h, {meters:.1} m, {holes} holes)"
            );
            Ok(())
        }
        "remove" => {
            need(&args, 2, "a file and a tool number")?;
            cli_args::finish(&args, 2)?;
            let file = Path::new(&args[0]);
            let mut crib = store::load_with_limit(file, max_mib()?)?;
            let removed = crib.remove(tool_number(&args[1])?)?;
            store::save(&crib, file)?;
            println!(
                "tool {} ({}) removed; its number is not given again",
                removed.id, removed.name
            );
            Ok(())
        }
        "stats" => {
            need(&args, 1, "a file")?;
            cli_args::finish(&args, 1)?;
            let crib = store::load_with_limit(Path::new(&args[0]), max_mib()?)?;
            let rows = crib.summary();
            if rows.is_empty() {
                println!("the crib has no tool");
                return Ok(());
            }
            println!("kind       tools  hours   metres    holes  to replace");
            for r in &rows {
                println!(
                    "{:<9} {:>6} {:>6.1} {:>8.1} {:>8} {:>11}",
                    r.kind.key(),
                    r.tools,
                    r.hours,
                    r.meters,
                    r.holes,
                    r.to_replace
                );
            }
            println!(
                "{:<9} {:>6} {:>6.1} {:>8.1} {:>8} {:>11}",
                "total",
                rows.iter().map(|r| r.tools).sum::<usize>(),
                rows.iter().map(|r| r.hours).sum::<f64>(),
                rows.iter().map(|r| r.meters).sum::<f64>(),
                rows.iter().fold(0u64, |s, r| s.saturating_add(r.holes)),
                rows.iter().map(|r| r.to_replace).sum::<usize>()
            );
            Ok(())
        }
        "use" => {
            let hours = flag(&mut args, "--hours")?.unwrap_or(0.0);
            let meters = flag(&mut args, "--meters")?.unwrap_or(0.0);
            let holes = whole_flag(&mut args, "--holes")?.unwrap_or(0);
            need(&args, 2, "a file and a tool number")?;
            cli_args::finish(&args, 2)?;
            if holes > 1_000_000_000_000 {
                return Err(format!("--holes {holes} is not a count of holes"));
            }
            let file = Path::new(&args[0]);
            let mut crib = store::load_with_limit(file, max_mib()?)?;
            let id = args[1]
                .parse::<u32>()
                .map_err(|_| format!("'{}' is not a tool number", args[1]))?;
            let w = crib.record_use(id, hours, meters, holes)?;
            store::save(&crib, file)?;
            println!("tool {id}: {}", describe(w));
            Ok(())
        }
        "wear" => {
            need(&args, 1, "a file")?;
            cli_args::finish(&args, 1)?;
            let crib = store::load_with_limit(Path::new(&args[0]), max_mib()?)?;
            let list = crib.to_replace();
            if list.is_empty() {
                println!("no tool is worn");
            }
            for (t, w) in list {
                println!("{:>3}  {:<24} {}", t.id, t.name, describe(w));
            }
            Ok(())
        }
        "export-csv" => {
            need(&args, 1, "a file")?;
            cli_args::finish(&args, 1)?;
            print!(
                "{}",
                csv::to_csv(&store::load_with_limit(Path::new(&args[0]), max_mib()?)?)
            );
            Ok(())
        }
        "import-csv" => {
            need(&args, 2, "a crib file and a CSV table")?;
            cli_args::finish(&args, 2)?;
            let text = files::read_text_limited(Path::new(&args[1]), files::mib(max_mib()?))?;
            let crib = csv::from_csv(&text)?;
            // the table that was there is kept beside the new one: an import replaces it
            let target = Path::new(&args[0]);
            if target.exists() {
                let mut bak = target.as_os_str().to_owned();
                bak.push(".bak");
                std::fs::copy(target, &bak).map_err(|e| format!("{}.bak: {e}", args[0]))?;
                println!("the previous crib is kept as {}.bak", args[0]);
            }
            store::save(&crib, target)?;
            println!("{} tools written to {}", crib.tools.len(), args[0]);
            Ok(())
        }
        "restore" => {
            need(&args, 1, "a crib file")?;
            cli_args::finish(&args, 1)?;
            let target = Path::new(&args[0]);
            let tools = store::restore_from_backup(target, max_mib()?)?;
            println!(
                "WARNING: {} was replaced by its backup {}.bak ({tools} tools); whatever was saved after that backup is not in it. The file that was there is kept as {}.damaged",
                args[0], args[0], args[0]
            );
            Ok(())
        }
        "calc" => {
            let flutes = whole_flag(&mut args, "--flutes")?;
            let angle = flag(&mut args, "--angle")?;
            let depth = flag(&mut args, "--depth")?;
            need(&args, 4, "a kind, a diameter, a material and a maximum rpm")?;
            cli_args::finish(&args, 4)?;
            let diameter = args[1]
                .parse::<f64>()
                .map_err(|_| format!("the diameter '{}' is not a number", args[1]))?;
            let mut t = Tool::new("calc", Kind::parse(&args[0])?, diameter);
            if let Some(f) = flutes {
                t.flutes = u32::try_from(f).map_err(|_| format!("--flutes {f} is too large"))?;
            }
            t.tip_angle_deg = angle;
            let max_rpm = args[3]
                .parse::<f64>()
                .map_err(|_| format!("the maximum rpm '{}' is not a number", args[3]))?;
            // the surface speeds, the chip loads and the factors are the ones of tools.cfg (the defaults when there is none)
            let params = Settings::load(&data_dir())?.params;
            let (c, trace) =
                feeds_speeds_traced_at(&t, Material::parse(&args[2])?, max_rpm, &params, depth)?;
            let width = trace
                .v_width_mm
                .map(|w| format!(", V-bit cut width {w:.4} mm"))
                .unwrap_or_default();
            println!(
                "{} rpm, feed {} mm/min, plunge {} mm/min, pass {} mm{}{}{}",
                c.rpm,
                c.feed_mm_min,
                c.plunge_mm_min,
                c.pass_depth_mm,
                if c.rpm_limited {
                    " (the spindle limits the speed)"
                } else {
                    ""
                },
                width,
                if matches!(t.kind, Kind::VBit | Kind::Engraver) {
                    " - only a starting point: what a V-bit or an engraver cuts with depends on the depth, try it on a scrap board"
                } else {
                    ""
                }
            );
            Ok(())
        }
        "vbit" => {
            let depth = flag(&mut args, "--depth")?;
            let angle = flag(&mut args, "--angle")?;
            let flat = flag(&mut args, "--flat")?;
            cli_args::finish(&args, 0)?;
            let depth = depth.ok_or("vbit needs --depth MM")?;
            // the angle and the flat that are not given are the ones of tools.cfg (60 degrees and 0.1 mm without one)
            let params = Settings::load(&data_dir())?.params;
            let angle = angle.unwrap_or(params.v_bit_angle_deg);
            let flat = flat.unwrap_or(params.v_bit_flat_mm);
            let width = v_bit_width_mm(depth, angle, flat)?;
            println!(
                "cut width {width:.4} mm = 2 x depth {depth} x tan(angle {angle} / 2) + flat {flat}"
            );
            Ok(())
        }
        other => Err(format!("{other}: unknown command (see help)")),
    }
}

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}
