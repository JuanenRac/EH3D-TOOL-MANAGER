# Changelog

All notable changes to this project are documented here.

## [Unreleased]

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
