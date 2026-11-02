# EH3D-LAB-BRIDGE: requirements

This register is the single source of the state of each capability. Status values: planned, prototype, implemented, validated. Every requirement is `planned` today: nothing is implemented or verified. High test coverage is not proof of geometric or electrical correctness, so each requirement says how it will be checked and what result is expected.

| Requirement | Description | Status | How it will be checked | Expected result | Known limits |
|---|---|---|---|---|---|
| LBRI-001 | Import one defined measurement file format. | planned | automatic: reference files | Values and units are preserved. | Not started: no limits known yet. |
| LBRI-002 | Store the configuration of each test. | planned | automatic | A test can be repeated from its stored configuration. | Not started: no limits known yet. |
| LBRI-003 | Compare a measured and a simulated curve with a stated tolerance. | planned | automatic: synthetic curves | Known deviations are recovered. | Not started: no limits known yet. |
| LBRI-004 | Measurement record with instrument and condition. | planned | automatic | A record without an instrument is rejected. | Not started: no limits known yet. |
| LBRI-005 | Carry uncertainty through the comparison. | planned | automatic | The result states its uncertainty. | Not started: no limits known yet. |
| LBRI-006 | Link a record to the technical passport. | planned | automatic | The passport can cite the record by hash. | Not started: no limits known yet. |

Rules: a claim of correctness names the corpus, the tolerance and the tool version it holds for. A phase is complete when the measured rate does not drop, every failure is classified and fixed or documented, and the measured figure is reported.
