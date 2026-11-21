# EH3D-LAB-BRIDGE: status

## Current state

EH3D-LAB-BRIDGE is early code on a computer: a Rust library and a command line (`import`, `compare`, `record`) that read a measured curve from one defined CSV format, check the record of a test and compare the measured curve with a simulated one, giving the deviation of every point with its expanded uncertainty. A simulated bench damages the exported file in several ways. All of it has run only on data made by its own tests; the files of real instruments, the control of instruments and the link with the technical passport are not written, and no requirement is validated.

## Capabilities

The official state of each capability is in `REQUIREMENTS.md`. The register has 31 requirements: 25 implemented, 4 prototype, 2 planned and 0 validated. By progress: 25 done, 4 partial, 1 pending and 1 blocked. This page never states a state that `REQUIREMENTS.md` does not.

## Open questions

* Which instrument file formats come first.
* How the tolerance and the uncertainty are characterised once the bench of the author exists.

## What it needs from other repositories

* EH3D-PHOENIX-PCB: the simulation results.
* EH3D-COMMON: the container.
* EH3D-PROJECT-PASSPORT: the passport that will cite a record.

## Known technical debt and omitted tests

* 63 tests of the Rust code (nine of them keep this register true) and the 68 of the base (65 of them test the shared tools of the base); the Rust tests build their own curves and records and never touch real data.
* Eight of the tests run the simulated bench (`tests/simulation.rs`).
* Not tested: files from real instruments.
* `cargo deny` is not run yet.
