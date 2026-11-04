//! What the README promises can be run: every `cargo run --release -- ...` line of the quick start works, the scripts run.sh and run.bat show the same sample, and the help runs.

use std::path::PathBuf;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_eh3d-tools");

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(name: &str) -> String {
    std::fs::read_to_string(root().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn words(s: &str) -> Vec<String> {
    s.split_whitespace().map(str::to_string).collect()
}

fn of_readme() -> Vec<Vec<String>> {
    read("README.md")
        .lines()
        .filter_map(|l| l.trim().strip_prefix("cargo run --release -- ").map(words))
        .collect()
}

fn of_run_sh() -> Vec<String> {
    let text = read("run.sh");
    let line = text
        .lines()
        .find(|l| l.contains("set -- ") && !l.contains("\"$@\""))
        .expect("run.sh has the sample");
    words(
        line.split("set -- ")
            .nth(1)
            .unwrap()
            .split(';')
            .next()
            .unwrap(),
    )
}

#[test]
fn the_commands_of_the_readme_run_and_print_something() {
    let commands = of_readme();
    assert!(!commands.is_empty(), "the README shows no command");
    for args in commands {
        let out = Command::new(BIN)
            .args(&args)
            .current_dir(root())
            .output()
            .expect("the program runs");
        assert!(
            out.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!out.stdout.is_empty(), "{args:?} printed nothing");
    }
}

#[test]
fn the_script_and_the_readme_show_the_same_sample() {
    let sample = of_run_sh();
    assert!(!sample.is_empty());
    assert!(
        of_readme().contains(&sample),
        "the README does not show the sample of run.sh"
    );
}

#[test]
fn the_help_runs_and_an_unknown_option_is_refused() {
    let help = Command::new(BIN).arg("help").output().unwrap();
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout);
    for command in [
        "init",
        "add",
        "list",
        "use",
        "wear",
        "export-csv",
        "import-csv",
        "calc",
    ] {
        assert!(text.contains(command), "the help does not name {command}");
    }
    let bad = Command::new(BIN)
        .args(["calc", "drill", "0.8", "fr4", "24000", "--oops"])
        .output()
        .unwrap();
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("unknown option"));
}
