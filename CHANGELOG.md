# Changelog

All notable changes to this project are documented here.

## [Unreleased]
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
