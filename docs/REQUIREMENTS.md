# EH3D-LAB-BRIDGE: requirements

This register is the single source of the state of each capability. Status values: planned, prototype, implemented, validated. 5 requirements are `prototype` (code and tests exist, only on data made by the tests) and 1 are `planned` (LBRI-006); none has been used on real data or validated. High test coverage is not proof of geometric or electrical correctness, so each requirement says how it will be checked and what result is expected.

| Requirement | Description | Status | How it will be checked | Expected result | Known limits |
|---|---|---|---|---|---|
| LBRI-001 | Import one defined measurement file format. | prototype | automatic: reference files | Values and units are preserved. | One defined CSV format with the unit of each column in the header; values and units are read and written back exactly. The files of real instruments are not read. |
| LBRI-002 | Store the configuration of each test. | prototype | automatic | A test can be repeated from its stored configuration. | The record keeps the instrument, the conditions, the quantity and the hash of the data, so a test can be repeated by a person; the settings of the instrument itself are free text. |
| LBRI-003 | Compare a measured and a simulated curve with a stated tolerance. | prototype | automatic: synthetic curves | Known deviations are recovered. | Linear interpolation of the simulated curve at the measured points, checked with synthetic curves with a known offset; the tolerance comes only from the settings and there is no default. |
| LBRI-004 | Measurement record with instrument and condition. | prototype | automatic | A record without an instrument is rejected. | A record without an instrument, a condition, a quantity or a data file is rejected with a stable code. |
| LBRI-005 | Carry uncertainty through the comparison. | prototype | automatic | The result states its uncertainty. | The expanded uncertainty is the coverage factor times the root of the sum of the squares of the measured and simulated uncertainties; without them the comparison is reported but not judged. The values come from the file or the settings and nobody has characterised a bench. |
| LBRI-006 | Link a record to the technical passport. | planned | automatic | The passport can cite the record by hash. | Not started. |

Rules: a claim of correctness names the corpus, the tolerance and the tool version it holds for. A phase is complete when the measured rate does not drop, every failure is classified and fixed or documented, and the measured figure is reported.
