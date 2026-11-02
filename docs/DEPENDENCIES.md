# EH3D-LAB-BRIDGE: dependencies

One row per relation with another repository. The kind is conceptual, compile-time or run-time. The state of a contract is: not written, drafted, accepted, frozen or published. A repository existing does not mean that its contract is ready: this project does not implement or consume a contract that is not accepted.

| From | To | Kind | Contract | Contract state | Minimum version | Tested version | Compatibility test | Change owner | What it blocks | Migration |
|---|---|---|---|---|---|---|---|---|---|---|
| EH3D-LAB-BRIDGE | EH3D-PHOENIX-PCB | conceptual | simulation results to compare | drafted | not set | not set | to be written with the first code | the owner of the contract | the comparison | to be defined |
| EH3D-LAB-BRIDGE | EH3D-COMMON | compile | container, diagnostics | drafted | not set | not set | to be written with the first code | the owner of the contract | all of it | to be defined |
| EH3D-LAB-BRIDGE | EH3D-THERMAL-LAB | conceptual | thermal results to compare | not written | not set | not set | to be written with the first code | the owner of the contract | thermal validation | to be defined |
| EH3D-LAB-BRIDGE | EH3D-PROJECT-PASSPORT | conceptual | records the tests | not written | not set | not set | to be written with the first code | the owner of the contract | nothing here is blocked by it | to be defined |

Rules: repositories are built next to each other as sibling folders and each declares in its manifest the versions of the others that it needs, once those exist. A lower layer never depends on a higher one. The manifest field for build dependencies stays empty until code exists.
