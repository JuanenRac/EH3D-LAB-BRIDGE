# EH3D-LAB-BRIDGE: requirements

This register is the single source of the state of each capability: `STATUS.md` and the README files cite it and never state a state that it does not. A state is one of `planned`, `prototype`, `implemented` and `validated`. `validated` means that a person has run the capability on a real machine and the evidence of the manifest says `bench` or `field`; nothing here is validated, because nothing of this program has been run with a real instrument or a real bench.

The register has 31 requirements: 25 implemented, 4 prototype, 2 planned and 0 validated. By progress: 25 done, 4 partial, 1 pending and 1 blocked. High coverage is not proof of correctness, so each row says how it is checked: the test that names it, and what it shows.

## How to read the register

* **ID:** `LBRI-NNN`, never reused and never renumbered; a new capability takes the next number.
* **State:** `implemented` is code that is complete and tested on a computer, `prototype` is part of the capability, `planned` is not written.
* **Progress:** `done`, `partial`, `pending` (can be written without hardware) or `blocked` (waits for hardware or for another program of the suite).
* **Evidence:** `simulator` when the test runs against a simulated bench whose exported file is damaged on purpose, `host-test` when it checks a calculation, a reader or a screen-less piece, `none` when nothing is written yet.
* **Test:** the acceptance criterion: `file::name` is the test `name` of `tests/file.rs` (`src/file::name` is a test inside `src/file.rs`), which must exist. For a capability that is not written the cell says `planned:` and the name of the test that will show it; the day that test exists, the row must change.
* **Limits:** what is not done or not proven; a row that is `partial` or `blocked` always says it.

The check is `tests/reqs.rs`: it reads this table, finds each test and fails when a state, a test or a count does not agree.

## Records and comparison

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| LBRI-001 | Import one defined measurement file format with its units. | prototype | partial | host-test | `src/curve::reads_values_and_units_exactly_and_writes_them_back` | One defined CSV format with the unit of each column in the header; the files of real instruments are not read. |
| LBRI-002 | Store the configuration of each test so that it can be repeated. | prototype | partial | host-test | `src/record::a_complete_record_passes_and_repeats_from_its_json` | The record keeps the instrument, the conditions, the quantity and the hash of the data, so a person can repeat the test; the settings of the instrument itself are free text. |
| LBRI-003 | Compare a measured and a simulated curve with a stated tolerance. | prototype | partial | host-test | `src/compare::a_known_offset_is_recovered` | Linear interpolation of the simulated curve at the measured points, checked with synthetic curves with a known offset; the tolerance comes only from the settings and there is no default. |
| LBRI-004 | A measurement record needs an instrument, a condition, a quantity and a data file. | implemented | done | host-test | `src/record::a_record_without_an_instrument_is_rejected` | - |
| LBRI-005 | Carry the uncertainty through the comparison. | prototype | partial | host-test | `src/compare::the_uncertainty_decides_between_within_inconclusive_and_outside` | The expanded uncertainty is the coverage factor times the root of the sum of the squares of the two uncertainties; the values come from the file or the settings and nobody has characterised a bench. |
| LBRI-006 | Link a record to the technical passport. | planned | pending | none | `planned:passport::the_passport_cites_a_record_by_hash` | Not written: it needs the passport to carry a section for records. |
| LBRI-007 | The hash of a record is the known SHA-256 of its data. | implemented | done | host-test | `src/record::the_hash_is_the_known_sha256` | - |
| LBRI-008 | Changed data gives a hash mismatch. | implemented | done | host-test | `src/record::changed_data_gives_a_hash_mismatch` | - |
| LBRI-009 | Every missing part of a record is reported. | implemented | done | host-test | `src/record::every_missing_part_is_reported` | - |
| LBRI-010 | Without a tolerance nothing is judged. | implemented | done | host-test | `src/compare::without_a_tolerance_nothing_is_judged` | - |
| LBRI-011 | The largest deviation says where it is and the first wins a tie. | implemented | done | host-test | `src/compare::the_largest_deviation_says_where_it_is_and_the_first_wins_a_tie` | - |
| LBRI-012 | A point beyond the tolerance stays outside whatever the coverage. | implemented | done | host-test | `src/compare::a_point_beyond_the_tolerance_stays_outside_whatever_the_coverage` | - |
| LBRI-013 | Points outside the simulation are counted, not invented. | implemented | done | host-test | `src/compare::points_outside_the_simulation_are_counted_not_invented` | - |
| LBRI-014 | Curves in different units are refused. | implemented | done | host-test | `src/compare::different_units_are_refused` | - |
| LBRI-015 | Interpolation is linear and stops at the ends. | implemented | done | host-test | `src/curve::interpolation_is_linear_and_stops_at_the_ends` | - |
| LBRI-016 | Bad files are refused with the line. | implemented | done | host-test | `src/curve::bad_files_are_refused_with_the_line` | - |
| LBRI-017 | A header with semicolons or tabs says that the columns need commas. | implemented | done | host-test | `src/curve::a_header_with_semicolons_or_tabs_says_the_columns_need_commas` | - |

## The command line and the settings

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| LBRI-018 | `import` prints the summary and the hash. | implemented | done | host-test | `src/cli::import_prints_the_summary_and_the_hash` | - |
| LBRI-019 | `compare` exits with 1 only when the curve is outside the set tolerance. | implemented | done | host-test | `src/cli::compare_exits_1_only_when_outside_the_set_tolerance` | - |
| LBRI-020 | A unit mismatch and bad files exit with 2. | implemented | done | host-test | `src/cli::unit_mismatch_and_bad_files_exit_2` | - |
| LBRI-021 | `record` checks the record against its data. | implemented | done | host-test | `src/cli::record_is_checked_against_its_data` | - |
| LBRI-022 | There is no default tolerance. | implemented | done | host-test | `src/config::there_is_no_default_tolerance` | - |
| LBRI-023 | A file limit of zero bytes is refused. | implemented | done | host-test | `src/config::a_file_limit_of_zero_bytes_is_refused_and_a_positive_one_is_kept` | - |

## Simulated bench

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| LBRI-024 | The same seed writes the same file and another seed another. | implemented | done | simulator | `simulation::the_same_seed_writes_the_same_file_and_another_seed_another` | The bench is simulated; no instrument has exported the file. |
| LBRI-025 | A clean export is read back and agrees with the true line. | implemented | done | simulator | `simulation::a_clean_export_is_read_back_and_agrees_with_the_true_line` | - |
| LBRI-026 | A spike is found at its point and the next clean export is judged again. | implemented | done | simulator | `simulation::a_spike_is_found_at_its_point_and_the_next_clean_export_is_judged_again` | - |
| LBRI-027 | A drift starts within the tolerance and ends outside it. | implemented | done | simulator | `simulation::a_drift_starts_within_and_ends_outside` | - |
| LBRI-028 | A clock that steps back is refused at its line. | implemented | done | simulator | `simulation::a_clock_that_steps_back_is_refused_at_its_line` | - |
| LBRI-029 | Lost points leave a valid file with a gap that is not filled in. | implemented | done | simulator | `simulation::lost_points_leave_a_valid_file_with_a_gap_that_is_not_filled_in` | - |
| LBRI-030 | A record made from the clean file does not accept the cut one. | implemented | done | simulator | `simulation::a_record_made_from_the_clean_file_does_not_accept_the_cut_one` | - |
| LBRI-031 | Read the files that a real instrument writes. | planned | blocked | none | `planned:instrument_files::a_file_of_a_real_instrument_is_read_with_its_units` | Waits for a sample file of a real instrument; until then only the defined CSV format is read. |
