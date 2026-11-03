# EH3D-LAB-BRIDGE: status

## Current state

EH3D-LAB-BRIDGE is early code. Version 0.0.1 holds the base that every repository of the suite carries (manifest, README files in eight languages, changelog, CI, shared checks) and the written scope, requirements, dependencies, threat model and decisions. It also has a Rust library and a command line that read a measured curve from one defined CSV format (a header that names the unit of each column, optionally with an uncertainty column), check a test record (instrument, conditions, quantity and the SHA-256 of the data) and compare the measured curve with a simulated one, giving the deviation of every point with its expanded uncertainty. They have run only on data made by their own tests. Nothing has been run on a machine with real data, and no requirement is validated.

## Capabilities

The official state of each capability is in REQUIREMENTS.md: 6 requirements: 5 `prototype` and 1 `planned` (LBRI-006). This page never states a state that REQUIREMENTS.md does not.

## Open questions

* Which instrument file formats come first.

## Needs from other repositories

* EH3D-PHOENIX-PCB: simulation results
* EH3D-COMMON: container

## Known technical debt and omitted tests

23 tests of the Rust code and the 3 of the base; the Rust tests build their own curves and records and never touch real data. Not tested: files from real instruments. `cargo deny` is not run yet.
