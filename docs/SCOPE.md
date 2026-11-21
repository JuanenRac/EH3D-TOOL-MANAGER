# EH3D-TOOL-MANAGER: scope

Status: **host-test**. EH3D-TOOL-MANAGER is a library, a command line (`eh3d-tools`) and a window (`eh3d-tools-gui`) that keep the crib of cutting tools of the shop: what each tool is, how worn it is and what feed and speed it can be given. It has been tried on a computer only: the wear counters and the feeds and speeds have not been checked on a machine. This page says what it is for, what it will not do and where it sits among the other programs of the suite.

## Purpose

The tool crib shared by the CAM programs of the suite: drills, end mills, V-bits, engravers, lasers and nozzles with their diameter, flutes, length, tip angle and shank, a number for each tool that is never given again, a wear counter in hours, metres and holes with a state (good, to replace soon, to replace), and a calculator of surface speed and chip load limited by the spindle.

## What it does

* Keeps, beside the tools, the sheets of stencil material (material, thickness, the reduction of each aperture, the kerf of the beam and the least aspect ratio and area ratio of an aperture) in the JSON document and in a table of their own; the table of the tools does not change.
* Checks that the tools (by name and diameter) and the sheets (by name and thickness) that a job needs are in the crib and that the crib agrees with itself, and ends with the code 2 when it does not.
* Reads the cutting profile of a tool in a material from the machining database of `EH3D-COMMON`, only in a build with the feature `common-machining`, and says first that its figures were not tried on a machine.
* Keeps the crib as a JSON document or a CSV table for a spreadsheet, checks every value, refuses a repeated name and a field that makes no sense with its name, and names the row in every error of a CSV.
* Numbers each tool and never gives the number of a removed tool again; writes a CSV so that a spreadsheet does not run a text that starts like a formula.
* Adds the use of a tool (hours, metres, holes) after a job, says whether it is good, to replace soon (from 90 %) or to replace, and lists the worst first.
* Works the feed and the speed of a tool for FR4, aluminium, acrylic, wood and brass, limited by the spindle, and the width of the cut of a V-bit at a depth.
* Reads files only within a size limit and writes them whole through a temporary file, keeping the previous crib beside it and restoring it on request.
* Shows the crib in a window (built with `--features gui`), in eight languages, with a form, filters, a search, CSV import and export and the calculator; and keeps the crib as a project with revisions on the EH3D server when asked.

## What it will not do

* No decision about the limits of a machine, its spindle or its emergency stop (see below).
* No reading of the crib by the other CAM programs: only the CSV table of the tools is read by ISOCAM today, and none reads the table of sheets yet (see `NOT_YET.md`).
* No database: the crib is a JSON file.
* No measuring of a real tool: the values are what a person types.
* No claim that the feeds and speeds are right for a machine: they are starting points taken from typical values.

## Where it sits

* **It reads:** the crib as a JSON document or a CSV table, its settings file (`tools.cfg`) and, when asked, the server of the suite.
* **It writes:** the crib (whole, through a temporary file, with the previous crib kept beside it), a CSV table and reports; never over a file it could not read back.
* **It is read by:** ISOCAM through the CSV table, the other programs of the suite through those files or through the server; programs exchange work through files or through the server, never by depending on each other.
* **It depends on:** `EH3D-COMMON` (the client of the server), built next to it as a sibling folder.

## What it never decides

What speed and what feed a machine and a material can take, and when a tool must be replaced, belong to the person at the machine: the numbers here are starting points and a wear counter counts what it was told. Nothing here replaces the limits of a machine, its spindle limits or its emergency stop.

The requirements of the program, with the state of each and the test that shows it, are in `REQUIREMENTS.md`; the state in a paragraph is in `STATUS.md`.
