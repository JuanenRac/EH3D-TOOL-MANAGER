# Changelog

All notable changes to this project are documented here.

## [Unreleased]

- **The shared validation tools are updated:** the validator also runs the workflow and lessons checks of the suite with their latest rules, including a check to run before every push.
- **The CI validation checks the workflows and the lessons of past CI failures against a baseline:** `tools/workflow_hygiene.py` and `tools/lessons_lint.py` (copies of the shared tools) run in the validation, and only findings that are not in `tools/workflow_hygiene_baseline.json` and `tools/lessons_lint_baseline.json` fail it, so what exists today stays as it is and a new one is caught. `python tools/workflow_hygiene.py --root . --detail` and `python tools/lessons_lint.py --repo .` list the findings.
- A screenshot of the Tools tab in the README, taken with the published program and the example crib.
- **The files are read with a size limit and written whole:** the crib and the CSV table are read (every command of `eh3d-tools` that opens a crib file, `import-csv` for the table, and the window) only when they are not larger than the new key `max_file_mib` of `tools.cfg` (16 MiB by default, 1 to 1024, listed in `tools.cfg.example`, which a test keeps equal to the defaults); `tools.cfg` itself is read up to 1 MiB. A refusal starts with a code that does not change (`file_too_large`, `file_unreadable`, `file_not_text`) followed by the file name, and at most one byte more than the limit is ever read. The CSV export of the window now goes through a temporary file that is renamed over the target, as the crib already did (`files::write_atomic`, shared by both), so a cut or a full disk leaves the old file whole; a failure is `file_unwritable` and removes the temporary file, and a file is still replaced only when the person says so. Tests in `tests/files.rs`, `settings.rs` and `window.rs`.
- **The window opens on the page you choose:** `eh3d-tools-gui --tab tools|calc` (also together with `--shot`) opens the window on the table of the tools or on the calculator of feeds and speeds; a page that is not one of the two is an error that names the two, and without `--tab` the window opens on the table as before. The picture of the calculator is `--shot calc.png --tab calc`. The word is read by plain code with its tests.

## [0.0.7] - Picture mode for the window and a sample drawer of tools

- **The width of the cut of a V-bit by its angle:** `v_bit_width_mm` works out `2 x depth x tan(angle / 2) + flat` (a 60 degree bit at 0.5 mm: 0.5774 mm plus the flat of 0.1 mm = 0.6774 mm), with the angle from 10 to 170 degrees, the depth from 0 to 100 mm and the flat from 0 to 5 mm; every value out of its range is an error that names it. The command `eh3d-tools vbit --depth MM [--angle DEG] [--flat MM]` prints it.
- **Two new keys in `tools.cfg`:** `v_bit_angle_deg` (60 by default, 10 to 170) and `v_bit_flat_mm` (0.1 by default, 0 to 5), checked with the number of their line; they are the angle and the flat that `vbit` uses when the options are not given, and the flat of every width. `tools.cfg.example` lists them.
- **The feeds and speeds of a V-bit or an engraver at a depth:** `calc ... --depth MM` (and `feeds_speeds_traced_at` in the library) works the speed and the feed with the width of the cut at that depth as the diameter (at most the diameter of the tool, never under 0.01 mm), using the angle of the tool; a depth for another kind of tool is an error. Without a depth the numbers are the same as before.
- **The window shows the width of the cut:** the calculator has a field for the depth, shows the width of the cut in the result and the formula with its numbers in the eight languages, and the numbers of the calculation include the angle and the flat.
- **A picture mode for the window:** `eh3d-tools-gui --shot FILE.png` (built with `--features gui`) opens the window at the size of `--size WxH` (320 to 8192, 1180x760 by default), with `--theme forge|dark|light`, `--lang CODE` and the crib file of `--open FILE`, waits `--wait N` frames (0 to 3600, 6 by default), saves what it drew as a PNG file and closes by itself. An existing file is never replaced unless `--replace` is given, the window reads and keeps no settings in this mode (it starts from the default ones and from the crib it is given, or an empty one, never from the files of the person), the exit code is 0 when the file was written, 1 when it was not (also when the picture does not come back within 600 frames) and 2 for wrong arguments. The arguments are read by plain code with its tests, and a sample crib (`examples/crib.json`) is the one the pictures show.

## [0.0.6] - The window with the tool table, the feeds and speeds calculator and its configuration file

- **A window:** built with the `gui` feature (egui, the look the other programs of the suite share), program `eh3d-tools-gui`, which opens without a console on Windows. A table of the tools with filters (a search, the kind, the wear) and a sort, a form to add, change and remove a tool with the same checks and numbers as the command line (the number of a removed tool is never given again), the use of a job added to the counters of a tool, the crib file opened and saved, CSV import and export with the errors of the library, and a calculator of feeds and speeds that shows the formula of every number and the conditions that applied, with the numbers it works with editable. All of it in the eight languages; the state and the actions live in a module with tests and the screen only draws them.
- **Settings in `tools.cfg`:** the language, the size of the window, the crib file, the spindle speed the calculator starts with and every number the feeds and speeds are worked out with (the surface speed and the chip load of each material and the factors of the plunge and of the depth of a pass) are lines of a file instead of constants of the code; a value that makes no sense is an error that names its line, and `tools.cfg.example` lists every key with its default. The `calc` command reads the same file, and without it gives the numbers it always gave.
- **A tool can be replaced by its number in the library:** `Crib::update` makes the same checks as `add` and keeps the number and the counter of numbers; `Wear::fraction` gives the fraction of the limits that is used.
- **Release binaries workflow:** builds the command line and the window for Windows (64 and 32 bits), Linux and macOS, with their hashes and attestations; the CI also runs the lints with the `gui` feature.

## [0.0.5] - Stricter CSV, numbers that are never given twice and the shared validation job

- **The picture of the architecture counts the tests the repository has:** its legend said 34 and the suite has 71.
- **The validation job is the shared one of the documentation hub:** the checks of the manifest, the documents and the pictures run through the reusable workflow, fixed to a full commit of EH3D-DOCS whose actions are pinned by commit.
- **A number that was given is not given again:** the crib keeps `next_id`, so removing the last tool and adding another does not reuse its number; an id that is not a whole positive number is told as such.
- **The CSV is stricter and safer:** text after a closing quote is an error, a flute count that is too large has its own message, and a text that starts like a formula is written with a quote mark in front so that a spreadsheet shows it as text.
- **The material has a limit:** at most 80 characters and no control character.
- **`--holes` is a whole number** like the other counters, and an unknown option or an extra word is an error.
- **`calc` says when the result is only a starting point:** for a V-bit and an engraver, whose cut depends on the depth.

## [0.0.4] - Audit fixes, the new reading limits, tests and the application icon

- **Brand:** Add the application icon and the logos of the program.
- **Shared checks updated:** the validator also compares the badge and the Version row of the README with the manifest, fails when one of its checks cannot run, and the CHANGELOG step of the workflow reads the base branch from the environment.
- **BOM in the crib:** a crib file saved with a byte order mark is read.
- **Safer crib file:** the file is written through a temporary file and renamed, `import-csv` keeps the previous table as `.bak`, and the last tool number gives an error instead of wrapping round to 0.
- **The workflows are hardened:** every action is pinned by its commit with its version in a comment, and the checkouts keep no credentials.
- **More tests:** the kinds, the wear thresholds, the use that is recorded, the order of the tools to replace, and the JSON and CSV files that are damaged or odd.

## [0.0.3] - CI hardening and shared validator

- **Faster, bounded CI:** the Cargo build is cached, a run that a newer push supersedes is cancelled, every job has a time limit, and the Rust jobs live in their own workflow that is skipped when only documents or images change.
- **A fixed Rust toolchain:** `rust-toolchain.toml` names the compiler version (with rustfmt and clippy) that the CI installs, so a new stable release cannot turn the lints red without a change here.
- **Monthly dependency updates:** Dependabot opens one grouped pull request a month for the GitHub Actions and the Cargo dependencies.
- **A line for every change:** a pull request that changes code must add a line under `[Unreleased]` of this file (`tools/changelog_pr_check.py`; the label `no-changelog` skips it).
- **Shared checks updated:** the validator of the suite checks the text of the banner against the manifest and reads the files that git tracks.
- **Wear counters that are not whole numbers are refused:** reading a crib with a negative or textual `holes` or `max_holes`, or a `flutes` that is not a whole number, is an error that names the tool and the field instead of reading the default; the command line also refuses `--flutes 2.9` and `--max-holes -5` instead of cutting them.

## [0.0.2] - More tests and counters that cannot overflow

- **V-bits and engravers:** the feed and the depth of a pass are worked out from the diameter the tool works with (the point), like the speed, not from the shank, so a V-bit entered with its shank diameter no longer gets a feed sixteen times too high.
- **The banner of the README** shows the role and the technologies of this program (it showed the text of the shared library).
- **More tests:** the crib is checked at the exact limits of wear (90 % and 100 %), with names that are not ASCII, with values that overflow, with damaged CSV and JSON fields and through every command of the command line; a counter that would grow past what a number can hold is now refused instead of being saved as an unreadable value.

## [0.0.1] - The tool crib with wear counters, JSON and CSV, and starting feeds and speeds

- **The crib:** drills, end mills, V-bits, engravers, lasers and nozzles; each value is checked, a repeated name is refused and every tool gets a number.
- **The wear:** hours, metres and holes are added per job; good, replace soon (from 90 % of a limit) and replace; the worst tools are listed first.
- **Storage:** a JSON document and a CSV table, read with care (a quote that is never closed, a number that is not a number, a repeated number or name are errors that name the tool or the row).
- **Feeds and speeds:** surface speed and chip load for five materials, limited by the maximum speed of the spindle; a laser or a nozzle has no cutting feed.
- **The command line** `eh3d-tools` with `init`, `add`, `list`, `use`, `wear`, `export-csv`, `import-csv` and `calc`.
- **12 tests** run on a computer, with damaged input.
