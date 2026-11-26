# Changelog

All notable changes to this project are documented here.

## [Unreleased]

## [0.0.5] - Records that are not UTF-8 are refused, and many more tests

- **A test record that is not UTF-8 text is refused:** the bytes that were not text were replaced and the record was accepted with other letters than the ones written.
- **A JSON array is no longer read as a record:** only an object is.
- **8 more tests** of the Rust code: seeded curves read back and interpolated between their neighbours, the limits of points, tolerance and coverage exactly on their edge and a record cut at every place.
- **More tests, for the shared tools of the base and for the curves and the records:** `tools/test_shared_tools.py` adds 65 tests that run the lessons linter, the workflow check, the CHANGELOG check of a pull request and the banner check on small inputs made by the test itself (findings, baselines, exit codes and the shape of what they print). The Rust code gets 18 more: seeded curves with and without uncertainty written and read again are the same curve and the same text, 20 000 points are read without a panic, the curve answers exactly on every node and between the neighbours elsewhere, a question that is not a number has no answer, each kind of bad file is refused with the line of the fault (also after blank lines), a value that is not finite is refused however it is spelt, the most points is read and one more is not, the known SHA-256 of nothing and of a million bytes, a text that is not a record is an error and never a panic, and a record names every missing part in a fixed order.
- **The register of requirements names a test for each row:** `docs/REQUIREMENTS.md` has one row for each capability with an identifier (`LBRI-001` on), its state (planned, prototype, implemented or validated), how far it is (done, partial, pending or blocked), its evidence (simulator, host-test or none), the test that shows it and its limits; a capability that is not written names the test that will show it. `tests/reqs.rs` reads the table and fails when a test does not exist, when a state does not agree with the progress or the evidence, when `docs/STATUS.md` does not say the same numbers, when a README does not link the documents or when a document holds a date or a private reference. `docs/SCOPE.md`, `docs/STATUS.md` and `docs/NOT_YET.md` are rewritten to the same shape and the eight READMEs link the five documents.
- The shared workflow checker (`tools/workflow_hygiene.py`) is the current one of the documentation hub: the filters under a trigger (branches, paths, types) are no longer read as events.

## [0.0.4] - Clearer CSV errors and the place of the largest deviation

- **The threat model says what the code already does:** the rows of `docs/THREAT_MODEL.md` that have a mitigation in the code and a test for it are marked `partial` and name the tests and what is still not covered; the rows without either stay `planned`.

- **A comparison that could not be judged no longer exits with 0:** `compare` exits with 3 when the verdict is `not_judged` (no tolerance, an unknown uncertainty) or `inconclusive`, so a script that reads only the exit code cannot take it for a pass; the new setting `exit_on_unjudged = no` keeps the old exit code 0. Tests cover both.
- **A comparison that left most of the measured points out is no longer "within":** the result now states the share of the measured points that were compared (`coverage`), and when every compared point fits the tolerance but that share is below the new `min_coverage` setting (0.9 unless the settings file says otherwise, 0 turns the check off), the verdict is inconclusive instead of within. A point beyond the tolerance still makes the verdict outside whatever the coverage. Tested on synthetic curves with 1 of 11 points inside the simulated range.
- **A simulated bench:** `sim::SimBench` writes the file an instrument would export (a true line plus noise from a seeded generator, so the same seed gives the same file) and damages it on request: a spike, a drift, a clock that steps back, lost points, a lost unit, a text value, a file cut short or an empty one. `tests/simulation.rs` checks that a clean file is read back and judged, that each failure is refused at its own line or found at its own point, that a record made from the clean file does not accept the cut one, and that the next clean file is judged again. The manifest now says `simulator` as its evidence. No real instrument has been used.
- **A curve file with semicolons or tabs says so:** a header separated by `;` or by tabs (the export of many instruments and of spreadsheets in some regions) is refused with a message that says the columns must be separated by commas, instead of a count of columns that makes no sense.
- **The comparison says where the largest deviation is:** the result has a new field `max_abs_deviation_at`, the x of the point with the largest absolute deviation (the first one when several are equal), next to `max_abs_deviation`; it is null when no point was compared.
- **The settings refuse `max_file_bytes = 0`:** a limit of zero bytes would refuse every curve and record, so it is a bad value of its line.

## [0.0.3] - Shared validation tools, a split CI and a pinned Rust toolchain

- **A picture of the command line in the README:** `images/screens/lab-bridge-01-cli.png` shows `import` and `compare` on a measured and a simulated curve with the tolerance of the settings file, taken from the built program; it is shown in the eight READMEs.
- **The shared validation tools are updated:** the validator also runs the workflow and lessons checks of the suite with their latest rules, including a check to run before every push.
- **The CI validation checks the workflows and the lessons of past CI failures against a baseline:** `tools/workflow_hygiene.py` and `tools/lessons_lint.py` (copies of the shared tools) run in the validation, and only findings that are not in `tools/workflow_hygiene_baseline.json` and `tools/lessons_lint_baseline.json` fail it, so what exists today stays as it is and a new one is caught. `python tools/workflow_hygiene.py --root . --detail` and `python tools/lessons_lint.py --repo .` list the findings.
- **The validation of the CI is the reusable one of the documentation hub:** the job `validate` calls `validate.yml` of EH3D-DOCS (pinned by commit) and the check of the CHANGELOG is a job of its own.
- **The CI is split in two workflows:** the validation of the repository (manifest, documentation, pictures, tools) stays in `ci.yml` and the Rust jobs move to `rust.yml`, which does not run for a change that only touches Markdown files or pictures, so a change of the documentation no longer builds the code on two systems, and they keep a cache of the build. The names of the jobs are the same.
- **The Rust toolchain is pinned:** `rust-toolchain.toml` fixes the version of Rust (the same one in every repository of the suite) with `rustfmt` and `clippy`, the workflows install that version and not `stable`, so a new release of Rust cannot turn the CI red by itself.

## [0.0.2] - First release: the library, the command line, the stable error codes and the tests

- Add the library: a measured curve read from one defined CSV format with the unit of each column in the header and an optional uncertainty column.
- Add the test record with instrument, conditions, quantity and the SHA-256 of the data, rejected when a part is missing, and the comparison with a simulated curve with deviation, expanded uncertainty and a verdict that is within, outside, inconclusive or not judged; there is no default tolerance. Stable codes `PHOENIX-LBRI-001` to `PHOENIX-LBRI-004`.
- Add the command line (`import`, `compare`, `record`) and a settings file; run format, lints and tests of the Rust code on Linux and Windows in the CI.
- **Brand:** Add the application icon and the logos of the program.
- **Shared checks updated:** the validator also compares the badge and the Version row of the README with the manifest, fails when one of its checks cannot run, and the CHANGELOG step of the workflow reads the base branch from the environment.
- **The workflows are hardened:** every action is pinned by its commit with its version in a comment, and the checkouts keep no credentials.

## [0.0.1] - The base of the repository

- **The base that every repository of the suite has:** the manifest, the README files in eight languages, this changelog, the CI workflow, the issue and pull-request templates, and the shared checks of EH3D-DOCS (the repository check and the drawers of the two README pictures).
- **A planned project, described honestly:** the scope, the requirements (all planned), the relations with other repositories, the threat model, the decisions that apply and the rules for writing code are in `docs/`; there is no code yet.
