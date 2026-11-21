# EH3D-LAB-BRIDGE: scope

Status: **early code**. EH3D-LAB-BRIDGE is a library and a command line that read a measured curve from one defined CSV format, check the record of a test and compare the curve with a simulated one. It has been tried on data made by its own tests only. This page says what it is for, what it will not do and where it sits among the other programs of the suite.

## Purpose

The bridge between the simulation and the laboratory: it correlates the data sets of a simulation with real measurements, keeps the configuration of each test so that a person can repeat it, and reports the deviation of every point with its uncertainty.

## What it does

* Reads a measured curve from one defined CSV format whose header names the unit of each column (optionally with an uncertainty column), with a byte order mark, blank lines and CRLF accepted, and writes it back exactly.
* Refuses a bad file with the line, a header that uses semicolons or tabs (it says the columns need commas), and too many points.
* Checks the record of a test: the instrument, the conditions, the quantity and the SHA-256 of the data; a record without one of them is rejected with a stable code, and changed data gives a hash mismatch.
* Compares the measured curve with a simulated one by linear interpolation, giving the deviation of every point, where the largest deviation is, and its expanded uncertainty, and judges it only against a tolerance that the settings give (there is no default).
* Counts the points outside the range of the simulation instead of inventing values, and refuses curves in different units.
* Runs a simulated bench whose exported file is damaged on purpose (a spike, a drift, a clock that steps back, lost points, a lost unit, a cut file).

## What it will not do

* It does not control any instrument: no live control in the first version.
* It makes no calibration claim.
* It invents no tolerance number: they are set after the bench of the author has been characterised.
* It does not read the files of real instruments yet (see `REQUIREMENTS.md`).
* It does not claim that anything was proven with a real bench: the evidence of every capability is `simulator` or `host-test`.

## Where it sits

* **It reads:** a curve (CSV with units in the header), the record of a test (JSON) and a settings file (the tolerance, the coverage factor, default uncertainties and the size limits).
* **It writes:** the summary, the hash and the comparison it is asked to print, to the standard output; it never writes to the files it reads.
* **It is read by:** nothing reads it yet: the technical passport will cite a record by hash once the link is written.
* **It depends on:** nothing of the suite at build time; the simulation results come from EH3D-PHOENIX-PCB and the container from EH3D-COMMON.

## What it never decides

Whether a measurement is right, whether a bench is calibrated, and the limits, the interlocks and the emergency stop of any machine, belong to the person who runs the bench. The result of a comparison is a statement about two curves and their stated uncertainties; it is not a validation of a design. Nothing here starts motion.

The requirements of the program, with the state of each and the test that shows it, are in `REQUIREMENTS.md`; the state in a paragraph is in `STATUS.md`.
