# AURORA Data Storage & Export Format Specification

**Component**: Data Storage & Export | **Status**: Design | **Version**: 1.0.0

---

## Overview

AURORA models can be stored and exported in multiple formats to support various use cases:

1. **Primary Format**: JSON files in filesystem (Git-friendly, canonical)
2. **Archive Format**: `Arch.ZIP` (distributable, portable)
3. **Output Formats**: SVG diagrams, HTML reports, CSV matrices, Markdown documentation

This document specifies the structure and requirements for each format.

---

## 1. Primary Format: Filesystem Organization

### 1.1 Directory Structure

```text
project-root/
├── docs/
│   ├── cards/
│   │   ├── driver/
│   │   │   ├── driver-root.json
│   │   │   ├── driver-performance.json
│   │   │   ├── driver-security.json
│   │   │   └── ... (all driver cards)
│   │   │
│   │   ├── requirement/
│   │   │   ├── requirement-api-latency.json
│   │   │   ├── requirement-api-security.json
│   │   │   └── ... (all requirement cards)
│   │   │
│   │   ├── behavior/
│   │   │   ├── behavior-authentication.json
│   │   │   └── ... (all behavior cards)
│   │   │
│   │   ├── interface/
│   │   │   ├── interface-public-api.json
│   │   │   └── ... (all interface cards)
│   │   │
│   │   ├── constraint/
│   │   │   ├── constraint-latency-100ms.json
│   │   │   └── ... (all constraint cards)
│   │   │
│   │   ├── actor/
│   │   │   ├── actor-end-user.json
│   │   │   └── ... (all actor cards)
│   │   │
│   │   ├── logical-component/
│   │   │   ├── logical-component-api-gateway.json
│   │   │   └── ... (all logical component cards)
│   │   │
│   │   ├── deployable-node/
│   │   │   ├── deployable-node-aws-region-us-east.json
│   │   │   └── ... (all deployment nodes)
│   │   │
│   │   ├── test/
│   │   │   ├── test-api-latency-check.json
│   │   │   └── ... (all test cards)
│   │   │
│   │   ├── artifact/
│   │   │   ├── artifact-openapi-spec.json
│   │   │   └── ... (all artifact cards)
│   │   │
│   │   ├── view/
│   │   │   ├── view-traceability-matrix.json
│   │   │   └── ... (all view cards)
│   │   │
│   │   └── note/
│   │       ├── note-architecture-decision-1.json
│   │       └── ... (all note cards)
│   │
│   └── links/
│       └── links.json                    # All links in single atomic file
│
├── schemas/
│   ├── card.schema.json
│   ├── link.schema.json
│   ├── driver.schema.json
│   └── ... (type-specific schemas)
│
├── .aurora/
│   ├── preferences.json                  # Project configuration
│   ├── metadata.json                     # Project metadata
│   └── cache/                            # Generated, not in version control
│       ├── diagrams/
│       │   ├── view-id.mermaid
│       │   ├── view-id.svg
│       │   └── view-id.html
│       └── indexes/
│           └── indexes.json
│
└── .git/                                 # Version control (optional)
```

### 1.2 File Naming Convention

**Cards**:

```
{type}-{name}.json

Examples:
- driver-root.json
- requirement-api-latency.json
- behavior-user-authentication.json
- interface-public-api.json
- constraint-latency-100ms.json
- actor-end-user.json
- logical-component-api-gateway.json
- deployable-node-aws-region-us-east.json
- test-api-latency-check.json
- artifact-openapi-spec.json
- view-traceability-matrix.json
- note-architecture-decision-001.json
```

Rules:

- **Prefix**: Card type (lowercase, singular)
- **Name**: Card name (lowercase, hyphens only, no spaces)
- **Extension**: `.json`
- **Pattern**: `^[a-z]+-[a-z0-9-]+\.json$`

**Links**:

```
links.json

Single file containing all links (atomic updates)
```

### 1.3 Folder Organization Rationale

**Benefits of type-based folders**:

- ✓ Easy to navigate (all drivers in one place)
- ✓ Grouped by semantic type
- ✓ IDE tree view clarity
- ✓ Easier version control (smaller files, granular diffs)
- ✓ Supports parallel workflows (team A on requirements, team B on tests)
- ✓ Scalability (12 folders × ~500 cards each = manageable)

**Alternative considered**: Single `cards/` folder with all cards

- ✗ Harder to navigate (12+ types mixed)
- ✗ Larger diffs for any change
- ✓ Slightly simpler to implement
- ✓ Useful for very small models (< 100 cards)

**Conclusion**: Type-based folders recommended for production.

---

## 2. Archive Format: Arch.ZIP

### 2.1 ZIP Structure

The `Arch.ZIP` file is a standard ZIP archive containing the canonical model in a portable, distributable format.

```
Arch.ZIP
├── cards/
│   ├── driver/
│   │   ├── driver-root.json
│   │   ├── driver-performance.json
│   │   └── ... (all drivers)
│   ├── requirement/
│   │   ├── requirement-api-latency.json
│   │   └── ... (all requirements)
│   ├── behavior/
│   ├── interface/
│   ├── constraint/
│   ├── actor/
│   ├── logical-component/
│   ├── deployable-node/
│   ├── test/
│   ├── artifact/
│   ├── view/
│   └── note/
│
├── links/
│   └── links.json
│
├── schemas/
│   ├── card.schema.json
│   ├── link.schema.json
│   ├── driver.schema.json
│   └── ... (all schemas)
│
├── metadata.json                        # Project metadata
├── preferences.json                     # Project preferences
├── README.md                            # Brief overview
└── MANIFEST.json                        # Archive manifest
```

### 2.2 Manifest File (MANIFEST.json)

```json
{
  "version": "1.0.0",
  "format": "aurora-archive-v1",
  "created": "2025-12-14T10:30:00Z",
  "created_by": "AURORA Tooling v1.0.0",
  "model": {
    "name": "Project Name",
    "description": "Brief description",
    "root_driver_id": "driver:root",
    "statistics": {
      "total_cards": 127,
      "cards_by_type": {
        "driver": 10,
        "requirement": 42,
        "behavior": 8,
        "interface": 4,
        "constraint": 12,
        "actor": 3,
        "logical-component": 15,
        "deployable-node": 8,
        "test": 20,
        "artifact": 4,
        "view": 1,
        "note": 0
      },
      "total_links": 156,
      "coverage": {
        "requirements_with_tests": "95%",
        "requirements_with_acceptance_criteria": "100%"
      }
    }
  },
  "contents": [
    {
      "type": "cards",
      "path": "cards/",
      "count": 127,
      "format": "json"
    },
    {
      "type": "links",
      "path": "links/links.json",
      "count": 156,
      "format": "json"
    },
    {
      "type": "schemas",
      "path": "schemas/",
      "format": "json-schema"
    }
  ],
  "checksums": {
    "cards": "sha256:abc123...",
    "links": "sha256:def456...",
    "total": "sha256:xyz789..."
  }
}
```

### 2.3 README.md in Archive

```markdown
# AURORA Architecture Model

**Project**: [Project Name]
**Version**: [Version]
**Created**: [Date]
**Root Driver**: driver:root

## Quick Start

1. Extract this archive to a directory
2. Load the model using AURORA tooling
3. View rendered diagrams in the `diagrams/` folder
4. Edit cards in `cards/` or use the AURORA editor

## Contents

- `cards/` — All architecture cards (drivers, requirements, etc.)
- `links/links.json` — Relationships between cards
- `schemas/` — JSON Schema definitions
- `metadata.json` — Project information
- `preferences.json` — Tooling preferences

## Statistics

- Total Cards: 127
- Total Links: 156
- Requirements: 42
- Tests: 20
- Coverage: 95% of requirements have tests

## File Organization

Cards are organized by type:
- `cards/driver/` — Driver cards
- `cards/requirement/` — Requirement cards
- `cards/behavior/` — Behavior cards
- ... (10 other types)

## Validation

All files conform to AURORA JSON Schemas in `schemas/`.

To validate:
```bash
npx ajv validate -s schemas/card.schema.json -d cards/**/*.json
npx ajv validate -s schemas/link.schema.json -d links/links.json
```

## Loading in AURORA

```typescript
import { ModelLoader } from '@aurora/core'

const loader = new ModelLoader()
const model = await loader.loadArchive('Arch.ZIP')

console.log(`Loaded ${model.cards.size} cards and ${model.links.size} links`)
```

---

**AURORA Format Version**: 1.0.0

```

### 2.4 ZIP Creation Process

```typescript
class ArchiveExporter {
  async createArchive(
    model: ArchitectureModel,
    outputPath: string = 'Arch.ZIP'
  ): Promise<void> {
    const zip = new AdmZip()

    // 1. Add all cards organized by type
    for (const [cardId, card] of model.cards) {
      const cardType = card.type
      const folder = `cards/${cardType}/`
      const filename = `${card.id}.json`
      const content = JSON.stringify(card, null, 2)

      zip.addFile(folder + filename, Buffer.from(content, 'utf-8'))
    }

    // 2. Add all links
    const linksArray = Array.from(model.links.values())
    const linksContent = JSON.stringify(linksArray, null, 2)
    zip.addFile('links/links.json', Buffer.from(linksContent, 'utf-8'))

    // 3. Add schemas
    for (const [schemaName, schema] of model.schemas) {
      const schemaContent = JSON.stringify(schema, null, 2)
      zip.addFile(`schemas/${schemaName}`, Buffer.from(schemaContent, 'utf-8'))
    }

    // 4. Create manifest
    const manifest = this.generateManifest(model)
    zip.addFile(
      'MANIFEST.json',
      Buffer.from(JSON.stringify(manifest, null, 2), 'utf-8')
    )

    // 5. Create README
    const readme = this.generateREADME(model)
    zip.addFile('README.md', Buffer.from(readme, 'utf-8'))

    // 6. Add project metadata
    zip.addFile(
      'metadata.json',
      Buffer.from(JSON.stringify(model.metadata, null, 2), 'utf-8')
    )

    // 7. Add preferences
    zip.addFile(
      'preferences.json',
      Buffer.from(JSON.stringify(model.preferences, null, 2), 'utf-8')
    )

    // 8. Write to disk
    zip.writeZip(outputPath)
  }

  private generateManifest(model: ArchitectureModel): ManifestData {
    return {
      version: '1.0.0',
      format: 'aurora-archive-v1',
      created: new Date().toISOString(),
      created_by: `AURORA Tooling v${AURORA_VERSION}`,
      model: {
        name: model.metadata.name,
        description: model.metadata.description,
        root_driver_id: model.rootDriverId,
        statistics: {
          total_cards: model.cards.size,
          cards_by_type: this.countCardsByType(model),
          total_links: model.links.size,
          coverage: this.calculateCoverage(model),
        },
      },
      contents: [
        {
          type: 'cards',
          path: 'cards/',
          count: model.cards.size,
          format: 'json',
        },
        {
          type: 'links',
          path: 'links/links.json',
          count: model.links.size,
          format: 'json',
        },
        {
          type: 'schemas',
          path: 'schemas/',
          format: 'json-schema',
        },
      ],
      checksums: {
        cards: await this.hashCards(model),
        links: await this.hashLinks(model),
        total: await this.hashModel(model),
      },
    }
  }

  private countCardsByType(model: ArchitectureModel): Record<string, number> {
    const counts: Record<string, number> = {}
    for (const card of model.cards.values()) {
      counts[card.type] = (counts[card.type] || 0) + 1
    }
    return counts
  }

  private calculateCoverage(model: ArchitectureModel): Record<string, string> {
    const requirements = Array.from(model.cards.values()).filter(
      (c) => c.type === 'requirement'
    )
    const withTests = requirements.filter((req) =>
      this.hasIncomingLinkOfType(req.id, model, 'verified-by')
    )
    const withCriteria = requirements.filter(
      (req) => req.attributes.acceptance_criteria
    )

    return {
      requirements_with_tests:
        `${Math.round((withTests.length / requirements.length) * 100)}%`,
      requirements_with_acceptance_criteria:
        `${Math.round((withCriteria.length / requirements.length) * 100)}%`,
    }
  }
}
```

### 2.5 ZIP Extraction Process

```typescript
class ArchiveLoader {
  async loadArchive(zipPath: string): Promise<ArchitectureModel> {
    const zip = new AdmZip(zipPath)
    const entries = zip.getEntries()

    // 1. Load all cards
    const cards = new Map<string, Card>()
    for (const entry of entries) {
      if (entry.entryName.startsWith('cards/') && entry.entryName.endsWith('.json')) {
        const content = entry.getData().toString('utf-8')
        const card = JSON.parse(content) as Card
        cards.set(card.id, card)
      }
    }

    // 2. Load links
    const linksEntry = entries.find((e) => e.entryName === 'links/links.json')
    const linkArray = JSON.parse(linksEntry.getData().toString('utf-8')) as Link[]
    const links = new Map(linkArray.map((l) => [l.id, l]))

    // 3. Load metadata
    const metadataEntry = entries.find((e) => e.entryName === 'metadata.json')
    const metadata = JSON.parse(metadataEntry.getData().toString('utf-8'))

    // 4. Load preferences
    const prefsEntry = entries.find((e) => e.entryName === 'preferences.json')
    const preferences = JSON.parse(prefsEntry.getData().toString('utf-8'))

    // 5. Create model
    const model = new ArchitectureModel(cards, links, metadata)
    model.preferences = preferences

    return model
  }
}
```

### 2.6 Advantages of Arch.ZIP Format

| Aspect | Benefit |
| --- | --- |
| **Portability** | Single file, easy to email/download |
| **Compression** | JSON compresses well (50-70% reduction) |
| **Versioning** | Archive timestamp = version snapshot |
| **Offline** | Works without internet/Git |
| **Distribution** | Clear deliverable format |
| **Validation** | Manifest enables integrity checking |
| **Organization** | Type-based folders for clarity |

---

## 3. Output Formats

### 3.1 SVG Format (Recommended)

**Primary vector diagram format for architectural views.**

#### 3.1.1 SVG Structure

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 800">
  <!-- Metadata -->
  <defs>
    <style type="text/css">
      .driver { fill: #f9f; stroke: #f00; stroke-width: 2; }
      .requirement { fill: #bbf; stroke: #00f; stroke-width: 1.5; }
      .behavior { fill: #bfb; stroke: #0f0; stroke-width: 1; }
      .test { fill: #ffb; stroke: #ff0; stroke-width: 1; }
      .link { stroke: #666; fill: none; marker-end: url(#arrowhead); }
      .link-strong { stroke: #000; stroke-width: 2; }
      .link-weak { stroke: #ccc; stroke-width: 0.5; stroke-dasharray: 3,3; }
    </style>
    <marker
      id="arrowhead"
      markerWidth="10"
      markerHeight="10"
      refX="9"
      refY="3"
      orient="auto"
    >
      <polygon points="0 0, 10 3, 0 6" fill="#666" />
    </marker>
  </defs>

  <!-- Nodes (cards) -->
  <g id="nodes">
    <g id="driver-root" class="node driver">
      <rect x="500" y="20" width="200" height="60" rx="4" />
      <text x="600" y="55" text-anchor="middle" font-weight="bold">
        driver:root
      </text>
    </g>

    <g id="requirement-api-latency" class="node requirement">
      <rect x="100" y="150" width="180" height="50" rx="2" />
      <text x="190" y="180" text-anchor="middle">
        requirement:api-latency
      </text>
    </g>

    <!-- More nodes... -->
  </g>

  <!-- Links (edges) -->
  <g id="links">
    <line
      class="link link-normal"
      x1="190"
      y1="150"
      x2="600"
      y2="80"
      marker-end="url(#arrowhead)"
    />
    <!-- More links... -->
  </g>

  <!-- Interactive elements -->
  <g id="info-panel" style="display: none;">
    <rect width="250" height="200" fill="white" stroke="black" />
    <text id="info-title" x="10" y="20" font-weight="bold" />
    <text id="info-description" x="10" y="50" />
  </g>
</svg>
```

#### 3.1.2 SVG Generation

```typescript
class SVGRenderer implements Renderer {
  render(diagram: Diagram, layout: LayoutResult, options?: RenderOptions): string {
    const svg = new SVGDocument(layout.bounds.width, layout.bounds.height)

    // 1. Add styles
    svg.addStyles(this.generateStyles(options?.theme))

    // 2. Add markers (arrowheads)
    svg.addMarkers()

    // 3. Render nodes
    const nodesGroup = svg.addGroup('nodes')
    for (const node of layout.nodes) {
      const card = diagram.cardMap.get(node.id)
      this.renderNode(nodesGroup, node, card, options)
    }

    // 4. Render links
    const linksGroup = svg.addGroup('links')
    for (const edge of layout.edges) {
      const link = diagram.linkMap.get(edge.id)
      this.renderEdge(linksGroup, edge, link, options)
    }

    // 5. Add metadata
    svg.addComment(`Generated by AURORA ${AURORA_VERSION}`)
    svg.addComment(`Created: ${new Date().toISOString()}`)
    svg.addComment(`View: ${diagram.viewId}`)

    return svg.toString()
  }

  private renderNode(
    group: SVGGroup,
    node: Node,
    card: Card,
    options?: RenderOptions
  ): void {
    const nodeGroup = group.addGroup(card.id)

    // Background rectangle
    const style = this.getNodeStyle(card, options?.theme)
    nodeGroup.addRect({
      x: node.x - node.width / 2,
      y: node.y - node.height / 2,
      width: node.width,
      height: node.height,
      rx: 4,
      class: card.type,
      style: style,
    })

    // Label text
    nodeGroup.addText({
      x: node.x,
      y: node.y,
      text: `${card.id}`,
      textAnchor: 'middle',
      dominantBaseline: 'middle',
      fontSize: '12px',
      fontWeight: 'bold',
    })

    // Status indicator (small circle)
    const statusColor = this.getStatusColor(card.status)
    nodeGroup.addCircle({
      cx: node.x + node.width / 2 - 8,
      cy: node.y - node.height / 2 + 8,
      r: 4,
      fill: statusColor,
    })
  }

  private renderEdge(
    group: SVGGroup,
    edge: Edge,
    link: Link,
    options?: RenderOptions
  ): void {
    const edgeGroup = group.addGroup(link.id)

    // Path along waypoints
    const pathData = this.computePathData(edge.waypoints)
    const style = this.getLinkStyle(link, options?.theme)

    edgeGroup.addPath({
      d: pathData,
      class: `link link-${link.strength}`,
      style: style,
      markerEnd: 'url(#arrowhead)',
    })

    // Label at midpoint
    const midpoint = this.computeMidpoint(edge.waypoints)
    edgeGroup.addText({
      x: midpoint.x,
      y: midpoint.y - 5,
      text: link.rationale || '',
      fontSize: '10px',
      fill: '#666',
      textAnchor: 'middle',
    })
  }
}
```

#### 3.1.3 SVG Features

| Feature | Implementation |
| --- | --- |
| **Interactivity** | Click nodes to navigate, hover for details |
| **Styling** | CSS classes for types and statuses |
| **Scalability** | Viewbox allows responsive sizing |
| **Metadata** | Embedded as XML comments |
| **Export** | Can be converted to PNG via headless browser |

#### 3.1.4 SVG Export Example

```bash
# Export view as SVG
aurora export --view=view-traceability-matrix --format=svg --output=diagram.svg

# Export all views as SVG
aurora export --all --format=svg --output-dir=./diagrams/

# Convert SVG to PNG (requires ImageMagick or similar)
convert -density 150 diagram.svg diagram.png
```

### 3.2 Mermaid Format

**GitHub-compatible markdown diagrams.**

```markdown
# Architecture Overview

## Hierarchy View

\`\`\`mermaid
graph TD
    DR["🎯 driver:root"]
    RQ1["requirement:api-latency"]
    RQ2["requirement:api-security"]
    BH1["behavior:fast-response"]
    TS1["test:api-latency-check"]

    DR --> RQ1
    DR --> RQ2
    RQ1 --> BH1
    BH1 --> TS1

    style DR fill:#f9f,stroke:#f00
    style RQ1 fill:#bbf,stroke:#00f
    style BH1 fill:#bfb,stroke:#0f0
    style TS1 fill:#ffb,stroke:#ff0
\`\`\`

## Traceability Matrix

| Requirement | Driver | Test | Status |
| --- | --- | --- | --- |
| requirement:api-latency | driver:root | test:api-latency-check | ✓ Verified |
| requirement:api-security | driver:root | test:auth-token | ✓ Verified |
```

### 3.3 HTML Report Format

**Interactive web-based views.**

```html
<!DOCTYPE html>
<html>
<head>
    <title>AURORA Architecture Report</title>
    <script src="https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js"></script>
    <link rel="stylesheet" href="report.css">
</head>
<body>
    <header>
        <h1>Architecture Report</h1>
        <p>Generated: <span id="timestamp"></span></p>
    </header>

    <nav>
        <ul>
            <li><a href="#hierarchy">Hierarchy</a></li>
            <li><a href="#traceability">Traceability</a></li>
            <li><a href="#dependencies">Dependencies</a></li>
            <li><a href="#coverage">Coverage</a></li>
        </ul>
    </nav>

    <main>
        <section id="hierarchy">
            <h2>Hierarchy View</h2>
            <div class="mermaid">
                graph TD
                ...
            </div>
        </section>

        <section id="traceability">
            <h2>Traceability Matrix</h2>
            <table class="matrix">
                <thead>
                    <tr>
                        <th>Requirement</th>
                        <th>Driver</th>
                        <th>Test</th>
                        <th>Status</th>
                    </tr>
                </thead>
                <tbody>
                    ...
                </tbody>
            </table>
        </section>

        <section id="coverage">
            <h2>Coverage Report</h2>
            <div class="chart">
                <canvas id="coverage-chart"></canvas>
            </div>
        </section>
    </main>

    <script src="report.js"></script>
</body>
</html>
```

### 3.4 CSV Format

**Spreadsheet-friendly export for analysis.**

```csv
ID,Type,Name,Status,Owner,Description,Created,Modified,Incoming Links,Outgoing Links
driver:root,driver,Root Driver,approved,architecture-team,Project mission and goals,2025-01-01,2025-12-14,0,10
requirement:api-latency,requirement,API Latency,approved,api-team,Response time must be < 100ms,2025-01-05,2025-12-10,1,3
requirement:api-security,requirement,API Security,approved,security-team,Must encrypt all data in transit,2025-01-10,2025-12-12,1,2
behavior:authentication,behavior,User Authentication,implemented,auth-team,OAuth2 flow for user login,2025-02-01,2025-12-14,2,1
...
```

### 3.5 Markdown Format

**Documentation-friendly human-readable format.**

```markdown
# AURORA Architecture Documentation

## Overview

**Project**: [Project Name]
**Version**: [Version]
**Date**: [Date]

---

## Drivers

### driver:root

**Name**: Root Driver

**Description**: Project mission and overarching goals

**Status**: approved

**Owner**: architecture-team

**Related Requirements**:
- requirement:api-latency
- requirement:api-security
- requirement:api-versioning

---

## Requirements

### requirement:api-latency

**Name**: API Latency

**Description**: Response time must be < 100ms

**Status**: approved

**Owner**: api-team

**Acceptance Criteria**:
- 95th percentile latency < 100ms
- 99th percentile latency < 200ms
- No timeout errors

**Derives From**: driver:root

**Verified By**:
- test:api-latency-check
- test:load-test-sustained

---

## Tests

### test:api-latency-check

**Name**: API Latency Check

**Type**: performance

**Framework**: Apache JMeter

**Verifies**: requirement:api-latency

**Status**: verified

**Coverage**: 100%

---

## Coverage Report

| Type | Count | Covered | Coverage |
| --- | --- | --- | --- |
| Drivers | 10 | 10 | 100% |
| Requirements | 42 | 40 | 95% |
| Behaviors | 8 | 8 | 100% |
| Tests | 20 | 20 | 100% |

---

Generated by AURORA Tooling v1.0.0
```

---

## 4. Export Operations

### 4.1 Export Command Interface

```bash
# Export archive
aurora export --format=archive --output=Arch.ZIP

# Export as SVG
aurora export --format=svg --output=diagram.svg

# Export all views
aurora export --all --format=svg --output-dir=./diagrams/

# Export specific view
aurora export --view=view-traceability-matrix --format=html --output=report.html

# Export with filters
aurora export --filter=status:approved --format=csv --output=approved.csv

# Export with options
aurora export \
  --format=svg \
  --theme=dark \
  --dpi=300 \
  --output=high-res-diagram.svg
```

### 4.2 Export Options

```typescript
interface ExportOptions {
  format: 'archive' | 'svg' | 'mermaid' | 'html' | 'csv' | 'markdown'
  output?: string                    // Output file path
  outputDir?: string                 // For multiple files
  
  // Format-specific
  theme?: 'light' | 'dark' | 'print'
  dpi?: number                       // For SVG → PNG
  quality?: 'draft' | 'normal' | 'high'
  
  // Filtering
  filter?: string                    // status:approved, type:requirement, etc.
  viewId?: string                    // Export specific view
  all?: boolean                      // Export all views
  
  // Rendering
  includeMetadata?: boolean          // Include provenance, audit history
  includeLinks?: boolean             // Include relationship info
  includeCoverage?: boolean          // Include coverage metrics
}
```

### 4.3 Batch Export

```typescript
class BatchExporter {
  async exportAll(model: ArchitectureModel, outputDir: string): Promise<void> {
    // Archive
    await this.exportArchive(model, `${outputDir}/Arch.ZIP`)

    // All views as SVG
    for (const view of model.views()) {
      await this.exportView(model, view, 'svg', `${outputDir}/view-${view.id}.svg`)
    }

    // Coverage report as HTML
    await this.exportCoverage(model, `${outputDir}/coverage-report.html`)

    // Traceability matrix as CSV
    await this.exportMatrix(model, `${outputDir}/traceability-matrix.csv`)

    // Full documentation as Markdown
    await this.exportDocumentation(model, `${outputDir}/ARCHITECTURE.md`)
  }
}
```

---

## 5. Implementation Checklist

### MVP (Minimum Viable Product)

- [x] Filesystem storage with type-based folders
- [x] Single `links.json` file (atomic)
- [x] `Arch.ZIP` archive creation and extraction
- [x] Manifest and README in archive
- [x] SVG diagram rendering
- [x] CSV export for matrices

### Phase 2

- [ ] Mermaid diagram export
- [ ] HTML report generation
- [ ] Markdown documentation export
- [ ] Batch export operations
- [ ] Export compression options

### Phase 3

- [ ] Database backend support (PostgreSQL)
- [ ] S3/Cloud storage integration
- [ ] Real-time export streaming
- [ ] Custom export templates
- [ ] Diff and comparison exports

---

## 6. Validation & Integrity

### 6.1 Archive Validation

```typescript
class ArchiveValidator {
  async validate(zipPath: string): Promise<ValidationResult> {
    const zip = new AdmZip(zipPath)
    
    // 1. Check manifest
    const manifest = this.loadManifest(zip)
    
    // 2. Validate all cards
    const cardsValid = await this.validateCards(zip, manifest)
    
    // 3. Validate links
    const linksValid = await this.validateLinks(zip, manifest)
    
    // 4. Verify checksums
    const checksumsValid = await this.verifyChecksums(zip, manifest)
    
    // 5. Check reference integrity
    const refsValid = this.validateReferences(zip)
    
    return {
      valid: cardsValid && linksValid && checksumsValid && refsValid,
      errors: [],
      warnings: [],
    }
  }
}
```

### 6.2 Checksum Strategy

```typescript
// Manifest includes SHA-256 checksums for:
// - All cards combined
// - All links
// - Total archive

// Verification:
const actualChecksum = await sha256(zip.getBuffer())
const expectedChecksum = manifest.checksums.total

if (actualChecksum !== expectedChecksum) {
  throw new IntegrityError('Archive checksum mismatch')
}
```

---

## 7. Performance Considerations

| Operation | Time | Size |
| --- | --- | --- |
| Create archive (1000 cards) | < 2s | ~2-3 MB |
| Extract archive | < 1s | - |
| SVG render (large diagram) | < 500ms | ~500 KB |
| CSV export (1000 cards) | < 100ms | ~1 MB |
| HTML report generation | < 2s | ~5 MB |

---

## 8. Examples

### 8.1 Export Workflow

```typescript
// Load model
const model = await loader.loadModel('./project')

// Export archive
const archExporter = new ArchiveExporter()
await archExporter.createArchive(model, 'Arch.ZIP')

// Export as SVG
const svgRenderer = new SVGRenderer()
for (const view of model.views()) {
  const svg = await svgRenderer.render(view)
  await fs.writeFile(`diagrams/${view.id}.svg`, svg)
}

// Export as CSV
const csvExporter = new CSVExporter()
const csv = csvExporter.exportMatrix(model)
await fs.writeFile('traceability-matrix.csv', csv)
```

### 8.2 Archive Extraction and Usage

```typescript
// Load from archive
const loader = new ArchiveLoader()
const model = await loader.loadArchive('Arch.ZIP')

// Validate
const validator = new ArchiveValidator()
const result = await validator.validate('Arch.ZIP')

// Query
const requirements = model.cards.values()
  .filter(c => c.type === 'requirement')

// Render
const renderer = new SVGRenderer()
for (const req of requirements) {
  const view = new ElementView(req.id, { depth: 2 })
  const svg = await renderer.render(view)
}
```

---

## 9. Conclusion

AURORA supports multiple storage and export formats:

1. **Primary**: Filesystem with type-based folders (canonical, Git-friendly)
2. **Archive**: `Arch.ZIP` (portable, distributable)
3. **Output**: SVG (recommended), Mermaid, HTML, CSV, Markdown

The `Arch.ZIP` format provides a self-contained, validated, portable representation of the entire architecture model with metadata, schemas, and manifest for integrity and discoverability.

---

**Format Version**: 1.0.0 | **Status**: Specification Complete
