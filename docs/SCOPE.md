# EH3D-LAB-BRIDGE: scope

Status: **early code**. EH3D-LAB-BRIDGE has a library and a command line; most of what this page lists is still planned. This page says what it is meant to do, what it will not do, and where it sits. Nothing has been run on real data.

## Purpose

EH3D-LAB-BRIDGE is planned as a module that correlates simulation datasets with real laboratory measurements.

Correlates simulation datasets with real laboratory measurements and keeps the configuration of each test.

## What it is planned to do

* Import data from oscilloscopes, network analysers and programmable power supplies, starting with well-defined file formats and leaving instrument protocols for later.
* Compare measurements with simulated curves and report the deviation with its uncertainty.
* Store the configuration of each test.
* Keep a measurement record: instrument, condition, result and comparison with the simulation, with units, estimated error and sensitivity.

## What it will not do

* No live control of instruments in the first version.
* No calibration claims.
* No tolerance numbers are invented: they are set after the author's bench has been characterised.

## Planned shape

A library without a window, written in Rust, with a command-line interface and, where it applies, a kind of task for the server. Programs of the suite exchange work through project files or through the server, never by depending on each other.

The package is a single crate (a library and a command line, no window) with `import`, `compare` and `record`; a curve is a CSV file and a test record is a JSON file. Files written by real instruments, the control of instruments and the link with the technical passport are not written.

## Where it sits

It belongs to the PHOENIX family of the EH3D Fabrication Suite: independent repositories that relate through versioned contracts. The design tool (EH3D-PHOENIX-PCB) does not depend at build time on the repositories that depend on it.
See DEPENDENCIES.md for the relations, REQUIREMENTS.md for the requirements and STATUS.md for the honest state.
