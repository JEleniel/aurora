# AURORA Tooling Architecture

**Version**: 1.0.0 | **Status**: Design | **Audience**: Tool implementers, contributors

---

## 1. Executive Summary

AURORA Tooling is a modular, domain-driven system for creating, managing, validating, querying, and rendering architectural models expressed as JSON cards and directional links. The architecture prioritizes:

- **Separation of Concerns**: Clear layering between data, validation, query, visualization, and UI.
- **Extensibility**: Plugin-based validation, rendering, and view generation.
- **Traceability**: Complete audit trails and provenance preservation throughout all operations.
- **Determinism**: Identical inputs always produce identical outputs; no hidden state or randomness.
- **Machine-First Design**: JSON canonicity; human interaction layers on top.

---

## 2. Architectural Overview

### 2.1 High-Level Layers

```text
┌─────────────────────────────────────────────────────────────┐
│                   PRESENTATION LAYER                         │
│  (Web UI, CLI, IDE Plugins, Markdown Rendering)             │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│                   APPLICATION LAYER                          │
│  (Editors, Validators, Query Engine, Export/Import)         │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│                    DOMAIN LAYER                              │
│  (Card, Link, View, Constraint, Graph Models)               │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│              PERSISTENCE & SERIALIZATION                     │
│  (JSON I/O, File System, Git Integration, Caching)          │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Core Subsystems

1. **Model System**: In-memory representation of cards, links, views
2. **Validation Engine**: Schema conformance, naming conventions, reference integrity
3. **Query Engine**: Graph traversal, filtering, searching, path finding
4. **Rendering Engine**: Diagram generation, report generation, view projection
5. **Persistence System**: File I/O, serialization, version control integration
6. **UI Framework**: Web UI, CLI, IDE plugin infrastructure

---

## 3. Domain Model Architecture

### 3.1 Core Entities

```text
Card {
  id: string                    // namespace:element-name
  type: CardType                // driver, requirement, behavior, etc.
  name: string                  // Human-readable title
  description: string           // Plain text explanation
  version: SemanticVersion      // 1.0.0
  status: CardStatus            // proposed, approved, implemented, verified, deprecated, retired
  attributes: Map<string, any>  // Type-specific attributes
  relations: CardRef[]          // References to other cards
  links: LinkRef[]              // Outgoing links
  constraints: ConstraintRef[]  // Applied constraints
  provenance: Provenance        // Source, owner, origin
  audit_history: AuditEvent[]   // Complete change trail
  created_date: DateTime        // ISO 8601
  modified_date: DateTime       // ISO 8601
}

Link {
  id: string                    // link:source:target:context
  source_id: string             // Card ID
  target_id: string             // Card ID
  rationale: string             // Why this link exists
  strength: LinkStrength        // weak, normal, strong
  metadata: Map<string, any>    // View-dependent context
  created_date: DateTime
  modified_date: DateTime
}

View {
  id: string
  type: ViewType                // matrix, diagram, hierarchy, graph, custom
  scope: CardRef[]              // Which cards to include
  filter: FilterCriteria        // Status, owner, type constraints
  sort_by: SortField[]          // Ordering
  format: OutputFormat          // mermaid, svg, html, json, csv
  relationships: RelationshipType[] // Which link semantics to show
}

ArchitectureModel {
  cards: Map<string, Card>
  links: Map<string, Link>
  views: Map<string, View>
  root_driver_id: string        // Single reference to Root Driver
  metadata: Map<string, any>    // Project-level metadata
}
```

### 3.2 Type Enumerations

```typescript
enum CardType {
  DRIVER = "driver",
  REQUIREMENT = "requirement",
  BEHAVIOR = "behavior",
  INTERFACE = "interface",
  CONSTRAINT = "constraint",
  ACTOR = "actor",
  LOGICAL_COMPONENT = "logical-component",
  DEPLOYABLE_NODE = "deployable-node",
  TEST = "test",
  ARTIFACT = "artifact",
  VIEW = "view",
  NOTE = "note",
}

enum CardStatus {
  PROPOSED = "proposed",
  APPROVED = "approved",
  IMPLEMENTED = "implemented",
  VERIFIED = "verified",
  DEPRECATED = "deprecated",
  RETIRED = "retired",
}

enum LinkStrength {
  WEAK = "weak",
  NORMAL = "normal",
  STRONG = "strong",
}

enum ViewType {
  MATRIX = "matrix",              // Traceability matrix
  DIAGRAM = "diagram",            // Graph visualization
  HIERARCHY = "hierarchy",        // Tree view
  SEQUENCE = "sequence",          // Behavioral sequence
  STATE_MACHINE = "state_machine", // FSM diagram
  CUSTOM = "custom",
}

enum OutputFormat {
  MERMAID = "mermaid",
  SVG = "svg",
  PNG = "png",
  HTML = "html",
  JSON = "json",
  CSV = "csv",
  MARKDOWN = "markdown",
}
```

### 3.3 Graph Properties

#### Invariant 1: Directed Acyclic Graph (DAG)

- All links form a DAG converging to Root Driver
- No cycles allowed
- Validated on every model mutation

#### Invariant 2: Root Connectivity

- Every card (except Root Driver) must have path to Root Driver
- Either direct link or transitive link through other cards
- Validated on load and after mutations

#### Invariant 3: Deterministic Ordering

- Cards/links with same inputs always serialize identically
- Sorted maps, stable hashing, no timestamp dependencies (except audit history)

---

## 4. Validation Engine Architecture

### 4.1 Validation Pipeline

```text
Input (Card/Link JSON)
  │
  ├─→ [Schema Validator] ──→ Validates against schemas/*.json
  │                           - Type checking
  │                           - Required fields
  │                           - Enum constraints
  │
  ├─→ [Naming Validator] ──→ Validates conventions
  │                           - ID format: namespace:element-name
  │                           - Lowercase, hyphens only
  │                           - No reserved words
  │
  ├─→ [Reference Validator] ──→ Validates integrity
  │                             - All referenced cards exist
  │                             - No dangling links
  │                             - Types match expectations
  │
  ├─→ [Semantic Validator] ──→ Validates model rules
  │                            - DAG property (no cycles)
  │                            - Root connectivity
  │                            - Version format (semver)
  │
  └─→ Output: ValidationResult (errors, warnings, suggestions)
```

### 4.2 Validation Result Structure

```typescript
interface ValidationResult {
  valid: boolean
  errors: ValidationError[]        // Blocking issues
  warnings: ValidationWarning[]    // Non-blocking issues
  suggestions: ValidationSuggestion[] // Recommendations
}

interface ValidationError {
  code: string                      // e.g., "SCHEMA_MISMATCH"
  level: "error" | "warning" | "info"
  entity: { type: string, id: string }
  message: string
  path: string                      // JSON path to problem
  suggestion: string                // How to fix
}
```

### 4.3 Pluggable Validators

Validators are registered in a registry and can be:

- **Enabled/disabled** per validation run
- **Ordered** (e.g., schema before semantic)
- **Scoped** (per-card, per-link, model-wide)

Example plugin interface:

```typescript
interface Validator {
  name: string
  description: string
  enabled: boolean
  
  validate(entity: Card | Link | Model): ValidationResult
}
```

---

## 5. Query Engine Architecture

### 5.1 Query Capabilities

#### Graph Traversal

```typescript
class QueryEngine {
  // Find all cards reachable from source (forward direction)
  findReachable(sourceId: string): Set<string>
  
  // Find all cards that reach target (reverse direction)
  findDependents(targetId: string): Set<string>
  
  // Find shortest path between two cards
  findPath(sourceId: string, targetId: string): Card[]
  
  // Find all paths (useful for traceability)
  findAllPaths(sourceId: string, targetId: string): Card[][]
}
```

#### Filtering & Selection

```typescript
class FilterBuilder {
  byType(types: CardType[]): FilterBuilder
  byStatus(statuses: CardStatus[]): FilterBuilder
  byOwner(owners: string[]): FilterBuilder
  byConstraint(constraintIds: string[]): FilterBuilder
  byAttribute(key: string, value: any): FilterBuilder
  
  execute(): Card[]
}
```

#### Full-Text & Attribute Search

```typescript
class SearchEngine {
  // Full-text on name + description
  fullText(query: string): Card[]
  
  // Regex on IDs
  regexId(pattern: string): Card[]
  
  // Query attributes with expressions
  queryAttribute(key: string, expression: string): Card[]
}
```

#### Centered Element View

```typescript
class ElementView {
  // Center on a card, show neighborhood
  centerOn(cardId: string, depth: number = 1): {
    center: Card
    related: Card[]
    links: Link[]
    distances: Map<string, number>
  }
}
```

### 5.2 Query Optimization

- **Indexing**: Hash tables by ID, type, status, owner for O(1) lookups
- **Lazy Evaluation**: Filters composed but not evaluated until materialized
- **Graph Caching**: Adjacency lists and reverse adjacency lists cached in memory
- **Query Plan**: For complex queries, optimizer determines traversal order

---

## 6. Rendering Engine Architecture

### 6.1 Rendering Pipeline

```typescript
ArchitectureModel
  │
  ├─→ [View Interpreter] ──→ Determines scope, filter, format
  │
  ├─→ [Projection Engine] ──→ Filters model to view-specific subset
  │
  ├─→ [Layout Engine] ──→ Computes visual positions for nodes/edges
  │                        (hierarchical, force-directed, tree, etc.)
  │
  ├─→ [Format Renderer] ──→ Renders to target format
  │                          - Mermaid
  │                          - SVG
  │                          - HTML
  │                          - PNG (via Mermaid CLI or Graphviz)
  │
  └─→ Output: Diagram/Report
```

### 6.2 Projection System

A **projection** is a deterministic, reproducible view of the model:

```typescript
class Projector {
  // Hierarchy view (tree of drivers → requirements → tests)
  projectHierarchy(filter?: FilterCriteria): Diagram
  
  // Traceability matrix (requirements vs drivers vs tests)
  projectMatrix(rows: CardType, columns: CardType): Matrix
  
  // Dependency graph (all links with optional filtering)
  projectGraph(filter?: FilterCriteria): Graph
  
  // Behavior sequence (state machine, sequence diagrams)
  projectBehavior(behaviorId: string): SequenceDiagram
  
  // Custom projection (user-defined via View card)
  projectCustom(viewCard: View): CustomOutput
}
```

### 6.3 Layout Algorithms

Depending on diagram type and complexity:

- **Hierarchical**: Top-down tree layout (drivers → requirements → implementations)
- **Force-Directed**: Physics-based layout for complex dependencies
- **Layered**: Swim-lane layout for behavioral sequences
- **Circular**: Radial layout with center element (for centered element view)

### 6.4 Format Renderers

Each renderer is a separate module:

```typescript
interface Renderer {
  name: string
  formats: OutputFormat[]
  
  render(diagram: Diagram, options?: RenderOptions): string
}

// Implementations:
// - MermaidRenderer (outputs Mermaid DSL)
// - SVGRenderer (outputs SVG with styling)
// - HTMLRenderer (outputs interactive HTML)
// - PNGRenderer (requires external binary)
// - MatrixRenderer (outputs CSV, HTML table)
// - MarkdownRenderer (outputs Markdown with embedded diagrams)
```

---

## 7. Persistence & Serialization

### 7.1 File Organization

```typescript
project/
├── docs/
│   ├── cards/
│   │   ├── driver-root.json
│   │   ├── driver-*.json
│   │   ├── requirement-*.json
│   │   └── ... (one file per card)
│   ├── links/
│   │   └── links.json (all links in single file, or split by category)
│   └── views/
│       └── views.json (derived views)
├── schemas/
│   ├── card.schema.json
│   ├── link.schema.json
│   ├── *.schema.json
│   └── ... (type-specific schemas)
├── .aurora/
│   └── preferences.json (project configuration)
└── .git/ (optional: version control)
```

### 7.2 Serialization Contracts

**Cards:**

- One file per card: `docs/cards/{namespace}-{name}.json`
- Or all cards in single file: `docs/cards.json`
- Each contains full card object with all fields

**Links:**

- Single file: `docs/links.json` (recommended for atomicity)
- Or split by category: `docs/links/link-type-source.json`
- Each link includes full source and target IDs

**Views:**

- User-defined views stored as View cards
- Cached projections stored separately (optional, regenerated on demand)

### 7.3 I/O Operations

```typescript
class PersistenceLayer {
  // Load model from disk
  loadModel(projectRoot: string): Promise<ArchitectureModel>
  
  // Save model to disk (atomic or transactional)
  saveModel(model: ArchitectureModel, projectRoot: string): Promise<void>
  
  // Save individual card (with validation)
  saveCard(card: Card, projectRoot: string): Promise<void>
  
  // Save all links (atomic)
  saveLinks(links: Link[], projectRoot: string): Promise<void>
  
  // Export to external format (diagram, report)
  exportAs(model: ArchitectureModel, format: OutputFormat): Promise<string>
}
```

### 7.4 Git Integration

```typescript
class GitIntegration {
  // Semantic diff (not line-by-line)
  semanticDiff(baseModel: Model, headModel: Model): SemanticDiff
  
  // Impact analysis for PR review
  analyzeImpact(changes: SemanticDiff): ImpactAnalysis
  
  // Preserve provenance in commits
  commitWithProvenance(message: string, author: string): void
}
```

---

## 8. Application Layer Architecture

### 8.1 Core Services

#### CardService

```typescript
class CardService {
  // CRUD
  create(cardData: Partial<Card>): Card
  read(cardId: string): Card | null
  update(cardId: string, changes: Partial<Card>): Card
  delete(cardId: string): void
  
  // Bulk
  list(filter?: FilterCriteria): Card[]
  
  // Relationships
  getLinks(cardId: string, direction?: "in" | "out"): Link[]
  getRelated(cardId: string): Card[]
  getRelations(cardId: string): Card[]
}
```

#### LinkService

```typescript
class LinkService {
  // CRUD
  create(source: string, target: string, metadata?: any): Link
  read(linkId: string): Link | null
  update(linkId: string, changes: Partial<Link>): Link
  delete(linkId: string): void
  
  // Validation
  validateDAG(): boolean
  validateRootConnectivity(): boolean
}
```

#### ViewService

```typescript
class ViewService {
  // Define views
  createView(viewCard: View): void
  
  // Render views (delegates to rendering engine)
  renderView(viewId: string, format: OutputFormat): Promise<string>
  
  // Cache management
  invalidateCache(viewId: string): void
  regenerateAllCaches(): Promise<void>
}
```

#### ValidationService

```typescript
class ValidationService {
  // Validate individual entities
  validateCard(card: Card): ValidationResult
  validateLink(link: Link): ValidationResult
  
  // Validate entire model
  validateModel(model: ArchitectureModel): ValidationResult
  
  // Get report with grouping and filtering
  getValidationReport(groupBy?: string): ValidationReport
}
```

### 8.2 Transactional Semantics

For operations affecting multiple entities:

```typescript
class Transaction {
  begin(): void
  
  addCardChange(change: Change): void
  addLinkChange(change: Change): void
  addViewChange(change: Change): void
  
  validate(): ValidationResult
  commit(): Promise<void>
  rollback(): void
}

// Usage:
const tx = new Transaction()
tx.addCardChange({ type: "create", entity: newCard })
tx.addLinkChange({ type: "create", entity: newLink })
const result = tx.validate()
if (result.valid) await tx.commit()
```

### 8.3 Automatic Save & Change Tracking

Every modification to a card or link is automatically persisted with a complete change history:

**No manual save button** — Changes are debounced (2-second window) and automatically written to disk, with every change recorded in the card's audit history.

```typescript
class AutoSaveManager {
  // When a card field changes, this is called automatically
  async onCardEdited(cardId: string, changes: FieldChanges) {
    const card = this.getCard(cardId)
    
    // 1. Increment change counter
    card.change_counter++
    
    // 2. Record old values
    const oldValues = {}
    for (const [field, newValue] of Object.entries(changes)) {
      oldValues[field] = card[field]
      card[field] = newValue
    }
    
    // 3. Add history entry
    card.last_modified = new Date().toISOString()
    card.audit_history.push({
      change_number: card.change_counter,
      event: "modified",
      timestamp: card.last_modified,
      by: this.currentUser,
      fields_modified: Object.keys(changes),
      previous_values: oldValues,
      note: changes.__change_note
    })
    
    // 4. Queue for debounced save
    this.queueSave(cardId, card, { debounceMs: 2000 })
  }
  
  // Internal: perform the actual disk write
  private async flushSave(cardId: string) {
    const card = this.getCard(cardId)
    
    // Validate before writing
    const validation = await this.validator.validateCard(card)
    if (!validation.valid) {
      this.emit('save-error', { cardId, validation })
      return
    }
    
    // Write to disk atomically
    const path = this.cardPath(cardId)
    await fs.writeFile(path, JSON.stringify(card, null, 2))
    
    // Emit event for UI update
    this.emit('card-saved', {
      cardId,
      changeCounter: card.change_counter,
      lastModified: card.last_modified
    })
  }
}
```

**Design Principles**:

- **Debounced saves**: Changes don't immediately hit disk; instead, writes are batched (2-second window). This reduces I/O while ensuring no work is lost.
- **Always valid**: Every save is validated against the card schema before writing.
- **Complete history**: Every change (including creation) is recorded with old values for rollback and audit.
- **No user friction**: No "save" button; users never experience "unsaved changes" warnings.
- **Visible audit trail**: Users can inspect the complete change history including who changed what and when.
- **Heartbeat save**: Every 30 seconds, all queued changes are flushed (for long-running sessions).

**Save Triggers**:

- Field edit (debounced 2s)
- Focus blur (immediate)
- Link added/removed (immediate)
- Status change (immediate)
- App shutdown (flush all)
- Heartbeat timer (every 30s)

**UI Indicators**:

```typescript
interface CardStatusBar {
  changeCounter: number        // "Changes: 42"
  lastModified: string         // "Last modified 2 min ago"
  lastModifiedBy: string       // "by alice@example.com"
  saveStatus: SaveStatus       // "saved" | "saving" | "error"
  pendingChanges?: number      // "2 pending" (if any)
}
```

---

## 9. Presentation Layer Architecture

#### Technology Stack

- **Framework**: React + TypeScript (or Vue, Svelte)
- **State Management**: Redux or Zustand
- **Diagram Rendering**: Mermaid.js (client-side)
- **Styling**: Tailwind CSS
- **Editor**: Monaco Editor (VS Code editor component)

#### Page Hierarchy

```typescript
/
├── /editor
│   ├── /cards
│   │   ├── /create
│   │   ├── /{id}/edit
│   │   └── /{id}/view
│   ├── /links
│   │   ├── /{id}/edit
│   │   └── /{id}/view
│   └── /views
│       ├── /{id}/edit
│       └── /{id}/render
├── /dashboard
│   ├── /overview (stats, status)
│   ├── /traceability (matrix)
│   ├── /dependencies (graph)
│   └── /coverage (reports)
├── /validation
│   └── /report (validation errors/warnings)
└── /settings
    └── /preferences (project configuration)
```

#### Key UI Components

- **Card Editor**: Form-based editor with JSON preview
- **Link Editor**: Visual link creation (drag-and-drop from cards)
- **Diagram Viewer**: Interactive Mermaid diagram with click-to-navigate
- **Centered Element View**: Selected card in center with neighbors around it
- **Search Box**: Full-text, regex, attribute search
- **Filter Panel**: Status, type, owner, constraint filters
- **Validation Panel**: Real-time validation feedback
- **History Panel**: Audit trail for selected entity

### 9.2 CLI Tool

```bash
aurora init <project-name>              # Initialize new project
aurora card create <type> <name>        # Create card interactively
aurora card list [--filter=status:approved]  # List cards
aurora card view <id>                   # Display card details
aurora link create <source> <target>    # Create link
aurora validate [--strict]              # Validate entire model
aurora render <view-id> [--format=svg]  # Render a view
aurora export [--format=mermaid]        # Export entire model
aurora query <expression>               # Execute graph query
aurora diff [--base=main]               # Compare with baseline
```

### 9.3 IDE Plugin (VS Code)

- **TreeView Extension**: Navigate cards and links
- **Document Provider**: Open cards in editor with syntax highlighting
- **Validation on Save**: Real-time validation feedback
- **Diagram Preview**: Side-by-side preview of rendered diagrams
- **Quick Commands**: Create, delete, link entities via Command Palette

---

## 10. Integration & Extensibility

### 10.1 Plugin Architecture

```typescript
interface Plugin {
  name: string
  version: string
  
  onLoad(context: PluginContext): void
  onUnload(): void
}

interface PluginContext {
  cardService: CardService
  linkService: LinkService
  validators: ValidatorRegistry
  renderers: RendererRegistry
  queryEngine: QueryEngine
}

// Plugin can:
// - Register custom validators
// - Register custom renderers
// - Hook into save/load lifecycle
// - Add CLI commands
// - Add UI panels
```

### 10.2 Extension Points

1. **Custom Validators**: Domain-specific validation rules
2. **Custom Renderers**: Support for new diagram types or formats
3. **Custom Query Functions**: Domain-specific graph queries
4. **Import/Export**: Support for external formats (PlantUML, ArchiMate, SysML)
5. **Integration Hooks**: External tool integrations (Slack notifications, Jira sync, etc.)

---

## 11. Performance & Scalability

### 11.1 In-Memory Performance

- **Model Size**: Optimized for 100-10,000 cards (typical architecture projects)
- **Load Time**: < 1s for 5,000-card model
- **Query Time**: < 100ms for most traversals
- **Render Time**: < 500ms for typical diagrams

### 11.2 Optimization Strategies

- **Lazy Loading**: Load only cards/links needed for current view
- **Caching**: Cache graph adjacency lists, search indexes, rendered diagrams
- **Incremental Rendering**: Render diagrams incrementally, show placeholders
- **Web Workers**: Move heavy computation (layout, validation) to background threads

### 11.3 Scalability Considerations

For larger models (10,000+ cards):

- Consider database backend (PostgreSQL with JSON columns)
- Implement pagination and lazy loading
- Add view/projection caching layer
- Consider distributed rendering for very large diagrams

---

## 12. Security & Governance

### 12.1 Access Control

- **Read**: By default, anyone can read cards and links
- **Write**: Restricted to authorized users (configurable per project)
- **Audit Trail**: Every change tracked with author, timestamp, rationale

### 12.2 Data Integrity

- **Validation**: All data validated before write
- **Transactions**: Multi-entity changes are atomic
- **Backups**: Automatic backups of model state
- **Version History**: Full Git history preserved

### 12.3 Provenance & Traceability

- **Source Tracking**: Where did this card come from?
- **Ownership**: Who owns this card? Who approved it?
- **Change Log**: Complete audit history with before/after state
- **Lineage**: What requirements does this design satisfy?

---

## 13. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-4)

- [x] Domain model (Card, Link, View) implemented
- [ ] Persistence layer (JSON I/O)
- [ ] Basic validation engine
- [ ] Query engine (traversal, filtering)
- [ ] CLI tool (basic commands)

### Phase 2: Rendering & UI (Weeks 5-8)

- [ ] Rendering engine (Mermaid, SVG)
- [ ] View projection system
- [ ] Web UI (card editor, diagram viewer)
- [ ] Centered element view

### Phase 3: Advanced Features (Weeks 9-12)

- [ ] Git integration (semantic diff, impact analysis)
- [ ] IDE plugin (VS Code)
- [ ] Plugin system
- [ ] Caching and optimization

### Phase 4: Polish & Release (Weeks 13-16)

- [ ] Comprehensive testing
- [ ] Documentation and tutorials
- [ ] Performance tuning
- [ ] v1.0 release

---

## 14. Technology Decisions

### 14.1 Core Language

**Recommendation**: TypeScript

- Strong typing matches domain model
- JavaScript runtime available (Node.js + Browser)
- Rich ecosystem for all layers (CLI, web, IDE)
- Good performance characteristics

**Alternatives**: Python (rapid prototyping), Rust (performance, safety), Go (deployment simplicity)

### 14.2 Data Serialization

**JSON Only** (no YAML, XML, etc.)

- Canonical format per AURORA spec
- Native browser support
- No impedance mismatch with domain model

### 14.3 Validation

**AJV** for JSON Schema validation

- Fastest validator available
- Supports JSON Schema draft-07 (as used in AURORA)
- Can compile schemas to JavaScript for performance

### 14.4 Diagram Generation

**Mermaid.js** as primary

- GitHub Flavored Markdown integration
- Client-side rendering (no server needed)
- Supports all common diagram types
- Can export to SVG/PNG via Mermaid CLI

**Graphviz** as fallback for complex graphs

- More advanced layout algorithms
- Better for large graphs
- Requires external binary

### 14.5 Web Framework

**React** for web UI

- Largest ecosystem
- Strong TypeScript support
- Component-based, matches modular architecture

---

## 15. Testing Strategy

### 15.1 Test Pyramid

```bash
          /\
         /  \  Integration Tests
        /────\  (API + Rendering)
       /      \
      /        \
     /──────────\  Unit Tests
    /            \ (Services, Validators, Engines)
   /              \
  /────────────────\
```

### 15.2 Test Coverage

- **Unit Tests**: 80%+ coverage of services, validators, query engine
- **Integration Tests**: Scenario tests (create card, link, render view)
- **E2E Tests**: User workflows (full editor lifecycle)
- **Property-Based Tests**: Graph properties (DAG, root connectivity) hold under all mutations

---

## 16. Documentation

### 16.1 Developer Documentation

- **Architecture Guide** (this document)
- **API Reference** (auto-generated from TypeDoc comments)
- **Plugin Development Guide**
- **Contributing Guide**

### 16.2 User Documentation

- **Getting Started Guide**
- **Card & Link Reference**
- **View & Rendering Guide**
- **CLI Command Reference**
- **Web UI Tutorial**

---

## 17. Open Design Questions

1. **Database vs File-Based**: For small-medium projects, JSON files in Git are ideal. At what size do we switch to database?

2. **Real-Time Collaboration**: Should multiple users be able to edit simultaneously? Requires CRDTs or operational transformation.

3. **Plugin Security**: Should plugins be sandboxed? How do we prevent malicious plugins from corrupting the model?

4. **Custom Attributes**: How much flexibility in card attributes? Too much breaks schema validation; too little limits extensibility.

5. **View Caching**: Should rendered views be cached in Git? Increases size but improves performance and enables offline viewing.

---

## 18. Conclusion

AURORA Tooling is designed as a clean, layered, domain-driven system where the canonical representation (JSON cards and links) is maintained with absolute integrity, and all UI/rendering/querying is built on top of that foundation.

The architecture enables:

- **Deterministic behavior**: Same input → same output, always
- **Full traceability**: Every change recorded, every relationship explicit
- **Extensibility**: Plugins can add validators, renderers, and custom logic
- **Scalability**: From small projects to enterprise architectures
- **Accessibility**: Available as web UI, CLI, and IDE plugin

---

**Next Steps**: For each subsystem (Query Engine, Rendering Engine, Persistence Layer, UI), create detailed design specifications and implementation guides.
