// EH3D-TOOL-MANAGER - the tool crib of the EH3D Fabrication Suite.
// Copyright (C) 2026 JuanenRac (Electro Hobby 3D). GPL-3.0-or-later.
#![forbid(unsafe_code)]
//! Tools (drills, end mills, V-bits, engravers, lasers, nozzles) with their geometry and their wear, stored as JSON or CSV, and the feeds and speeds to start from. With the `gui`
//! feature it also has the window (`window` holds its state, `gui` draws it).

pub mod calc;
pub mod csv;
pub mod files;
#[cfg(feature = "gui")]
pub mod gui;
pub mod lang;
pub mod model;
pub mod settings;
pub mod shot;
pub mod store;
pub mod window;

/// The simulation of a shift on a machine: a seeded list of job reports (the hours, metres and holes that a job used with a tool), played into the crib the way the
/// machine would report them, with the crib saved after each job and the failures that can happen (a report that is not a number, a tool that was removed, a save that
/// cannot be written, a crib file cut in the middle) and the recovery from each. Nothing here has been connected to a machine.
pub mod sim {
    use crate::model::{Crib, Wear};
    use crate::store;
    use std::path::Path;

    /// A small deterministic generator (xorshift64*). It reads no clock.
    #[derive(Debug, Clone)]
    pub struct SimRng(u64);

    impl SimRng {
        /// A generator for the seed (zero is replaced, because xorshift cannot start from zero).
        pub fn new(seed: u64) -> SimRng {
            SimRng(if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            })
        }

        /// A number below `n` (`n` must be above zero).
        pub fn below(&mut self, n: u64) -> u64 {
            let mut x = self.0;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.0 = x;
            x.wrapping_mul(0x2545_F491_4F6C_DD1D) % n
        }
    }

    /// What the machine reports after a job.
    #[derive(Debug, Clone, PartialEq)]
    pub struct JobReport {
        /// The number of the tool in the crib.
        pub tool: u32,
        /// Hours the tool ran.
        pub hours: f64,
        /// Metres it travelled.
        pub meters: f64,
        /// Holes it made.
        pub holes: u64,
    }

    /// `count` reports for the tools, from the seed: 3 to 30 minutes, 1 to 40 metres and up to 150 holes each.
    pub fn jobs(seed: u64, tools: &[u32], count: usize) -> Vec<JobReport> {
        let mut rng = SimRng::new(seed);
        (0..count)
            .map(|_| JobReport {
                tool: tools[rng.below(tools.len() as u64) as usize],
                hours: (3 + rng.below(28)) as f64 / 60.0,
                meters: (1 + rng.below(40)) as f64,
                holes: rng.below(151),
            })
            .collect()
    }

    /// How a shift went.
    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct ShiftReport {
        /// Reports added to the counters.
        pub applied: usize,
        /// Reports the crib refused (not a number, a negative number, a tool that is not in the crib).
        pub refused: usize,
        /// Saves that could not be written.
        pub save_failures: usize,
        /// The numbers of the tools that reached their limit and were replaced by a new one (counters at zero), in order.
        pub replaced: Vec<u32>,
    }

    /// Play the reports into the crib. After each one the crib is saved to `file`; every `fail_save_every`-th save is made to a place that cannot be written instead, and
    /// counts as a failure. A tool that reaches its limit is replaced at once by a new one (the same tool with its counters at zero). With `final_save` the crib is saved to
    /// `file` at the end, which is how a failed save is recovered.
    pub fn run_shift(
        crib: &mut Crib,
        file: &Path,
        reports: &[JobReport],
        fail_save_every: Option<usize>,
        final_save: bool,
    ) -> ShiftReport {
        let nowhere = file.join("no-such-folder").join("crib.json");
        let mut out = ShiftReport::default();
        for (index, report) in reports.iter().enumerate() {
            match crib.record_use(report.tool, report.hours, report.meters, report.holes) {
                Ok(wear) => {
                    out.applied += 1;
                    if wear == Wear::Replace && crib.replace_tool(report.tool).is_ok() {
                        out.replaced.push(report.tool);
                    }
                }
                Err(_) => out.refused += 1,
            }
            let target = match fail_save_every {
                Some(k) if (index + 1) % k == 0 => nowhere.as_path(),
                _ => file,
            };
            if store::save(crib, target).is_err() {
                out.save_failures += 1;
            }
        }
        if final_save && store::save(crib, file).is_err() {
            out.save_failures += 1;
        }
        out
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::model::{Kind, Tool};
        use std::fs;
        use std::path::PathBuf;
        use std::sync::atomic::{AtomicUsize, Ordering};

        /// A folder of its own for each test (the tests run side by side), removed at the end.
        struct Dir(PathBuf);

        impl Dir {
            fn new() -> Dir {
                static NEXT: AtomicUsize = AtomicUsize::new(0);
                let n = NEXT.fetch_add(1, Ordering::SeqCst);
                let path =
                    std::env::temp_dir().join(format!("eh3d-tools-sim-{}-{n}", std::process::id()));
                fs::create_dir_all(&path).unwrap();
                Dir(path)
            }
            fn file(&self) -> PathBuf {
                self.0.join("crib.json")
            }
        }

        impl Drop for Dir {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }

        fn crib() -> (Crib, Vec<u32>) {
            let mut crib = Crib::default();
            let mut ids = Vec::new();
            for (name, kind, hours, holes) in [
                ("drill 0.8", Kind::Drill, 1.5, 600),
                ("v-bit 30", Kind::VBit, 2.0, 0),
                ("end mill 1.0", Kind::EndMill, 0.0, 400),
            ] {
                let mut tool = Tool::new(name, kind, 1.0);
                if kind == Kind::VBit {
                    tool.tip_angle_deg = Some(30.0);
                }
                tool.max_hours = hours;
                tool.max_holes = holes;
                ids.push(crib.add(tool).unwrap());
            }
            (crib, ids)
        }

        /// Two cribs hold the same tools and counters; the hours and metres may differ in the last digit, because the file keeps decimals that way.
        fn assert_same(loaded: &Crib, kept: &Crib) {
            assert_eq!(loaded.tools.len(), kept.tools.len());
            for (a, b) in loaded.tools.iter().zip(&kept.tools) {
                assert_eq!((a.id, &a.name, a.holes), (b.id, &b.name, b.holes));
                assert!((a.hours - b.hours).abs() < 1e-9 && (a.meters - b.meters).abs() < 1e-9);
            }
        }

        fn counters(crib: &Crib) -> Vec<(f64, f64, u64)> {
            crib.tools
                .iter()
                .map(|t| (t.hours, t.meters, t.holes))
                .collect()
        }

        #[test]
        fn the_same_seed_gives_the_same_reports_and_another_seed_different_ones() {
            let (_, ids) = crib();
            assert_eq!(jobs(7, &ids, 50), jobs(7, &ids, 50));
            assert_ne!(jobs(7, &ids, 50), jobs(8, &ids, 50));
            assert!(jobs(7, &ids, 50).iter().all(|j| ids.contains(&j.tool)));
        }

        #[test]
        fn a_shift_adds_up_the_counters_replaces_the_tools_that_reach_their_limit_and_saves_what_it_holds(
        ) {
            let dir = Dir::new();
            let (mut crib, ids) = crib();
            let reports = jobs(3, &ids, 120);
            let out = run_shift(&mut crib, &dir.file(), &reports, None, false);
            assert_eq!(out.applied, 120);
            assert_eq!(out.refused + out.save_failures, 0);
            assert!(!out.replaced.is_empty(), "no tool reached its limit");
            assert!(
                crib.to_replace()
                    .iter()
                    .all(|(_, wear)| !matches!(wear, Wear::Replace)),
                "a tool at its limit was left in"
            );
            let loaded = store::load(&dir.file()).unwrap();
            assert_same(&loaded, &crib);
        }

        #[test]
        fn a_report_that_is_not_a_number_or_for_a_tool_that_is_gone_is_refused_and_changes_nothing()
        {
            let dir = Dir::new();
            let (mut crib, ids) = crib();
            let before = counters(&crib);
            let bad = |tool, hours, meters| JobReport {
                tool,
                hours,
                meters,
                holes: 1,
            };
            let reports = vec![
                bad(ids[0], f64::NAN, 1.0),
                bad(ids[0], -1.0, 1.0),
                bad(ids[0], 1.0, f64::INFINITY),
                bad(999, 1.0, 1.0),
            ];
            let out = run_shift(&mut crib, &dir.file(), &reports, None, false);
            assert_eq!((out.applied, out.refused), (0, 4));
            assert_eq!(counters(&crib), before);
            crib.remove(ids[1]).unwrap();
            let gone = jobs(1, &[ids[1]], 5);
            let out = run_shift(&mut crib, &dir.file(), &gone, None, false);
            assert_eq!((out.applied, out.refused), (0, 5));
        }

        #[test]
        fn saves_that_cannot_be_written_are_counted_and_the_final_save_recovers_the_file() {
            let dir = Dir::new();
            let (mut crib, ids) = crib();
            store::save(&crib, &dir.file()).unwrap();
            let start = store::to_json(&crib);
            let reports = jobs(5, &ids, 30);
            let out = run_shift(&mut crib, &dir.file(), &reports, Some(1), false);
            assert_eq!(out.save_failures, 30);
            assert_eq!(fs::read_to_string(dir.file()).unwrap(), start);
            assert_ne!(store::to_json(&crib), start);
            store::save(&crib, &dir.file()).unwrap();
            assert_same(&store::load(&dir.file()).unwrap(), &crib);
            let (mut again, ids) = self::crib();
            let out = run_shift(&mut again, &dir.file(), &jobs(5, &ids, 30), Some(4), true);
            assert_eq!(out.save_failures, 30 / 4);
            assert_same(&store::load(&dir.file()).unwrap(), &again);
        }

        #[test]
        fn a_crib_file_cut_at_any_place_is_refused_and_the_last_good_copy_brings_it_back() {
            let dir = Dir::new();
            let (mut crib, ids) = crib();
            run_shift(&mut crib, &dir.file(), &jobs(9, &ids, 20), None, true);
            let good = fs::read_to_string(dir.file()).unwrap();
            let whole = good.trim_end().len();
            for cut in (0..whole).step_by(7) {
                fs::write(dir.file(), &good.as_bytes()[..cut]).unwrap();
                assert!(store::load(&dir.file()).is_err(), "cut at {cut}");
            }
            fs::write(dir.file(), &good).unwrap();
            assert_same(&store::load(&dir.file()).unwrap(), &crib);
        }

        #[test]
        fn a_seeded_campaign_repeats_exactly_and_the_counters_never_go_down_between_replacements() {
            let run = |seed: u64| {
                let dir = Dir::new();
                let (mut crib, ids) = crib();
                let reports = jobs(seed, &ids, 80);
                let mut last = counters(&crib);
                for report in &reports {
                    let one = run_shift(
                        &mut crib,
                        &dir.file(),
                        std::slice::from_ref(report),
                        None,
                        false,
                    );
                    let now = counters(&crib);
                    if one.replaced.is_empty() {
                        for (a, b) in last.iter().zip(&now) {
                            assert!(b.0 >= a.0 && b.1 >= a.1 && b.2 >= a.2, "seed {seed}");
                        }
                    }
                    last = now;
                }
                store::to_json(&crib)
            };
            for seed in 1..=15 {
                assert_eq!(run(seed), run(seed), "seed {seed}");
            }
            assert_ne!(run(1), run(2));
        }

        #[test]
        fn the_generator_never_stalls_on_a_zero_seed() {
            let mut rng = SimRng::new(0);
            let drawn: Vec<u64> = (0..8).map(|_| rng.below(1_000_000)).collect();
            assert!(drawn.windows(2).any(|w| w[0] != w[1]));
        }
    }
}
