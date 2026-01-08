# AURORA Persistence Layer Design

**Component**: Persistence Layer | **Status**: Design | **Complexity**: Medium

---

## Overview

The Persistence Layer handles loading, saving, versioning, and caching of AURORA models. It abstracts the storage mechanism (file system, database, cloud) and provides atomic transactions, version control integration, and backup capabilities.

---

## 1. Storage Architecture

### Design Principle

**Canonical Format**: JSON files in a Git-friendly structure

- Easy to version (line-by-line diffs)
- Human-readable (no binary)
- Standard format (JSON Schemas are canonical)
- Supports audit trails (Git history = change log)

### File Organization

```text
project/
├── docs/
│   ├── cards/
│   │   ├── driver-root.json          # Root Driver
│   │   ├── driver-performance.json   # Other drivers
│   │   ├── driver-security.json
│   │   ├── requirement-api-latency.json
│   │   ├── requirement-*.json        # All requirements
│   │   ├── behavior-*.json           # Behaviors
│   │   ├── interface-*.json          # Interfaces
│   │   ├── constraint-*.json         # Constraints
│   │   ├── actor-*.json              # Actors
│   │   ├── test-*.json               # Tests
│   │   ├── artifact-*.json           # Artifacts
│   │   ├── logical-component-*.json  # Components
│   │   ├── deployable-node-*.json    # Deployment targets
│   │   ├── view-*.json               # Views
│   │   └── note-*.json               # Notes
│   │
│   └── links/
│       └── links.json                # All links (single atomic file)
│
├── schemas/
│   ├── card.schema.json
│   ├── link.schema.json
│   └── *.schema.json                 # Type-specific schemas
│
├── .aurora/
│   ├── preferences.json              # Project config
│   ├── metadata.json                 # Project metadata
│   └── cache/
│       ├── diagrams/                 # Cached rendered diagrams
│       └── indexes/                  # Cached query indexes
│
└── .git/                             # Version control (optional but recommended)
    └── hooks/
        └── pre-commit                # Validation before commit
```

### Rationale for File Organization

1. **One file per card**: Independent editing, granular diffs, easy merge
2. **Single links file**: Atomic link updates (prevents orphaned references)
3. **Schemas in repo**: Version control enables schema evolution
4. **Preferences separate**: Project settings isolated from model
5. **Cache external**: Can be .gitignore'd, regenerated on demand

---

## 2. Automatic Save & Change Tracking

### Change Counter & History

Every card tracks changes with automatic incrementing and full history:

```typescript
interface CardChangeEntry {
  change_number: number              // Sequential: 1, 2, 3, ...
  event: string                      // "created" | "modified" | "approved" | ...
  timestamp: ISO8601DateTime         // Auto-set to current time
  by: string                         // User, agent, or system identifier
  fields_modified: string[]          // List of changed field names
  previous_values: Record<string, any> // Old values for rollback
  note?: string                      // Human-readable summary
}

// On card creation:
{
  change_counter: 1,
  last_modified: "2025-12-14T10:30:00Z",
  audit_history: [
    {
      change_number: 1,
      event: "created",
      timestamp: "2025-12-14T10:30:00Z",
      by: "user:alice@example.com",
      fields_modified: ["id", "type", "name", "description"],
      previous_values: {},
      note: "Initial card creation"
    }
  ]
}

// On subsequent edits:
// - change_counter increments: 1 → 2 → 3
// - new entry added to audit_history
// - previous_values captures old state
// - last_modified timestamp updated
```

### Automatic Save Strategy

The system implements a **debounced automatic save** approach:

```typescript
class AutoSaveManager {
  private saveQueue: Map<string, Card> = new Map()
  private saveDebounceMs = 2000        // Wait 2s after last edit
  private saveIntervalMs = 30000       // Force save every 30s (heartbeat)
  private debounceTimers = new Map<string, NodeJS.Timeout>()
  private autoSaveInterval: NodeJS.Timeout
  
  async onCardEdited(cardId: string, changes: FieldChanges) {
    const card = this.getCard(cardId)
    
    // 1. Apply changes to in-memory card
    const oldValues = {}
    for (const [field, newValue] of Object.entries(changes)) {
      oldValues[field] = card[field]
      card[field] = newValue
    }
    
    // 2. Update metadata
    card.change_counter++
    card.last_modified = new Date().toISOString()
    card.audit_history.push({
      change_number: card.change_counter,
      event: "modified",
      timestamp: card.last_modified,
      by: this.currentUser,
      fields_modified: Object.keys(changes),
      previous_values: oldValues,
      note: changes.__change_note  // Optional user note
    })
    
    // 3. Queue for save
    this.saveQueue.set(cardId, card)
    
    // 4. Cancel existing debounce timer
    if (this.debounceTimers.has(cardId)) {
      clearTimeout(this.debounceTimers.get(cardId)!)
    }
    
    // 5. Schedule debounced save
    const timer = setTimeout(
      () => this.flushSave(cardId),
      this.saveDebounceMs
    )
    this.debounceTimers.set(cardId, timer)
  }
  
  private async flushSave(cardId: string) {
    const card = this.saveQueue.get(cardId)
    if (!card) return
    
    try {
      // Validate card against schema
      await this.schema.validate(card)
      
      // Write to disk
      const path = this.cardPath(cardId)
      await fs.writeFile(path, JSON.stringify(card, null, 2))
      
      // Update in-memory index
      this.cardIndex.set(cardId, card)
      
      // Emit change event
      this.emit('card-saved', { cardId, changeCounter: card.change_counter })
      
      // Clean up
      this.saveQueue.delete(cardId)
      this.debounceTimers.delete(cardId)
      
    } catch (error) {
      this.emit('save-error', { cardId, error })
      // Keep in queue for retry
    }
  }
  
  // Force all pending saves (on blur, app close, etc.)
  async flushAll() {
    const promises = Array.from(this.saveQueue.keys())
      .map(cardId => this.flushSave(cardId))
    await Promise.all(promises)
  }
  
  // Heartbeat: periodic force-save for long-running sessions
  private startHeartbeat() {
    this.autoSaveInterval = setInterval(
      () => this.flushAll(),
      this.saveIntervalMs
    )
  }
}
```

### Automatic Save Triggers

Saves are triggered automatically:

- **On field change** — Debounced 2 seconds after last keystroke
- **On focus blur** — When user leaves a field/card editor
- **On app close** — Flush all pending changes before shutdown
- **Heartbeat timer** — Periodic force-save every 30 seconds (long sessions)
- **On link changes** — When card is linked/unlinked from other cards
- **On status change** — Explicit status transitions (approved, deprecated, etc.)

### No Manual Save Button

**Design Decision**: The UI does NOT include a save button. Changes are invisible to the user but immediately tracked and persisted. This:

- Reduces user friction (no save dance)
- Ensures no unsaved work is lost
- Creates complete audit trail
- Enables true collaboration (changes visible to others quickly)
- Prevents "stale" UI state

### Change Summary UI

The UI shows:

```typescript
interface CardStatusIndicator {
  changeCounter: number              // "Changes: 27"
  lastModified: DateTime             // "Last modified 2 minutes ago"
  lastModifiedBy: string             // "by alice@example.com"
  pendingChanges: number             // "2 pending" (queued but not saved)
  saveStatus: 'saved' | 'saving' | 'error'
}
```

### History Viewing

Users can inspect the complete change history:

```typescript
interface HistoryViewer {
  // Show all changes with full diffs
  async getChangeHistory(cardId: string): Promise<CardChangeEntry[]>
  
  // Inspect specific change
  async getChange(cardId: string, changeNumber: number): Promise<{
    before: Card
    after: Card
    diff: FieldDiff[]
  }>
  
  // Rollback to previous version
  async rollback(cardId: string, changeNumber: number, reason: string)
}
```

---

## 4. I/O Operations

### Loading Model

```typescript
class ModelLoader {
  // Load entire model from disk
  async loadModel(projectRoot: string): Promise<ArchitectureModel> {
    // 1. Load all cards from docs/cards/*.json
    const cards = await this.loadCards(`${projectRoot}/docs/cards`)
    
    // 2. Load links from docs/links/links.json
    const links = await this.loadLinks(`${projectRoot}/docs/links/links.json`)
    
    // 3. Load project metadata
    const metadata = await this.loadMetadata(`${projectRoot}/.aurora/metadata.json`)
    
    // 4. Validate all references resolve
    this.validateReferences(cards, links)
    
    // 5. Build indexes
    const model = new ArchitectureModel(cards, links, metadata)
    model.buildIndexes()
    
    // 6. Validate DAG and root connectivity
    this.validateInvariants(model)
    
    return model
  }

  private async loadCards(cardsDir: string): Promise<Map<string, Card>> {
    const cards = new Map<string, Card>()
    
    // Find all *.json files
    const files = await fs.readdir(cardsDir)
    for (const file of files.filter(f => f.endsWith('.json'))) {
      const path = `${cardsDir}/${file}`
      const content = await fs.readFile(path, 'utf-8')
      const card = JSON.parse(content) as Card
      
      // Validate against schema
      const valid = ajv.validate(cardSchema, card)
      if (!valid) throw new SchemaValidationError(file, ajv.errors)
      
      cards.set(card.id, card)
    }
    
    return cards
  }

  private async loadLinks(linksPath: string): Promise<Map<string, Link>> {
    const content = await fs.readFile(linksPath, 'utf-8')
    const linkArray = JSON.parse(content) as Link[]
    
    const links = new Map<string, Link>()
    for (const link of linkArray) {
      // Validate against link schema
      const valid = ajv.validate(linkSchema, link)
      if (!valid) throw new SchemaValidationError(`links.json`, ajv.errors)
      
      links.set(link.id, link)
    }
    
    return links
  }

  private validateReferences(cards: Map<string, Card>, links: Map<string, Link>): void {
    // All source and target IDs must exist
    for (const link of links.values()) {
      if (!cards.has(link.source_id)) {
        throw new ReferenceError(`Link ${link.id} references non-existent source ${link.source_id}`)
      }
      if (!cards.has(link.target_id)) {
        throw new ReferenceError(`Link ${link.id} references non-existent target ${link.target_id}`)
      }
    }
  }

  private validateInvariants(model: ArchitectureModel): void {
    // Check DAG property
    if (hasCycle(model.graph)) {
      throw new DAGViolationError("Model contains cycles")
    }
    
    // Check root connectivity
    for (const card of model.cards) {
      if (card.id === model.rootDriverId) continue
      if (!isConnectedToRoot(card, model)) {
        throw new RootConnectivityError(`Card ${card.id} not connected to root`)
      }
    }
  }
}
```

### Saving Model

```typescript
class ModelSaver {
  // Save entire model atomically
  async saveModel(model: ArchitectureModel, projectRoot: string): Promise<void> {
    // Use transactional save: write to temp, then atomic rename
    const tempRoot = `${projectRoot}/.aurora/tmp-${Date.now()}`
    
    try {
      // 1. Save all cards
      await this.saveCards(model.cards, `${tempRoot}/docs/cards`)
      
      // 2. Save links (atomic)
      await this.saveLinks(model.links, `${tempRoot}/docs/links/links.json`)
      
      // 3. Save metadata
      await this.saveMetadata(model.metadata, `${tempRoot}/.aurora/metadata.json`)
      
      // 4. Validate before commit
      const loaded = await ModelLoader.loadModel(tempRoot)
      
      // 5. Atomic rename (atomic on modern filesystems)
      await fs.rename(tempRoot, projectRoot)
      
    } catch (error) {
      // Cleanup temp directory
      await fs.rm(tempRoot, { recursive: true })
      throw error
    }
  }

  async saveCard(card: Card, projectRoot: string): Promise<void> {
    // Validate
    const valid = ajv.validate(cardSchema, card)
    if (!valid) throw new SchemaValidationError(card.id, ajv.errors)
    
    // Save to docs/cards/{id}.json
    const path = `${projectRoot}/docs/cards/${card.id}.json`
    const content = JSON.stringify(card, null, 2)
    
    // Write to temp, atomic rename
    const tempPath = `${path}.tmp`
    await fs.writeFile(tempPath, content, 'utf-8')
    await fs.rename(tempPath, path)
  }

  async saveLinks(links: Map<string, Link>, linksPath: string): Promise<void> {
    // Validate all
    const linkArray = Array.from(links.values())
    for (const link of linkArray) {
      const valid = ajv.validate(linkSchema, link)
      if (!valid) throw new SchemaValidationError(link.id, ajv.errors)
    }
    
    // Save atomically
    const tempPath = `${linksPath}.tmp`
    const content = JSON.stringify(linkArray, null, 2)
    await fs.writeFile(tempPath, content, 'utf-8')
    await fs.rename(tempPath, linksPath)
  }
}
```

---

## 5. Transactional Semantics

### Transaction Model

```typescript
class Transaction {
  private changes: Change[] = []
  private snapshot: ArchitectureModel
  private journal: ChangeLog = []

  constructor(private model: ArchitectureModel) {
    // Snapshot current state for rollback
    this.snapshot = this.model.clone()
  }

  // Register changes (not applied yet)
  createCard(card: Partial<Card>): Card {
    const fullCard = Card.create(card)
    this.changes.push({ type: 'card:create', entity: fullCard })
    return fullCard
  }

  updateCard(cardId: string, changes: Partial<Card>): Card {
    const card = this.model.cards.get(cardId)
    if (!card) throw new NotFoundError(`Card ${cardId}`)
    
    const updated = { ...card, ...changes }
    this.changes.push({ type: 'card:update', entity: updated, before: card })
    return updated
  }

  createLink(source: string, target: string, metadata?: any): Link {
    const link = Link.create(source, target, metadata)
    this.changes.push({ type: 'link:create', entity: link })
    return link
  }

  deleteCard(cardId: string): void {
    const card = this.model.cards.get(cardId)
    if (!card) throw new NotFoundError(`Card ${cardId}`)
    
    this.changes.push({ type: 'card:delete', entity: card })
  }

  deleteLink(linkId: string): void {
    const link = this.model.links.get(linkId)
    if (!link) throw new NotFoundError(`Link ${linkId}`)
    
    this.changes.push({ type: 'link:delete', entity: link })
  }

  // Validate all changes together
  validate(): ValidationResult {
    const tempModel = this.snapshot.clone()
    
    // Apply all changes to temp model
    for (const change of this.changes) {
      this.applyChange(tempModel, change)
    }
    
    // Validate temp model
    return validator.validate(tempModel)
  }

  // Commit changes to model
  async commit(author: string, message: string): Promise<void> {
    // 1. Validate
    const result = this.validate()
    if (!result.valid) throw new ValidationError(result.errors)
    
    // 2. Apply changes to model
    for (const change of this.changes) {
      this.applyChange(this.model, change)
    }
    
    // 3. Update audit history
    for (const change of this.changes) {
      this.journal.push({
        event: change.type,
        by: author,
        event_time: new Date(),
        note: message,
        before: change.before,
        after: change.entity,
      })
    }
    
    // 4. Persist to disk
    await ModelSaver.saveModel(this.model, this.model.projectRoot)
    
    // 5. Commit to Git (if enabled)
    if (this.model.gitEnabled) {
      await this.commitToGit(author, message)
    }
  }

  // Rollback to snapshot
  rollback(): void {
    this.model = this.snapshot.clone()
    this.changes = []
  }

  private applyChange(model: ArchitectureModel, change: Change): void {
    switch (change.type) {
      case 'card:create':
        model.cards.set(change.entity.id, change.entity)
        break
      case 'card:update':
        model.cards.set(change.entity.id, change.entity)
        break
      case 'card:delete':
        model.cards.delete(change.entity.id)
        break
      case 'link:create':
        model.links.set(change.entity.id, change.entity)
        break
      case 'link:delete':
        model.links.delete(change.entity.id)
        break
    }
  }
}
```

---

## 6. Caching Strategy

### Index Cache

```typescript
class IndexCache {
  // In-memory indexes rebuilt on load
  private byId: Map<string, Card>
  private byType: Map<CardType, Card[]>
  private byStatus: Map<CardStatus, Card[]>
  private outgoingLinks: Map<string, Link[]>
  private incomingLinks: Map<string, Link[]>

  // Invalidate on mutation
  invalidateCardId(cardId: string): void {
    delete this.byId[cardId]
  }

  invalidateAll(): void {
    this.byId.clear()
    this.byType.clear()
    this.byStatus.clear()
    this.outgoingLinks.clear()
    this.incomingLinks.clear()
  }
}
```

### Diagram Cache

```typescript
class DiagramCache {
  // Cache rendered diagrams by view ID + format
  private cache: Map<string, Map<OutputFormat, string>> = new Map()

  getCached(viewId: string, format: OutputFormat): string | null {
    return this.cache.get(viewId)?.get(format) ?? null
  }

  cache(viewId: string, format: OutputFormat, output: string): void {
    if (!this.cache.has(viewId)) {
      this.cache.set(viewId, new Map())
    }
    this.cache.get(viewId)!.set(format, output)
  }

  invalidateView(viewId: string): void {
    this.cache.delete(viewId)
  }

  invalidateAll(): void {
    this.cache.clear()
  }

  // Persist cache to disk
  async save(projectRoot: string): Promise<void> {
    const cacheDir = `${projectRoot}/.aurora/cache`
    // ... serialize cache to .cache files
  }

  async load(projectRoot: string): Promise<void> {
    const cacheDir = `${projectRoot}/.aurora/cache`
    // ... deserialize cache from .cache files
  }
}
```

---

## 7. Git Integration

### Semantic Diff

```typescript
class GitIntegration {
  // Compare two models semantically (not line-by-line)
  semanticDiff(baseModel: Model, headModel: Model): SemanticDiff {
    return {
      cardsAdded: this.findAddedCards(baseModel, headModel),
      cardsRemoved: this.findRemovedCards(baseModel, headModel),
      cardsModified: this.findModifiedCards(baseModel, headModel),
      linksAdded: this.findAddedLinks(baseModel, headModel),
      linksRemoved: this.findRemovedLinks(baseModel, headModel),
    }
  }

  // Analyze impact of changes
  analyzeImpact(diff: SemanticDiff): ImpactAnalysis {
    const affected = new Set<string>()
    
    // Cards removed affects downstream dependents
    for (const card of diff.cardsRemoved) {
      for (const dependent of this.getDependents(card)) {
        affected.add(dependent.id)
      }
    }
    
    // Links removed affects traceability
    for (const link of diff.linksRemoved) {
      affected.add(link.target_id)
    }
    
    return {
      directlyAffected: diff.cardsModified.concat(diff.cardsRemoved),
      transitivelyAffected: Array.from(affected),
      riskLevel: this.assessRiskLevel(diff),
    }
  }

  private assessRiskLevel(diff: SemanticDiff): "low" | "medium" | "high" {
    if (diff.cardsRemoved.length > 0) return "high"
    if (diff.cardsRemoved.length + diff.linksRemoved.length > 5) return "high"
    if (diff.cardsModified.length > 10) return "medium"
    return "low"
  }

  // Commit with semantic metadata
  async commitWithProvenance(
    message: string,
    author: string,
    diff: SemanticDiff
  ): Promise<void> {
    const commitMetadata = {
      model_changes: diff,
      author: author,
      timestamp: new Date(),
    }
    
    // Store metadata in commit message or separate file
    const commitMsg = `${message}\n\n${JSON.stringify(commitMetadata, null, 2)}`
    
    await this.gitExec(['add', '.'])
    await this.gitExec(['commit', '-m', commitMsg])
  }
}
```

### Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Validate all JSON files
for file in docs/cards/*.json docs/links/links.json; do
  if ! jq . "$file" > /dev/null; then
    echo "Invalid JSON: $file"
    exit 1
  fi
done

# Validate against schemas
npx ajv validate -s schemas/card.schema.json \
  -d docs/cards/*.json || exit 1
npx ajv validate -s schemas/link.schema.json \
  -d docs/links/links.json || exit 1

# Run custom validation
node scripts/validate-model.js || exit 1

exit 0
```

---

## 8. Backup & Recovery

### Backup Strategy

```typescript
class BackupManager {
  // Create backup before major operations
  async createBackup(model: ArchitectureModel): Promise<string> {
    const timestamp = new Date().toISOString()
    const backupDir = `.aurora/backups/backup-${timestamp}`
    
    // Copy entire docs/ and schemas/ to backup
    await fs.cp(model.projectRoot, backupDir, { recursive: true })
    
    return backupDir
  }

  // List available backups
  async listBackups(projectRoot: string): Promise<BackupInfo[]> {
    const backupDir = `${projectRoot}/.aurora/backups`
    const files = await fs.readdir(backupDir)
    
    return files
      .filter(f => f.startsWith('backup-'))
      .map(f => ({
        timestamp: f.replace('backup-', ''),
        path: `${backupDir}/${f}`,
      }))
  }

  // Restore from backup
  async restore(model: ArchitectureModel, backupPath: string): Promise<void> {
    // Create safety backup first
    await this.createBackup(model)
    
    // Copy backup to current location
    await fs.cp(backupPath, model.projectRoot, { recursive: true, force: true })
    
    // Reload model
    model = await ModelLoader.loadModel(model.projectRoot)
  }
}
```

---

## 9. Export Operations

### Export to External Formats

```typescript
class ModelExporter {
  // Export to various formats
  async export(model: ArchitectureModel, format: ExportFormat): Promise<string> {
    switch (format) {
      case 'json':
        return this.exportJSON(model)
      case 'csv':
        return this.exportCSV(model)
      case 'markdown':
        return this.exportMarkdown(model)
      case 'svg':
        return await this.exportSVG(model)
      default:
        throw new Error(`Unsupported format: ${format}`)
    }
  }

  private exportJSON(model: ArchitectureModel): string {
    const data = {
      cards: Array.from(model.cards.values()),
      links: Array.from(model.links.values()),
      metadata: model.metadata,
    }
    return JSON.stringify(data, null, 2)
  }

  private exportCSV(model: ArchitectureModel): string {
    // Traceability matrix as CSV
    const rows: string[] = []
    rows.push('ID,Type,Name,Status,Owner')
    
    for (const card of model.cards.values()) {
      rows.push(`${card.id},${card.type},${card.name},${card.status},${card.owner}`)
    }
    
    return rows.join('\n')
  }

  private exportMarkdown(model: ArchitectureModel): string {
    const lines: string[] = ['# Architecture Model\n']
    
    for (const driver of model.drivers()) {
      lines.push(`## ${driver.name}\n`)
      lines.push(`${driver.description}\n`)
    }
    
    return lines.join('\n')
  }

  private async exportSVG(model: ArchitectureModel): Promise<string> {
    // Render entire model as SVG
    const diagram = renderingEngine.render(model)
    return diagram.toSVG()
  }
}
```

---

## 10. Implementation Notes

### Language: TypeScript

```typescript
// Use native Node.js fs module with promises
import { promises as fs } from 'fs'

// Use AJV for validation
import Ajv from 'ajv'
const ajv = new Ajv()

// Use simple-git for Git operations
import simpleGit from 'simple-git'
```

### File System Operations

- Always use atomic writes (write to temp, rename)
- Use `.gitignore` for cache directories
- Handle symlinks appropriately

---

## 11. Testing

### Unit Tests

- Load/save roundtrips
- Transaction isolation
- Validation before write
- Index correctness

### Integration Tests

- Full model load/save cycle
- Git integration
- Backup/restore
- Large file handling

---

## 12. Performance Targets

| Operation              | Time     | Notes                          |
| ---------------------- | -------- | ------------------------------ |
| Load 1000-card model   | < 1s     | Read, parse, index             |
| Save 1000-card model   | < 2s     | Validation, write, atomic rename |
| Save single card       | < 100ms  | Validation, atomic write       |
| Create backup          | < 5s     | Copy operation                 |
| Semantic diff          | < 100ms  | In-memory comparison           |
