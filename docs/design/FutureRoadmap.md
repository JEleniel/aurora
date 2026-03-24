<!-- AGENT: Do not read or modify this file without specific instructions. -->

# Future Roadmap (Aurora v3)

## Purpose

This document is a sketch intended to support a prompt. It outlines the direction for Aurora v3 tooling, with the expectation that the Architect will expand this into detailed designs and requirements.

## Capability parity (CLI, Editor, MCP)

The goal is capability parity across:

- **CLI**: automation-friendly, scriptable, non-interactive workflows.
- **Editor**: interactive UI workflows (editing, browsing, rendering, reviewing).
- **MCP server**: a stable interface that enables agents and tools to work with models without needing to understand the underlying storage format.

Parity here means the same _model operations_ and _outputs_ are available where they make sense (for example, the CLI does not have an editor UI).

## Output formats: Markdown + SVG and “all PDF”

Aurora’s current outputs are primarily Markdown plus rendered diagrams (SVG). Aurora v3 adds a first-class option to produce an **all-PDF** deliverable set, including:

- PDFs for non-diagram architecture documents.
- PDFs for rendered diagrams (single combined file, or one file per drawing).
- A single-file PDF “complete set” option.

## Tool consolidation: svg_prep becomes integrated

The functionality of `svg_prep` becomes integrated into the CLI and Editor, and the separate `svg_prep` tool goes away.

`svg_prep` started as a quick tool, but proved useful to users. In Aurora v3, the intent is that users do not have to learn or install a separate tool to:

- Prepare, validate, and normalize SVG assets/templates.
- Keep model configuration and assets in sync.

## Model storage and agent access

Once the MCP server is available, it becomes possible for an agent to work with the Aurora model through the MCP interface without understanding the model storage format.

Because Aurora already uses SQLite in the stack, it is a natural fit to offer a **single-file SQLite storage** option for models.

## Security, signing, and audit (TBD)

Encryption, signing, and auditing are desired capabilities for Aurora v3, but the detailed design is not yet defined.

This section is intentionally a placeholder for:

- Model encryption (at rest).
- Cryptographic signing.
- Full change record / audit history.

## Roadmap sketch

This list is deliberately high-level.

- **Generation**
    - Common non-diagram architecture documents (see below)
    - PDF output (including a single-file complete set)
- **Analysis**
    - Change impact analysis: given a proposed change to an architecture element, determine upstream and downstream impact
    - In-model basic index
    - Cost modeling
- **Rendering**
    - Improved default layout engine
    - Improved edge routing
    - More layout options
    - PDF output (one file or one file per drawing)
    - Design reference: `docs/design/ViewLayouts.md` and `docs/design/ViewRenderingArchitecture.md`
- **Storage**
    - SQLite database model storage option (including a single-file model)
- **Security and audit (TBD)**
    - Cryptographic signing
    - Full change record
    - Full model encryption

## Common non-diagram architecture documents (Aurora terms)

The items below describe common architecture documents using Aurora’s card types, as defined in `Aurora.modelconfiguration.json`.

Notes:

- “Included cards” are the typical card types that would be pulled into the document.
- “Roots” are the typical starting points for generation (for example, a Mission, System, or Threat Model).
- Some documents map closely to existing Aurora _views_ (for example, “Context”, “Requirements”, “Deployment”, and “Security”), but the document format is non-diagram narrative/report output.

### Core architecture documents

| Document                             | What it describes in Aurora terms                                                                                              | Included cards (typical)                              | Typical roots |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------- | ------------- |
| Architecture Overview                | The mission intent, primary motivations, constraints, and the top-level systems/applications that satisfy the mission.         | MIS, DRI, STK, ACT, VND, SYS, APP, CNS, ADR           | MIS           |
| Architecture Decision Records (ADRs) | A set of decision records, optionally connected to the requirements/constraints they support.                                  | ADR, REQ, CNS, DRI, MIS                               | ADR, MIS      |
| System Context Description           | The boundary of the system-of-interest and the external parties/sources that interact with it.                                 | MIS, STK, ACT, VND, SYS, APP, COM, INT, DSR, CNS, REQ | MIS           |
| Quality Attribute Requirements       | Verifiable requirements and constraints that define measurable quality expectations, plus how they are justified and verified. | REQ, CNS, ADR, TES, CAP, FEA                          | MIS, SYS, APP |

### Design definition artifacts

| Document                      | What it describes in Aurora terms                                                                                 | Included cards (typical)                    | Typical roots |
| ----------------------------- | ----------------------------------------------------------------------------------------------------------------- | ------------------------------------------- | ------------- |
| Component Specifications      | For each component, its responsibility, dependencies, exposed/called interfaces, and produced/consumed artifacts. | COM, INT, ART, DST, DSR, TES, REQ, CNS      | SYS, APP, COM |
| Interface / API Contracts     | For each interface boundary, the accepted/returned artifacts and the components that call or expose it.           | INT, ART, COM, REQ, CNS, TES                | INT, COM      |
| Data Architecture Description | Data sources, stores, and artifacts, including ownership/protection and governing constraints/controls.           | DSR, DST, ART, AST, ROW, CNS, CTL, RIS, COM | SYS, APP, DST |
| Technology Stack Rationale    | Decisions and constraints that justify technology choices for systems, applications, and components.              | ADR, CNS, REQ, SYS, APP, COM                | MIS, SYS, APP |

### Operational architecture documents

| Document                           | What it describes in Aurora terms                                                                                                  | Included cards (typical)                              | Typical roots |
| ---------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------- | ------------- |
| Deployment Model Description       | Where software runs: deployments, nodes, and the applications/components and stores placed on them.                                | DEP, NOD, APP, COM, DST, DSR, CNS                     | DEP, VND      |
| Runtime Behavior / Execution Model | The dynamic behavior of the system, expressed as processes/activities and/or state machines depending on the domain.               | PRO, ATV, CON, TRG, ACT, STM, STA, PRD, EVT           | CAP, STM      |
| Security Architecture              | Protection of assets against threats, including threat models, controls, constraints, and resulting risks.                         | THM, AST, ROW, ADV, THC, THD, CTL, CNS, RIS, DST, ART | THM           |
| Observability Strategy             | Operational visibility as requirements/constraints and produced artifacts (logs/metrics/traces) tied back to components and tests. | REQ, CNS, COM, ART, INT, TES                          | SYS, APP      |

### Lifecycle and governance documents

| Document                        | What it describes in Aurora terms                                                                               | Included cards (typical)                                        | Typical roots |
| ------------------------------- | --------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------- | ------------- |
| Constraints and Assumptions Log | The declared constraints that limit allowable solutions, plus decisions that justify or respond to constraints. | CNS, ADR, REQ, MIS                                              | MIS           |
| Risk Register (Technical)       | Known risks and their relationships to threats, assets, controls, and the architecture elements they affect.    | RIS, THM, THD, ADV, THC, AST, ROW, CTL, CNS, DST, SYS, APP, COM | MIS, THM      |

## Lightweight supporting artifacts

| Artifact                            | What it describes in Aurora terms                                                                        | Included cards (typical) | Typical roots |
| ----------------------------------- | -------------------------------------------------------------------------------------------------------- | ------------------------ | ------------- |
| Definition of Done for Architecture | Verifiable completion criteria expressed as requirements, tests, and constraints tied to mission intent. | REQ, TES, CNS, DRI, MIS  | MIS           |
