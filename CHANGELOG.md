# Changelog

All notable changes to this project are documented here.

## [Unreleased]

- **V-bits and engravers:** the feed and the depth of a pass are worked out from the diameter the tool works with (the point), like the speed, not from the shank, so a V-bit entered with its shank diameter no longer gets a feed sixteen times too high.

## [0.0.1] - The tool crib with wear counters, JSON and CSV, and starting feeds and speeds

- **The crib:** drills, end mills, V-bits, engravers, lasers and nozzles; each value is checked, a repeated name is refused and every tool gets a number.
- **The wear:** hours, metres and holes are added per job; good, replace soon (from 90 % of a limit) and replace; the worst tools are listed first.
- **Storage:** a JSON document and a CSV table, read with care (a quote that is never closed, a number that is not a number, a repeated number or name are errors that name the tool or the row).
- **Feeds and speeds:** surface speed and chip load for five materials, limited by the maximum speed of the spindle; a laser or a nozzle has no cutting feed.
- **The command line** `eh3d-tools` with `init`, `add`, `list`, `use`, `wear`, `export-csv`, `import-csv` and `calc`.
- **12 tests** run on a computer, with damaged input.
