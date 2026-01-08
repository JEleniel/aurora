# AURORA Tooling Design Documentation

**Project**: AURORA | **Component**: Tooling | **Version**: 1.0.0 | **Status**: Architectural Design

---

## Documentation Index

This directory contains comprehensive architectural and design documentation for AURORA tooling—a complete system for creating, managing, validating, querying, and rendering architecture models expressed as JSON cards and directional links.

### Primary Documents

1. **[ARCHITECTURE.md](ARCHITECTURE.md)** — **START HERE**
   - Executive summary and high-level overview
   - Complete layered architecture (presentation, application, domain, persistence)
   - Core subsystems and their responsibilities
   - Technology stack decisions
   - Implementation roadmap (16-week phases)
   - Key design decisions and rationale

2. **[QUERY-ENGINE.md](QUERY-ENGINE.md)**
   - Graph traversal (reachability, paths, neighborhoods)
   - Filtering (by type, status, owner, constraints, attributes)
   - Full-text and regex search
   - Specialized queries (traceability, dependencies, impact analysis)
   - Query optimization and indexing strategy
   - Performance characteristics and benchmarks

3. **[RENDERING-ENGINE.md](RENDERING-ENGINE.md)**
   - Rendering pipeline (projection → layout → rendering)
   - Projection system (hierarchy, matrix, graph, sequence, state machine, centered element)
   - Layout algorithms (hierarchical, force-directed, circular, layered)
   - Format renderers (Mermaid, SVG, HTML, CSV, Markdown)
   - Styling, theming, and customization
   - Caching and performance optimization

4. **[PERSISTENCE.md](PERSISTENCE.md)**
   - File organization and storage strategy
   - I/O operations (load, save, transactions)
   - Transactional semantics with rollback
   - Caching strategy (indexes, diagrams)
   - Git integration (semantic diffs, impact analysis)
   - Backup and recovery procedures
   - Export to external formats

5. **[DATA-STORAGE-EXPORT.md](DATA-STORAGE-EXPORT.md)** — **Data Storage & Formats**
   - Primary storage: Filesystem with type-based folder organization
   - Archive format: `Arch.ZIP` (portable, distributable)
   - ZIP structure with manifest and integrity checksums
   - Output formats: SVG (primary), Mermaid, HTML, CSV, Markdown
   - SVG generation with interactive features
   - Export operations and batch processing
   - Validation and integrity checking

---

## Quick Reference

### Architecture Diagram

```text
┌─────────────────────────────────────────────────────────────┐
│                   PRESENTATION LAYER                         │
│  Web UI (React) │ CLI (Node) │ IDE Plugin (VS Code)          │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│                   APPLICATION LAYER                          │
│  CardService │ LinkService │ ViewService │ ValidationService │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│                    DOMAIN LAYER                              │
│  Query Engine │ Rendering Engine │ Validation Engine        │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│              PERSISTENCE & SERIALIZATION                     │
│  JSON I/O │ Git Integration │ Caching │ File System          │
└─────────────────────────────────────────────────────────────┘
```

### Core Entities

```typescript
Card {
  id: "namespace:name"
  type: CardType
  name: string
  description: string
  version: string
  status: CardStatus
  attributes: {}
  links: []
  audit_history: []
}

Link {
  id: "link:source:target:context"
  source_id: string
  target_id: string
  rationale: string
  strength: "weak" | "normal" | "strong"
  metadata: {}
}

View {
  id: string
  type: ViewType
  scope: []
  filter: {}
  format: OutputFormat
}
```

### Subsystem Responsibilities

| Subsystem | Responsibility | Key Classes |
| --- | --- | --- |
| **Query Engine** | Graph traversal, filtering, searching | `QueryEngine`, `QueryBuilder`, `IndexManager` |
| **Rendering Engine** | Diagram generation, layout, styling | `Projector`, `LayoutEngine`, `RendererRegistry` |
| **Persistence Layer** | File I/O, transactions, version control | `ModelLoader`, `ModelSaver`, `GitIntegration` |
| **Validation Engine** | Schema, naming, reference, semantic validation | `ValidationService`, `ValidatorRegistry` |
| **Card Service** | CRUD operations on cards | `CardService` |
| **Link Service** | CRUD operations on links | `LinkService` |

---

## Design Principles

### 1. Separation of Concerns

Each layer has clear responsibilities:

- **Presentation**: User interaction (web, CLI, IDE)
- **Application**: Business logic (services)
- **Domain**: Core concepts (card, link, view)
- **Persistence**: Storage abstraction

### 2. JSON Canonicity

- Cards and links stored as JSON (AURORA spec)
- Schemas are source of truth
- UI/diagrams are derived views
- No dual editing (JSON is canonical)

### 3. Determinism

- Identical inputs → identical outputs
- No hidden state or randomness
- Audit trails preserve all changes
- Diagrams reproducible from JSON

### 4. Extensibility

- Pluggable validators, renderers, query functions
- Custom attributes on cards
- Custom view projections
- Extension points for integrations

### 5. Traceability

- Every change recorded in audit history
- Complete provenance (source, owner, rationale)
- Git history as change log
- Impact analysis on modifications

---

## Technology Stack

### Core

- **Language**: TypeScript (strong typing, rich ecosystem)
- **Runtime**: Node.js (server) + Browser (client)
- **Data Format**: JSON (canonical) + JSON Schema (validation)

### Validation

- **AJV**: JSON Schema validator (fastest available)

### Query & Data

- **In-memory**: Maps, adjacency lists, indexes
- **Git**: Version control integration
- **Database** (optional for large models): PostgreSQL with JSON columns

### UI

- **Web**: React + TypeScript + Tailwind CSS
- **Diagram Rendering**: Mermaid.js (client-side)
- **Editor**: Monaco Editor (VS Code component)
- **State Management**: Redux or Zustand

### CLI

- **Framework**: Commander.js or Yargs
- **Output**: Chalk for colors, Table for formatting

### IDE Plugin

- **Platform**: VS Code Extension API
- **Ecosystem**: TypeScript, webpack

---

## File Organization

```
docs/design/tool/
├── README.md (this file)
├── ARCHITECTURE.md (primary design document)
├── QUERY-ENGINE.md (query subsystem)
├── RENDERING-ENGINE.md (rendering subsystem)
├── PERSISTENCE.md (persistence subsystem)
├── examples/
│   ├── query-examples.md
│   ├── rendering-examples.md
│   └── persistence-examples.md
└── (future: API.md, CLI.md, UI.md)
```

---

## Reading Guide

### For Architects

1. Read [ARCHITECTURE.md](ARCHITECTURE.md) completely
2. Understand layered design and subsystem boundaries
3. Review technology decisions and tradeoffs
4. Study implementation roadmap and phases

### For Backend Developers

1. Read [ARCHITECTURE.md](ARCHITECTURE.md) — Application & Domain layers
2. Deep dive [PERSISTENCE.md](PERSISTENCE.md)
3. Study [QUERY-ENGINE.md](QUERY-ENGINE.md)
4. Review API interfaces and contracts

### For Frontend Developers

1. Read [ARCHITECTURE.md](ARCHITECTURE.md) — Presentation & Application layers
2. Deep dive [RENDERING-ENGINE.md](RENDERING-ENGINE.md)
3. Review component hierarchy and state management
4. Study styling and theming system

### For DevOps/Infrastructure

1. Read deployment section in [ARCHITECTURE.md](ARCHITECTURE.md)
2. Review persistence and caching strategies
3. Study Git integration and version control
4. Plan infrastructure for web deployment

---

## Key Design Decisions

### 1. JSON-First Storage

**Decision**: Store all data as JSON files in Git-friendly structure

**Rationale**:

- Easy versioning (line-by-line diffs)
- Human-readable and inspectable
- Standard format with schemas
- No database migration burden
- Works offline

**Tradeoff**: Smaller scale (100-10k cards optimal) vs database scalability

### 2. Untyped Links

**Decision**: Links have no semantic type; meaning is view-dependent

**Rationale**:

- Simpler model (no link type enumeration)
- More flexible (same link can mean different things)
- Reduces data duplication
- View is source of interpretation

**Tradeoff**: Requires views for semantic understanding

### 3. Single Root Driver

**Decision**: All architectures converge to single Root Driver

**Rationale**:

- Clear hierarchy and structure
- Ensures connectivity
- Simplifies traversal
- Aligns with project mission

**Tradeoff**: Restricts to single-root structures (sufficient for most projects)

### 4. Atomic Link Updates

**Decision**: All links stored in single file to ensure atomicity

**Rationale**:

- Prevents orphaned references
- Transactional guarantees
- Git commits atomic

**Tradeoff**: Can't update individual links independently

### 5. Pluggable Renderers

**Decision**: Rendering as plugins, not hardcoded

**Rationale**:

- Extensibility (new formats without core changes)
- Separation of concerns
- Ease of testing

**Tradeoff**: Slightly more complex architecture

---

## Implementation Phases

### Phase 1: Foundation (Weeks 1-4)

**Goal**: Core model, persistence, CLI

- Domain model implementation
- JSON I/O (load/save)
- Basic validation
- Query engine (traversal, filtering)
- CLI tool (create, list, view, validate)

**Deliverable**: Working CLI for basic operations

### Phase 2: Rendering (Weeks 5-8)

**Goal**: Visualization and diagrams

- Projection system
- Layout algorithms
- Mermaid rendering
- SVG export
- Web UI scaffolding

**Deliverable**: View multiple diagram formats from command line

### Phase 3: UI & Polish (Weeks 9-12)

**Goal**: User-facing interfaces

- Web UI (card editor, diagram viewer)
- Git integration
- IDE plugin (VS Code)
- Caching optimization

**Deliverable**: Feature-complete web and CLI tools

### Phase 4: Release (Weeks 13-16)

**Goal**: Production-ready

- Comprehensive testing
- Documentation and tutorials
- Performance tuning
- v1.0 release with artifacts

**Deliverable**: v1.0 release with documentation

---

## Open Questions

1. **Real-Time Collaboration**: Should tool support multiple simultaneous editors? (Requires CRDTs)

2. **Database vs Files**: At what model size do we switch to database backend?

3. **View Caching**: Should rendered views be cached in Git? (Trade space for performance)

4. **Offline Mode**: Should web UI work offline? (Requires service workers)

5. **Integration Points**: What external systems should be supported? (Jira, Confluence, Slack, etc.)

---

## Conventions

### Naming

- **IDs**: `namespace:element-name` (lowercase, hyphens)
- **Files**: `{namespace}-{name}.json` (lowercase, hyphens)
- **Links**: `link:source:target:context`
- **Classes**: PascalCase (`QueryEngine`, `ModelLoader`)
- **Methods**: camelCase (`loadModel`, `saveCard`)
- **Constants**: UPPER_SNAKE_CASE

### Code Organization

```
src/
├── domain/
│   ├── Card.ts
│   ├── Link.ts
│   ├── View.ts
│   └── Model.ts
├── services/
│   ├── CardService.ts
│   ├── LinkService.ts
│   ├── ValidationService.ts
│   └── ViewService.ts
├── engines/
│   ├── QueryEngine.ts
│   ├── RenderingEngine.ts
│   └── ValidationEngine.ts
├── persistence/
│   ├── ModelLoader.ts
│   ├── ModelSaver.ts
│   └── GitIntegration.ts
├── ui/
│   ├── cli/
│   ├── web/
│   └── vscode/
└── tests/
    ├── unit/
    └── integration/
```

---

## Next Steps

1. **Detailed Specifications**: For each subsystem, create implementation-specific design docs
2. **API Reference**: Generate from code once implementation begins
3. **CLI Guide**: Command reference and examples
4. **UI Guide**: Screenshots, workflows, keyboard shortcuts
5. **Contribution Guide**: How to extend, plugin development, testing

---

## References

- **AURORA Specification**: [/home/jeleniel/repos/aurora/README.md](../../README.md)
- **Card Reference**: [/home/jeleniel/repos/aurora/docs/card-field-reference.md](../card-field-reference.md)
- **Schemas**: [/home/jeleniel/repos/aurora/schemas/](../../schemas/)
- **Agent Instruction**: [/home/jeleniel/repos/aurora/docs/AGENT-INSTRUCTION.md](../AGENT-INSTRUCTION.md)

---

**Document Version**: 1.0.0 | **Last Updated**: 2025-12-14 | **Status**: Design Complete
