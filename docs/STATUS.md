# EH3D-TOOL-MANAGER: status

## Current state

EH3D-TOOL-MANAGER is a working program on a computer: a library (`eh3d_tool_manager`), a command line (`eh3d-tools`) and a window (`eh3d-tools-gui`, built with `--features gui`). It keeps a crib of every kind of tool, counts wear, calculates feeds and speeds and the width of the cut of a V-bit, and keeps the crib as a project on the server. A seeded simulation of shifts of work runs the counters and the saves, including saves that fail. The suite of tests runs on Windows and Linux in the CI. The window has not been looked at on a screen and has no file dialog.

## Capabilities

The official state of each capability is in `REQUIREMENTS.md`. The register has 31 requirements: 22 implemented, 4 prototype, 5 planned and 0 validated. By progress: 22 done, 4 partial, 2 pending and 3 blocked. This page never states a state that `REQUIREMENTS.md` does not.

## Open questions

* Whether the crib stays a JSON file or becomes a database when several programs read it.
* Which programs read the crib directly instead of their own tool tables, and in what order.
* Whether the 2D profile of each tool belongs in the crib.

## What it needs from other repositories

* `EH3D-COMMON`: the client of the server, as a sibling folder at the version named in the workflows.
* The CAM programs of the suite, to read the crib instead of their own tool tables.
* A person with a machine, to check the feeds and the speeds.

## Known technical debt and omitted tests

* The feeds and speeds are starting points taken from typical values and were not checked on a machine.
* The window asks for the crib file and the CSV table as written paths: there is no file dialog.
* The drawing of the window is not covered by tests; only its logic is, and no person has looked at it on a screen.
* Tests that need a machine or a screen are not written: the register names each as `planned:` with the test that will show it.
