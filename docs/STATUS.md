# EH3D-TOOL-MANAGER: status

## Current state

EH3D-TOOL-MANAGER is a working program on a computer: a library (`eh3d_tool_manager`), a command line (`eh3d-tools`) and a window (`eh3d-tools-gui`, built with `--features gui`). It keeps a crib of every kind of tool and of the sheets of stencil material, counts wear, calculates feeds and speeds and the width of the cut of a V-bit, checks that the tools and sheets a job needs are in the crib, and keeps the crib as a project on the server. With the feature `common-machining` it also reads the cutting profile of a tool in a material from the machining database of `EH3D-COMMON`; its figures were not tried on a machine. A seeded simulation of shifts of work runs the counters and the saves, including saves that fail. The suite of tests runs on Windows and Linux in the CI. The window has not been looked at on a screen and has no file dialog.

## Capabilities

The official state of each capability is in `REQUIREMENTS.md`. The register has 41 requirements: 30 implemented, 5 prototype, 6 planned and 0 validated. By progress: 30 done, 5 partial, 2 pending and 4 blocked. This page never states a state that `REQUIREMENTS.md` does not.

## Open questions

* Whether the crib stays a JSON file or becomes a database when several programs read it.
* Which programs read the crib directly instead of their own tool tables, and in what order.
* Whether the 2D profile of each tool belongs in the crib.
* Which tables the programs that cut a stencil read: the table of sheets (`kind` is always `sheet`) is written, and none of them reads it yet.

## What it needs from other repositories

* `EH3D-COMMON`: the client of the server, as a sibling folder at the version named in the workflows.
* The CAM programs of the suite, to read the crib instead of their own tool tables, and the stencil program to read the table of sheets.
* A release of `EH3D-COMMON` with the module `machining` in the workflows, so that the CI builds the feature `common-machining`; until then that feature is built and tested only on a computer that has it.
* A person with a machine, to check the feeds and the speeds.

## Known technical debt and omitted tests

* The feeds and speeds are starting points taken from typical values and were not checked on a machine.
* The window asks for the crib file and the CSV table as written paths: there is no file dialog.
* The sheets, the check and the cutting profile are on the command line only; the window does not show them.
* The drawing of the window is not covered by tests; only its logic is, and no person has looked at it on a screen.
* Tests that need a machine or a screen are not written: the register names each as `planned:` with the test that will show it.
