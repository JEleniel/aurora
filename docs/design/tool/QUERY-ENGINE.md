# AURORA Query Engine Design

**Component**: Query Engine | **Status**: Design | **Complexity**: High

---

## Overview

The Query Engine is a subsystem that enables powerful, efficient querying of the AURORA model graph. It supports graph traversal, filtering, searching, and complex queries without materializing unnecessary data.

---

## Core Capabilities

### 1. Graph Traversal

#### Immediate Neighbors

```typescript
// Get all cards directly linked from source
outgoing(cardId: string): Card[]

// Get all cards that link to target
incoming(cardId: string): Card[]

// Get all directly connected cards (in + out)
neighbors(cardId: string): Card[]
```

#### Reachability

```typescript
// All cards reachable from source (forward)
reachable(cardId: string): Set<string>

// All cards from which source is reachable (reverse)
dependents(cardId: string): Set<string>

// Path exists between two cards?
isConnected(sourceId: string, targetId: string): boolean
```

#### Path Finding

```typescript
// Shortest path
shortestPath(sourceId: string, targetId: string): Card[] | null

// All paths (useful for traceability analysis)
allPaths(sourceId: string, targetId: string, maxDepth?: number): Card[][]

// Paths with metadata (including link details)
pathsWithMetadata(sourceId: string, targetId: string): PathInfo[]
```

### 2. Filtering

#### Type Filtering

```typescript
// Get all cards of specific types
ofType(...types: CardType[]): Card[]

// Get all requirements
requirements(): Card[]

// Get all drivers
drivers(): Card[]
```

#### Status Filtering

```typescript
// Get cards by status
byStatus(...statuses: CardStatus[]): Card[]

// Get approved and verified cards
approved(): Card[]
verified(): Card[]

// Get deprecated/retired cards
obsolete(): Card[]
```

#### Owner/Team Filtering

```typescript
byOwner(...owners: string[]): Card[]
```

#### Attribute Filtering

```typescript
// Filter by arbitrary attribute
where(predicate: (card: Card) => boolean): Card[]

// Example: all behaviors with latency > 100ms
where(c => c.type === 'behavior' && c.attributes.latency > 100)
```

#### Constraint Filtering

```typescript
// Cards affected by constraint
affectedByConstraint(constraintId: string): Card[]
```

### 3. Full-Text Search

```typescript
// Search in name + description
search(query: string): SearchResult[]

// Regex search on IDs
searchById(pattern: RegExp): Card[]

// Fuzzy search (for typo tolerance)
fuzzySearch(query: string, threshold?: number): Card[]
```

### 4. Composition & Chaining

Filters are composable:

```typescript
// Find all approved requirements that derive from driver X
requirements()
  .byStatus("approved")
  .reachableFrom("driver:root")
  .execute()

// Find all deprecated components with no tests
logicalComponents()
  .byStatus("deprecated")
  .filter(c => !hasIncomingLinks(c, "verified-by"))
  .execute()
```

---

## Internal Architecture

### Query Plan Optimization

For complex queries, optimize traversal order:

```typescript
Input: Find all requirements of approved drivers that have no test

Parse:
  - Type: requirement
  - Status: approved
  - Filter: incomingLinks from test count = 0
  - Reverse filter: outgoing to driver

Optimized Plan:
  1. Load all drivers (small set)
  2. Filter to approved
  3. Get all requirements reachable from those drivers
  4. Filter to those with incoming test links
  5. Exclude those from step 3
```

### Indexing Strategy

Maintain multiple indexes for O(1) lookups:

```typescript
class IndexManager {
  byId: Map<string, Card>           // id → card
  byType: Map<CardType, Card[]>     // type → cards
  byStatus: Map<CardStatus, Card[]> // status → cards
  byOwner: Map<string, Card[]>      // owner → cards
  
  // Graph indexes
  outgoingLinks: Map<string, Link[]>    // source → links
  incomingLinks: Map<string, Link[]>    // target → links
  
  // Search indexes
  textIndex: SearchIndex                // FTS
  idIndex: RegexIndex                   // regex
}
```

### Lazy Evaluation

Build query plans but don't evaluate until `.execute()`:

```typescript
const plan = requirements()
  .byStatus("approved")
  .reachableFrom("driver:root")
  // Nothing evaluated yet; plan created

const results = plan.execute()
// Now evaluation happens

const count = plan.count()
// Count without materializing all results
```

---

## Specialized Queries

### Traceability Queries

```typescript
// Complete traceability path from driver to test
tracePath(driverId: string, testId: string): {
  driver: Card
  requirements: Card[]
  behaviors: Card[]
  tests: Card[]
  links: Link[]
}

// All requirements not covered by tests
uncoveredRequirements(): Card[]

// Coverage report
coverageReport(): {
  total: number
  covered: number
  uncovered: number
  coveragePercent: number
}
```

### Dependency Queries

```typescript
// All dependencies of component (transitive)
dependencies(componentId: string): Card[]

// Circular dependencies (should be empty for DAG)
cirularDependencies(): Link[][]

// Dependency depth
dependencyDepth(cardId: string): number

// Cards with most dependents (critical)
criticalCards(): Card[]
```

### Impact Analysis

```typescript
// If I change this card, what else is affected?
impactAnalysis(cardId: string, depth?: number): Card[]

// For a set of changes, what's the total impact?
impactAnalysisForChanges(changes: Change[]): {
  directlyAffected: Card[]
  transitivelyAffected: Card[]
  riskLevel: "low" | "medium" | "high"
}
```

### Element View (Centered)

```typescript
// Get card with all neighbors and metadata
elementView(cardId: string, depth: number = 1): {
  center: Card
  outgoing: {
    card: Card
    link: Link
    distance: number
  }[]
  incoming: {
    card: Card
    link: Link
    distance: number
  }[]
}
```

---

## Query DSL (Optional)

For human-readable queries:

```typescript
# Get all unapproved requirements related to driver:security

find requirement
where status = "proposed" OR status = "approved"
related-to driver:security
order-by modified_date DESC

# Get coverage report for driver:performance

find requirement
where derives-from driver:performance
NOT verified-by test
```

---

## Performance Characteristics

| Operation | Time | Notes |
|-----------|------|-------|
| `byId(id)` | O(1) | Hash lookup |
| `byType(type)` | O(1) | Index lookup + copy |
| `byStatus(status)` | O(1) | Index lookup + copy |
| `outgoing(id)` | O(1) | Adjacency list |
| `incoming(id)` | O(1) | Reverse adjacency list |
| `reachable(id)` | O(V + E) | BFS/DFS |
| `shortestPath(s, t)` | O(V + E) | BFS |
| `allPaths(s, t)` | O(paths) | DFS |

For typical 5,000-card models:

- Most queries execute in < 10ms
- Complex path-finding < 100ms
- Index construction < 500ms

---

## Implementation Notes

### Language: TypeScript

```typescript
class QueryEngine {
  private model: ArchitectureModel
  private indexes: IndexManager

  // Filters return QueryBuilder for chaining
  ofType(...types: CardType[]): QueryBuilder
  byStatus(...statuses: CardStatus[]): QueryBuilder
  
  // Complex queries return results directly
  allPaths(from: string, to: string): Card[][]
  impactAnalysis(cardId: string): ImpactAnalysisResult
}

class QueryBuilder {
  private filters: Filter[]
  private engine: QueryEngine

  ofType(...types: CardType[]): QueryBuilder { ... }
  byStatus(...statuses: CardStatus[]): QueryBuilder { ... }
  where(predicate: (card: Card) => boolean): QueryBuilder { ... }
  
  execute(): Card[] { ... }
  count(): number { ... }
  first(): Card | null { ... }
}
```

### Caching Strategy

```typescript
class QueryCache {
  // Cache simple queries
  cacheQuery(query: string, result: Card[]): void
  
  // Invalidate on mutations
  invalidateOnCardChange(cardId: string): void
  invalidateOnLinkChange(linkId: string): void
  
  // Stats for optimization
  getHitRate(): number
  getTopQueries(): string[]
}
```

---

## Testing

### Unit Tests

- Graph traversal correctness
- Filter combination correctness
- Lazy evaluation (plan creation without execution)
- Index maintenance under mutations

### Integration Tests

- Complex multi-filter queries
- Path-finding in real models
- Impact analysis accuracy
- Performance benchmarks

---

## Future Enhancements

1. **Query Language**: Dedicated DSL for non-programmers
2. **Distributed Queries**: Support for very large models (100k+ cards)
3. **Real-Time Updates**: Subscriptions to query result changes
4. **Explain Plans**: Show how query is executed for optimization debugging
