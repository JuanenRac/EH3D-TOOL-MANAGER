# EH3D-TOOL-MANAGER: requirements

This register is the single source of the state of each capability: `STATUS.md` and the README files cite it and never state a state that it does not. A state is one of `planned`, `prototype`, `implemented` and `validated`. `validated` means that a person has run the capability on a real machine and the evidence of the manifest says `bench` or `field`; nothing here is validated, because the wear and the feeds and speeds have not been checked on a real machine.

The register has 42 requirements: 31 implemented, 5 prototype, 6 planned and 0 validated. By progress: 31 done, 5 partial, 2 pending and 4 blocked. High coverage is not proof of correctness, so each row says how it is checked: the test that names it, and what it shows.

## How to read the register

* **ID:** `TMGR-NNN`, never reused and never renumbered; a new capability takes the next number.
* **State:** `implemented` is code that is complete and tested on a computer, `prototype` is part of the capability, `planned` is not written.
* **Progress:** `done`, `partial`, `pending` (can be written without hardware) or `blocked` (waits for hardware or for another program of the suite).
* **Evidence:** `simulator` when the test runs against a simulated shift of work or a server in memory, `host-test` when it checks a calculation, a reader or a screen-less piece, `none` when nothing is written yet.
* **Test:** the acceptance criterion: `file::name` is the test `name` of `tests/file.rs` (`src/file::name` is a test inside `src/file.rs`), which must exist. For a capability that is not written the cell says `planned:` and the name of the test that will show it; the day that test exists, the row must change.
* **Limits:** what is not done or not proven; a row that is `partial` or `blocked` always says it.

The check is `tests/reqs.rs`: it reads this table, finds each test and fails when a state, a test or a count does not agree.

## The crib

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| TMGR-001 | The crib numbers its tools and refuses a repeated name. | implemented | done | host-test | `crib::the_crib_numbers_its_tools_and_refuses_a_repeated_name` | - |
| TMGR-002 | A tool that makes no sense is refused with the name of the field. | implemented | done | host-test | `crib::a_tool_that_makes_no_sense_is_refused_with_the_field` | - |
| TMGR-003 | Every kind of tool has its own key that reads back in any case. | implemented | done | host-test | `more::every_kind_has_its_own_key_that_reads_back_in_any_case_and_with_blanks` | - |
| TMGR-004 | A V-bit and an engraver need their angle and a drill does not. | implemented | done | host-test | `more::a_v_bit_and_an_engraver_need_their_angle_and_a_drill_does_not` | - |
| TMGR-005 | The name has one to eighty characters and no control character. | implemented | done | host-test | `more::the_name_has_one_to_eighty_characters_counted_as_characters_and_no_control_character` | - |
| TMGR-006 | A JSON crib round-trips and a damaged one is an error that says which tool. | implemented | done | host-test | `crib::a_damaged_json_is_an_error_that_says_which_tool` | - |
| TMGR-007 | A CSV round-trips names with commas and quotes, and a damaged one names the row. | implemented | done | host-test | `crib::a_damaged_csv_names_the_row` | - |
| TMGR-008 | The number of a removed tool is not given again. | implemented | done | host-test | `src/window::the_number_of_a_removed_tool_is_not_given_again` | - |

## Wear

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| TMGR-009 | The wear goes from good to soon to replace, from ninety percent. | implemented | done | host-test | `more::the_wear_is_good_below_ninety_percent_soon_from_there_and_replace_at_the_limit` | - |
| TMGR-010 | A tool with no limits never wears out and the worst of hours and holes decides. | implemented | done | host-test | `more::a_tool_with_no_limits_never_wears_out_and_the_worst_of_hours_and_holes_decides` | - |
| TMGR-011 | Recording a use adds to the counters and refuses what makes no sense. | implemented | done | host-test | `more::recording_a_use_adds_to_the_counters_and_says_the_wear_and_refuses_what_makes_no_sense` | Counts what it is told; no machine reports the use. |
| TMGR-012 | The holes counter does not wrap around. | implemented | done | host-test | `more::the_holes_counter_does_not_wrap_around_and_the_hours_cannot_grow_past_a_number` | - |
| TMGR-013 | A seeded simulation of shifts adds up the counters and replaces the tools that reach their limit. | prototype | partial | simulator | `src/lib::a_shift_adds_up_the_counters_replaces_the_tools_that_reach_their_limit_and_saves_what_it_holds` | A simulation made for the tests, not a shop. |

## Files and calculation

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| TMGR-014 | A file within the limit is read and a bigger one is refused with its code. | implemented | done | host-test | `files::a_file_within_the_limit_is_read_and_a_bigger_one_is_refused_with_its_code` | - |
| TMGR-015 | A file is replaced whole and no temporary file is left. | implemented | done | host-test | `files::a_file_is_replaced_whole_and_no_temporary_file_is_left` | - |
| TMGR-016 | A save keeps the previous crib beside it, and a restore puts it back and keeps the file it replaced. | implemented | done | host-test | `files::a_restore_puts_back_the_backup_on_request_keeps_the_replaced_file_and_refuses_a_bad_backup` | - |
| TMGR-017 | A crib file cut at any place is refused and the last good copy brings it back. | implemented | done | host-test | `src/lib::a_crib_file_cut_at_any_place_is_refused_and_the_last_good_copy_brings_it_back` | - |
| TMGR-018 | The width of the cut of a V-bit is twice the depth times the tangent of half the angle plus the flat. | implemented | done | host-test | `src/calc::the_width_of_a_v_bit_is_twice_the_depth_times_the_tangent_of_half_the_angle_plus_the_flat` | - |
| TMGR-019 | The `vbit` command refuses what makes no sense and says which value. | implemented | done | host-test | `vbit::the_vbit_command_refuses_what_makes_no_sense_and_says_which_value` | - |
| TMGR-020 | The calculator holds the numbers behind the cut and refuses a bad number. | implemented | done | host-test | `src/calc::the_trace_holds_the_numbers_behind_the_cut` | Feeds and speeds are starting points taken from typical values. |
| TMGR-021 | A bad settings value names its line. | implemented | done | host-test | `src/settings::a_value_that_makes_no_sense_names_its_line` | - |

## Server and window

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| TMGR-022 | The first push makes the project on the server and the next ones add revisions; a project that moved on refuses the push, and so does a copy that does not start from its last revision. | prototype | partial | simulator | `src/sync::a_project_that_moved_on_refuses_the_push_and_says_so` | Tried against a server in memory, not against a real server. |
| TMGR-023 | Nothing is sent to the server without an address and a token. | implemented | done | host-test | `src/server::nothing_is_sent_without_a_server_or_a_token` | - |
| TMGR-024 | The window makes the tool the library would accept and names the field that is wrong. | prototype | partial | host-test | `src/window::the_form_makes_the_tool_the_library_would_accept_and_names_the_field_that_is_wrong` | The logic is tested without a screen; no person has looked at the drawing. |
| TMGR-025 | The window opens the sample crib with tools of every kind. | prototype | partial | host-test | `shot_sample::the_sample_crib_loads_with_tools_of_every_kind` | The logic is tested without a screen; no person has looked at the drawing. |
| TMGR-026 | Every sentence of the window has a text in each of the eight languages, and the Portuguese is not the Brazilian. | implemented | done | host-test | `src/lang::the_portuguese_is_not_the_brazilian` | - |

## Not written yet

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| TMGR-027 | The CAM programs read the crib instead of their own tool tables. | planned | blocked | none | `planned:cam_reads_crib::the_cam_programs_take_their_tools_from_the_crib` | Needs the other programs of the suite to change. |
| TMGR-028 | The 2D profile of each tool is kept in the crib. | planned | pending | none | `planned:profile::a_tool_keeps_its_2d_profile` | The crib holds sizes only. |
| TMGR-029 | The window has a file dialog for the crib and the CSV table. | planned | pending | none | `planned:file_dialog::the_window_opens_a_crib_with_a_dialog` | The window asks for a written path today. |
| TMGR-030 | The feeds and speeds are checked on a machine. | planned | blocked | none | `planned:real_cuts::the_feeds_and_speeds_cut_well_on_a_machine` | Needs a machine and a person. |
| TMGR-031 | The window is looked at on a screen of each system. | planned | blocked | none | `planned:window_screen::the_window_matches_the_reference_picture` | Needs a person at a screen. |

## Sheets, the check and the cutting profile

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| TMGR-032 | A sheet of stencil material is numbered with a number of its own, a repeated name is refused and a value that makes no sense is refused with the name of the field. | implemented | done | host-test | `sheets::a_sheet_is_numbered_and_refuses_a_repeated_name_and_what_makes_no_sense` | - |
| TMGR-033 | A crib with sheets round-trips in JSON as format 2, keeping the keys it does not know, and a crib with no sheet is written as before. | implemented | done | host-test | `sheets::a_sheet_round_trips_in_json_and_a_crib_with_no_sheet_is_written_as_before` | - |
| TMGR-034 | The table of the tools keeps its header and holds no sheet, and the sheets have a table of their own whose kind is always sheet. | implemented | done | host-test | `sheets::the_table_of_tools_does_not_change_and_the_sheets_have_a_table_of_their_own` | - |
| TMGR-035 | An aperture is checked against the least aspect ratio and area ratio of its sheet and against the reduction; a limit of 0 is not applied. | implemented | done | host-test | `sheets::an_aperture_is_checked_against_the_ratios_and_the_reduction` | The limits are what the person wrote; the program has no default for them. |
| TMGR-036 | The check finds every tool and sheet that a job needs by name and size, and says what is missing. | implemented | done | host-test | `check::a_check_finds_the_tools_and_sheets_a_job_needs_and_names_what_is_missing` | - |
| TMGR-037 | The check command ends with the code 2 when it is not passed and with 1 when it cannot run. | implemented | done | host-test | `check::the_command_ends_with_code_2_when_the_check_is_not_passed_and_1_when_it_cannot_run` | - |
| TMGR-038 | A table with more rows than the limit is refused and a broken quote names its row. | implemented | done | host-test | `sheets::a_table_with_more_rows_than_the_limit_is_refused_and_a_broken_quote_names_its_row` | - |
| TMGR-039 | A fuzz with a fixed seed of the readers never panics, and what is read is written back the same. | implemented | done | host-test | `fuzz::a_seeded_fuzz_of_the_readers_never_panics_and_what_reads_writes_back_the_same` | - |
| TMGR-040 | The cutting profile of a tool in a material is read from the machining database of EH3D-COMMON and says first that its figures were not tried on a machine. | prototype | partial | host-test | `profile::a_profile_comes_from_the_database_and_says_when_it_was_not_verified` | Built only with the feature common-machining, which the CI cannot build until it checks out a release of EH3D-COMMON that has the module; the figures were not tried on a machine. |
| TMGR-041 | The CAM programs take the sheet and its kerf from the crib. | planned | blocked | none | `planned:cam_reads_sheets::stencilcam_takes_its_sheet_and_kerf_from_the_crib` | Needs the programs that cut a stencil to read the table of sheets. |

## Several programs on one crib

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| TMGR-042 | A crib is changed by one program at a time, no change is lost and a program that dies leaves no lock behind. | implemented | done | host-test | `files::programs_that_add_to_one_crib_at_the_same_time_lose_no_tool` | The lock is the one of the system; on a shared folder it is only as good as the locks of that folder, and a program that does not ask for it is not held back. |
