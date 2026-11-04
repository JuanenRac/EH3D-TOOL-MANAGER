// EH3D-TOOL-MANAGER - command line.
// Copyright (C) 2026 JuanenRac (Electro Hobby 3D). GPL-3.0-or-later.
#![forbid(unsafe_code)]

mod cli_args;

use eh3d_tool_manager::calc::{feeds_speeds, Material};
use eh3d_tool_manager::model::{Crib, Kind, Tool, Wear};
use eh3d_tool_manager::{csv, store};
use std::env;
use std::path::Path;
use std::process::ExitCode;

const HELP: &str = "eh3d-tools: the tool crib of the EH3D Fabrication Suite

  eh3d-tools init FILE.json
  eh3d-tools add FILE.json NAME KIND DIAMETER_MM [--flutes N] [--angle DEG] [--max-hours H] [--max-holes N]
  eh3d-tools list FILE.json
  eh3d-tools use FILE.json ID [--hours H] [--meters M] [--holes N]
  eh3d-tools wear FILE.json                 the tools that are worn or about to be
  eh3d-tools export-csv FILE.json
  eh3d-tools import-csv FILE.json TABLE.csv
  eh3d-tools calc KIND DIAMETER_MM MATERIAL MAX_RPM [--flutes N] [--angle DEG]

KIND: drill, end-mill, v-bit, engraver, laser, nozzle. MATERIAL: fr4, aluminium, acrylic, wood, brass.
The feeds and speeds are starting points to try on a scrap board, not a promise.";

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

fn describe(w: Wear) -> String {
    match w {
        Wear::Good(f) => format!("good ({:.0} %)", f * 100.0),
        Wear::Soon(f) => format!("replace soon ({:.0} %)", f * 100.0),
        Wear::Replace => "REPLACE".to_string(),
    }
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
            let mut crib = store::load(file)?;
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
            need(&args, 1, "a file")?;
            cli_args::finish(&args, 1)?;
            for t in &store::load(Path::new(&args[0]))?.tools {
                println!(
                    "{:>3}  {:<24} {:<9} {:>6.3} mm  {:>2} flutes  {:>7.1} h  {:>7} holes  {}",
                    t.id,
                    t.name,
                    t.kind.key(),
                    t.diameter_mm,
                    t.flutes,
                    t.hours,
                    t.holes,
                    describe(t.wear())
                );
            }
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
            let mut crib = store::load(file)?;
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
            let crib = store::load(Path::new(&args[0]))?;
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
            print!("{}", csv::to_csv(&store::load(Path::new(&args[0]))?));
            Ok(())
        }
        "import-csv" => {
            need(&args, 2, "a crib file and a CSV table")?;
            cli_args::finish(&args, 2)?;
            let text =
                std::fs::read_to_string(&args[1]).map_err(|e| format!("{}: {e}", args[1]))?;
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
        "calc" => {
            let flutes = whole_flag(&mut args, "--flutes")?;
            let angle = flag(&mut args, "--angle")?;
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
            let c = feeds_speeds(&t, Material::parse(&args[2])?, max_rpm)?;
            println!(
                "{} rpm, feed {} mm/min, plunge {} mm/min, pass {} mm{}{}",
                c.rpm,
                c.feed_mm_min,
                c.plunge_mm_min,
                c.pass_depth_mm,
                if c.rpm_limited {
                    " (the spindle limits the speed)"
                } else {
                    ""
                },
                if matches!(t.kind, Kind::VBit | Kind::Engraver) {
                    " - only a starting point: what a V-bit or an engraver cuts with depends on the depth, try it on a scrap board"
                } else {
                    ""
                }
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
