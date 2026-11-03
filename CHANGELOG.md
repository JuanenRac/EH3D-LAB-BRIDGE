# Changelog

All notable changes to this project are documented here.

## [Unreleased]
- Add the library: a measured curve read from one defined CSV format with the unit of each column in the header and an optional uncertainty column.
- Add the test record with instrument, conditions, quantity and the SHA-256 of the data, rejected when a part is missing, and the comparison with a simulated curve with deviation, expanded uncertainty and a verdict that is within, outside, inconclusive or not judged; there is no default tolerance. Stable codes `PHOENIX-LBRI-001` to `PHOENIX-LBRI-004`.
- Add the command line (`import`, `compare`, `record`) and a settings file; run format, lints and tests of the Rust code on Linux and Windows in the CI.
- **Brand:** Add the application icon and the logos of the program.
- **Shared checks updated:** the validator also compares the badge and the Version row of the README with the manifest, fails when one of its checks cannot run, and the CHANGELOG step of the workflow reads the base branch from the environment.
- **The workflows are hardened:** every action is pinned by its commit with its version in a comment, and the checkouts keep no credentials.

## [0.0.1] - The base of the repository

- **The base that every repository of the suite has:** the manifest, the README files in eight languages, this changelog, the CI workflow, the issue and pull-request templates, and the shared checks of EH3D-DOCS (the repository check and the drawers of the two README pictures).
- **A planned project, described honestly:** the scope, the requirements (all planned), the relations with other repositories, the threat model, the decisions that apply and the rules for writing code are in `docs/`; there is no code yet.
