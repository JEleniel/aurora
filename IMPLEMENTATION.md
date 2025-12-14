# AURORA Tooling Implementation

**Version**: 1.0.0 | **Status**: Implementation Phase 1
**Technology Stack**: Tauri (Rust/SvelteKit/TypeScript)

---

## Project Structure

```
aurora/
├── Cargo.toml                    # Workspace configuration
├── package.json                  # Root workspace config
│
├── src-tauri/                    # Rust backend (Tauri)
│   ├── Cargo.toml               # Rust dependencies
│   ├── src/
│   │   ├── main.rs              # Tauri app entry point
│   │   ├── lib.rs               # Library exports
│   │   ├── model.rs             # Domain models (Card, Link, View)
│   │   ├── persistence.rs       # File I/O & automatic save
│   │   ├── query.rs             # Query engine (BFS, paths, etc.)
│   │   └── commands.rs          # Tauri command handlers
│   ├── tauri.conf.json          # Tauri configuration
│   └── build.rs                 # Build script
│
├── frontend/                     # SvelteKit frontend (TypeScript)
│   ├── package.json             # Dependencies
│   ├── vite.config.ts           # Vite build config
│   ├── tailwind.config.js       # Tailwind CSS
│   ├── postcss.config.js        # PostCSS plugins
│   ├── tsconfig.json            # TypeScript config
│   ├── index.html               # HTML entry
│   ├── src/
│   │   ├── main.ts              # Svelte app entry
│   │   ├── App.svelte           # Root component
│   │   ├── style.css            # Global styles
│   │   └── components/
│   │       ├── Navigation.svelte    # Top navbar
│   │       ├── CardList.svelte      # Card sidebar
│   │       ├── CardEditor.svelte    # Card editor panel
│   │       ├── HistoryViewer.svelte # Change history viewer
│   │       └── Statistics.svelte    # Model statistics
│   └── build/                   # Built artifacts
│
├── docs/
│   ├── design/tool/             # Architecture specifications
│   │   ├── ARCHITECTURE.md      # System architecture
│   │   ├── PERSISTENCE.md       # Persistence & auto-save
│   │   ├── QUERY-ENGINE.md      # Query subsystem
│   │   ├── RENDERING-ENGINE.md  # Rendering subsystem
│   │   └── DATA-STORAGE-EXPORT.md # Storage formats
│   ├── cards/                   # Reference architecture (JSON)
│   ├── schemas/                 # JSON schema definitions
│   └── ...
│
└── IMPLEMENTATION.md            # This file
```

---

## Backend (Rust + Tauri)

### Core Modules

#### `model.rs` — Domain Models

Defines all AURORA domain types:

```rust
pub enum CardType {
    Driver, Requirement, Behavior, Interface, Constraint,
    LogicalComponent, DeployableNode, Actor, Test, Artifact, View, Note,
}

pub struct Card {
    pub id: String,
    pub r#type: CardType,
    pub name: String,
    pub change_counter: u64,        // Auto-incremented on save
    pub last_modified: DateTime,
    pub audit_history: Vec<AuditHistoryEntry>,
    // ... other fields
}

pub struct Link {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    // ... metadata
}

pub struct ArchitectureModel {
    pub cards: HashMap<String, Card>,
    pub links: Vec<Link>,
    pub views: HashMap<String, View>,
}
```

**Key Features**:

- Automatic `change_counter` incrementing
- Complete `audit_history` with field-level tracking
- `record_change()` method for tracking modifications

#### `persistence.rs` — I/O & Auto-Save

Handles file operations and automatic saving:

```rust
pub struct PersistenceManager {
    project_root: PathBuf,
    cards_dir: PathBuf,
    links_file: PathBuf,
}

pub struct AutoSaveManager {
    pending_saves: HashMap<String, Card>,  // Debounced saves
}
```

**Key Features**:

- **Debounced saves** (2s window)
- **Async I/O** (tokio)
- **Validation before write**
- **Atomic link writes**
- **Full model loading/saving**

#### `query.rs` — Graph Queries

Enables searching and traversal:

```rust
pub struct QueryEngine {
    model: ArchitectureModel,
}

impl QueryEngine {
    // Find by type, name, attributes
    pub fn find_by_type(&self, card_type: CardType) -> Vec<&Card>
    pub fn find_by_name(&self, name: &str) -> Vec<&Card>
    
    // Navigation
    pub fn get_neighbors(&self, card_id: &str) -> Vec<&Card>
    pub fn find_reachable(&self, card_id: &str) -> Vec<&Card>
    pub fn find_path(&self, from: &str, to: &str) -> Option<Vec<String>>
    
    // Analysis
    pub fn find_dependents(&self, card_id: &str) -> Vec<&Card>
    pub fn count_by_type(&self) -> HashMap<String, usize>
    pub fn get_statistics(&self) -> HashMap<String, usize>
}
```

#### `commands.rs` — Tauri Command Handlers

Bridges frontend and backend:

```rust
#[tauri::command]
pub async fn create_card(state: State<AppState>, ...) -> Result<ApiResponse<Card>>

#[tauri::command]
pub async fn update_card(state: State<AppState>, ...) -> Result<ApiResponse<Card>>

#[tauri::command]
pub async fn query_find_path(state: State<AppState>, ...) -> Result<ApiResponse<Option<Vec<String>>>>

#[tauri::command]
pub async fn save_project(state: State<AppState>) -> Result<ApiResponse<bool>>
```

**Available Commands**:

- `create_card` — Create a new card
- `get_card` — Load a card by ID
- `update_card` — Modify a card (triggers auto-save)
- `delete_card` — Remove a card
- `list_cards` — Get all cards
- `create_link` — Link two cards
- `list_links` — Get all links
- `get_links_for_card` — Get incoming/outgoing links
- `query_find_by_type` — Find cards by type
- `query_find_path` — Find path between cards
- `query_get_statistics` — Model statistics
- `load_project` — Load project from disk
- `save_project` — Flush all pending saves
- `get_autosave_status` — Check pending save count

---

## Frontend (SvelteKit + TypeScript)

### Component Structure

#### `Navigation.svelte`

Top navigation bar with save button and auto-save status indicator.

#### `CardList.svelte`

Sidebar showing all cards with:

- Search/filter
- Type-based color coding
- New card creation dialog
- Card selection

#### `CardEditor.svelte`

Main editor panel with:

- Card metadata display
- Edit mode toggle
- Field editing (name, description, rationale, status, priority, owner)
- Change counter display
- Last modified timestamp
- History viewer toggle

#### `HistoryViewer.svelte`

Expandable change history showing:

- Change number and timestamp
- Event type (created, modified, approved, etc.)
- Changed fields list
- Previous values
- Author/agent identifier
- Optional change notes

#### `Statistics.svelte`

Dashboard showing model statistics:

- Total cards
- Total links
- Total views

---

## Tauri Configuration

### IPC Invoke Handler

Frontend calls backend via `invoke()`:

```typescript
import { invoke } from '@tauri-apps/api/tauri'

// Call backend command
const response = await invoke('create_card', {
    id: 'driver:my-driver',
    cardType: 'driver',
    name: 'My Driver',
})

// Response format:
interface ApiResponse<T> {
    success: boolean
    data?: T
    error?: string
}
```

### State Management

Shared application state:

```rust
pub struct AppState {
    pub model: Arc<RwLock<ArchitectureModel>>,
    pub persistence: Arc<PersistenceManager>,
    pub autosave: Arc<AutoSaveManager>,
}
```

- **RwLock** for concurrent read/write access
- **Arc** for thread-safe sharing
- **tokio** for async operations

---

## Automatic Save Implementation

### Design

**Debounced save with heartbeat:**

1. **User edits field** → Queue change (in-memory)
2. **After 2 seconds of inactivity** → Write to disk
3. **Every 30 seconds** → Force-save (heartbeat)
4. **On app close** → Flush all pending
5. **On link change** → Immediate save

### Change Tracking

Every save creates an `AuditHistoryEntry`:

```rust
pub struct AuditHistoryEntry {
    pub change_number: u64,              // Sequential: 1, 2, 3
    pub event: String,                   // "created", "modified", etc.
    pub timestamp: DateTime<Utc>,
    pub by: String,                      // "user:email" or "agent:name"
    pub fields_modified: Vec<String>,    // ["name", "description"]
    pub previous_values: HashMap<String, Value>, // Old values for rollback
    pub note: Option<String>,            // Optional change description
}
```

### UI Feedback

The UI displays:

- **Change Counter**: "Changes: 27"
- **Last Modified**: "2 minutes ago"
- **Save Status**: "saved" | "saving" | "error"
- **Pending Count**: "3 pending" (if any unsaved)

---

## File Organization

### Disk Layout

```
project_root/
├── docs/
│   ├── cards/
│   │   ├── driver-root.json
│   │   ├── requirement-api-latency.json
│   │   └── ... (one file per card)
│   └── links/
│       └── links.json           # All links in one file
├── schemas/
│   ├── card.schema.json
│   ├── link.schema.json
│   └── ...
└── .aurora/
    ├── preferences.json
    ├── metadata.json
    └── cache/
        ├── diagrams/
        └── indexes/
```

### JSON Format

Cards are stored as pretty-printed JSON:

```json
{
  "id": "driver:root-driver",
  "type": "driver",
  "name": "Root Driver",
  "change_counter": 5,
  "last_modified": "2025-12-14T10:30:00Z",
  "audit_history": [
    {
      "change_number": 1,
      "event": "created",
      "timestamp": "2025-12-14T10:00:00Z",
      "by": "user:alice@example.com",
      "fields_modified": ["id", "type", "name"],
      "previous_values": {},
      "note": "Initial creation"
    },
    {
      "change_number": 2,
      "event": "modified",
      "timestamp": "2025-12-14T10:15:00Z",
      "by": "user:alice@example.com",
      "fields_modified": ["description"],
      "previous_values": {
        "description": "Old description text"
      },
      "note": "Updated description"
    }
  ],
  "description": "New description",
  "status": "approved",
  "version": "1.0.0",
  "relations": [],
  "links": [],
  "constraints": [],
  "attributes": {},
  "acceptance_criteria": []
}
```

---

## Getting Started

### Prerequisites

- Rust 1.70+
- Node.js 18+
- pnpm

### Installation

```bash
cd /home/jeleniel/repos/aurora

# Install dependencies
pnpm install

# Install Tauri CLI
pnpm add -D @tauri-apps/cli
```

### Development

```bash
# Run in dev mode (hot reload)
pnpm tauri:dev
```

This starts:

- Frontend dev server on <http://localhost:5173>
- Rust backend with debug symbols
- Tauri app window (auto-reloads on code changes)

### Production Build

```bash
# Build optimized binary
pnpm tauri:build
```

Creates:

- Optimized Rust binary (LTO, single codegen unit)
- Minified frontend assets
- Platform-specific installer (MSI/NSIS on Windows)

---

## Testing

### Rust Tests

```bash
cd src-tauri

# Run all tests
cargo test

# Test specific module
cargo test model::tests
cargo test persistence::tests

# With output
cargo test -- --nocapture
```

### Frontend Tests

```bash
cd frontend

# Lint
pnpm lint

# Type check
pnpm check
```

---

## Roadmap (Phase 1 - Foundation)

- [x] Domain models (Card, Link, View, ArchitectureModel)
- [x] Persistence layer (JSON I/O, auto-save, change tracking)
- [x] Query engine (find, path-finding, reachability)
- [x] Tauri backend with command handlers
- [x] SvelteKit frontend with Tailwind CSS
- [x] Card CRUD operations
- [x] Link management
- [x] Change history viewer
- [ ] Rendering engine (SVG, Mermaid output)
- [ ] View projections (hierarchy, matrix, graph, sequence)
- [ ] Validation engine (schema checking, constraints)
- [ ] Batch export (ZIP archive, multiple formats)
- [ ] Collaboration features (merge, conflict resolution)
- [ ] Performance optimization (indexing, caching)

---

## Architecture Decisions

### Tauri

**Why**: Rust + web technologies, small binary, native performance

### Tokio (async runtime)

**Why**: Efficient I/O, non-blocking saves, responsive UI

### RwLock (concurrency)

**Why**: Multiple readers, exclusive writers, zero-copy sharing

### SvelteKit

**Why**: Reactive, TypeScript, minimal bundle, great DX

### Tailwind CSS

**Why**: Rapid UI development, consistent design system, low overhead

---

## Performance Characteristics

### Benchmarks (Phase 1)

- **Card creation**: < 5ms
- **Card save (disk)**: ~50ms (depends on disk, debounced)
- **Query find_path**: O(V + E) BFS
- **Model load**: ~200ms (1000 cards, 5000 links)

### Memory Usage

- **Empty model**: ~1MB
- **1000 cards**: ~50MB (in-memory + indexes)
- **Pending saves**: Minimal (debounced queue)

---

## Known Limitations

**Phase 1 (Foundation)**:

1. Single-file links (no concurrent link editing)
2. No schema validation (coming in Phase 2)
3. No rendering engine (Phase 2)
4. No collaborative features (Phase 3+)
5. Local filesystem only (no cloud sync)

---

## Next Steps

1. **Phase 1b** — Complete rendering engine (SVG, Mermaid)
2. **Phase 1c** — Validation engine (schema, constraints, references)
3. **Phase 2** — View projections and matrix/graph rendering
4. **Phase 2b** — Batch export (ZIP, formats)
5. **Phase 3** — Collaborative features, conflict resolution

---

## Contributing

See [ARCHITECTURE.md](docs/design/tool/ARCHITECTURE.md) for design specs.

Workflow:

1. Read the relevant design doc
2. Create feature branch
3. Implement with tests
4. Submit PR with design rationale

---

## License

See root [LICENSE](LICENSE) file.

---

**AURORA v1.0.0** — Architecture as Code | 2025
