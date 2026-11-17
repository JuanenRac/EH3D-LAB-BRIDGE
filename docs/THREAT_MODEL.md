# EH3D-LAB-BRIDGE: threat model

Basic model for early code. The rows marked partial have a mitigation written and tests for it, run only on data made by the tests themselves; the rows marked planned have nothing yet. No row is assessed against real data.

| Threat | Planned mitigation | Planned test | State | Residual risk |
|---|---|---|---|---|
| Malformed or oversized instrument files | the curve reader refuses a bad line (it names the line), too many points, a lost unit, a text value, a cut file and an empty file | `bad_files_are_refused_with_the_line`, `too_many_points_are_refused`, `a_lost_unit_a_text_value_a_cut_file_and_an_empty_file_are_each_refused_where_they_are` | partial | samples made by the tests; no fuzzing and no real instrument file |
| Wrong units silently accepted | curves with different units are refused instead of compared | `different_units_are_refused`, `unit_mismatch_and_bad_files_exit_2` | partial | only the units the format knows |
| Corrupt or malicious project or input files (oversized entries, path tricks, deep nesting, resource exhaustion) | the record refuses unknown fields and the curve reader limits the points | `unknown_fields_are_refused`, `too_many_points_are_refused` | partial | hostile samples written by hand; no fuzzing yet |
| Temporary files, logs and reports that reveal private designs | no private data in outputs; redaction | automated pattern search | planned | not assessed: nothing is implemented |
| Compromised dependency or binary | pinned versions, licence and advisory checks, software bill of materials for releases | CI checks | planned | not assessed: nothing is implemented |
| Power loss or forced close during a save | atomic write: temporary file, then rename; old file never deleted first | interruption tests at controlled points | planned | not assessed: nothing is implemented |

Minimum negative tests once there is code: hostile files and resource limits; secrets absent from logs, errors and reports; pinned dependencies and licence review before publishing. Report a vulnerability privately as described in SECURITY.md, without attaching private designs.
