// EH3D-TOOL-MANAGER - the window, as a program of its own.
// Copyright (C) 2026 JuanenRac (Electro Hobby 3D). GPL-3.0-or-later.
//! `eh3d-tools-gui` opens the window. It is built with `--features gui`. On Windows it is a program of the graphical subsystem, so no console window opens behind it; for that
//! reason it prints nothing, and an error that keeps the window from opening is written to `last_error.txt` in the folder of the program. The command line (`eh3d-tools`) keeps
//! its console, so its output and its exit codes work in a terminal and in a script.
#![forbid(unsafe_code)]
#![cfg_attr(windows, windows_subsystem = "windows")]

use eh3d_tool_manager::settings::data_dir;
use std::process::ExitCode;

fn main() -> ExitCode {
    let dir = data_dir();
    match eh3d_tool_manager::gui::run(dir.clone()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // there is no console to print to: the reason goes to a file the person can open
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(
                dir.join("last_error.txt"),
                format!("the window could not open: {e}\n"),
            );
            ExitCode::from(1)
        }
    }
}
