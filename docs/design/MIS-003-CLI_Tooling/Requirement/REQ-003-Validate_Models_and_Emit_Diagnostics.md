# REQ-003: **Requirement**

**Name:** Validate Models and Emit Diagnostics\

## Description

The tooling SHALL validate loaded models for core invariants (unique ids, at least one Mission, valid link targets, reachability from the mission root) and SHALL emit structured diagnostics with severity (error|warning|info). Validation SHALL fail the command when errors are present and SHOULD still surface warnings (e.g., relationship matrix compliance).

## Links

_No outgoing links._
