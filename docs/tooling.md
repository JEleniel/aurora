# AURORA Tooling Guide

This guide documents the requirements, capabilities, and best practices for tooling that operates on AURORA artifacts.

---

## Overview

AURORA tooling has three primary responsibilities:

1. **Validation**: Ensure cards and links conform to schemas and conventions.
2. **Queries and Analysis**: Extract insights from the model (traceability, dependencies, coverage).
3. **Rendering**: Generate diagrams, reports, and other consumable views from the canonical model.

Tooling may be a single integrated tool or a collection of specialized tools that work together via standard AURORA JSON formats.

---

## Core Responsibilities

### Primary Responsibility: Canonical Source

- **Source of truth:** Cards and links stored as JSON are canonical; diagrams and reports are derived views.
- **No dual editing:** Avoid situations where diagrams and cards can be edited separately; changes must flow through the JSON model.
- **Provenance preservation:** When transforming or exporting artifacts, tooling must preserve `provenance` and `audit_history` fields.

### Validation

Tooling must validate:

1. **Schema Conformance** (against `schemas/*.json`):
   + Required fields: `id`, `type`, `name` (for cards); `id`, `type`, `source`, `target` (for links).
   + Allowed enum values: `status`, `type`, link `type`.
   + Version format: Semver compliance.
   + Audit history: Each entry must have `event_time` in ISO 8601 format.

2. **Naming Conventions** (per `docs/conventions.md`):
   + Card IDs: `namespace:element-name` format with lowercase and hyphens only.
   + Link IDs: Consistently formatted (e.g., `link:source-target-type`).
   + No reserved words or collisions.

3. **Reference Integrity**:
   + All card IDs referenced in `relations`, `links`, or `constraints` must exist.
   + All link source and target IDs must point to existing cards.
   + No dangling references.

4. **Traceability Completeness** (optional, configurable):
   + All requirements should derive from drivers.
   + All tests should verify requirements.
   + Major cards should have audit history.
   + (These may be warnings, not errors.)

### Diagram Generation

Tooling may translate card graphs into visual representations:

- **Mermaid diagrams** (GitHub Flavored Markdown friendly).
- **SVG/PNG exports** (for presentations, documentation).
- **Interactive visualizations** (web-based, tooling-specific).

## Generating Requirements Traceability Matrix

From cards with links of type `derives-from`, generate a matrix:

```text
Requirement        | Driver              | Status     | Owner
requirement:api-performance | driver:readability | approved | api-team
requirement:latency         | driver:performance  | verified | platform-team
```

## Generating State Machine Diagram

From a `behavior` card with a `state_machine` attribute, generate a UML-style state diagram.

## Generating Sequence Diagram

From a `behavior` card with a `sequence` attribute, generate a sequence diagram (PlantUML, Mermaid, etc.)..

### Reporting

Generate reports from the model:

- **Coverage Reports**: How many requirements have tests? How many have acceptance criteria?
- **Traceability Reports**: End-to-end coverage from drivers through requirements to tests.
- **Status Reports**: Count of cards by status (proposed, approved, implemented, verified).
- **Risk Reports**: Cards with missing audit history, weak links, or unverified requirements.

---

## Tooling Features

### Query and Navigation

Tooling should support:

1. **Graph Traversal**:
   + Find all cards that depend on a given card.
   + Find all cards that a given card depends on.
   + Find paths between two cards (what's the relationship?).

2. **Filtering**:
   + By type: Show all requirements.
   + By status: Show all deprecated cards.
   + By owner: Show cards owned by a team.
   + By constraint: Show all cards affected by a constraint.

3. **Searching**:
   + Full-text search on names and descriptions.
   + Regex search on IDs.
   + Query by attributes (e.g., find all behaviors with latency > 100ms).

4. **Centered Element View** (as per your requirement):
   + Select a card and display it centered on screen.
   + Show all directly related cards (via links/relations) around it.
   + Show link types and metadata (rationale, strength).
   + Allow "zoom in/out" to see cards one hop further away.

### Version Control Integration

Tooling should:

- Recognize AURORA models in Git and provide semantic diffs (not just line-by-line).
- Understand that card versions and audit history track evolution.
- Enable PR reviews with impact analysis: "This change affects these downstream requirements and behaviors."

### Collaboration Features

- **Markdown rendering**: Display cards in human-readable form (as currently done in `docs/cards/`).
- **Change proposals**: Agents or humans can propose changes to cards; tooling can render and validate them.
- **Approval workflows**: Track who approved what and when (via audit_history).

---

## Configuration

### `.aurora/preferences.json`

Project-level preferences:

```json
{
  "serialization": "json",
  "format_version": "1.0",
  "pre_release": false,
  "namespace_conventions": "kebab-case",
  "link_id_format": "link:source-type-linktype-target-name",
  "include_paths": ["docs/cards/**/*.json", "schemas/**/*.json"],
  "exclude_paths": ["docs/**/*.md", "*.example.json"],
  "tooling_options": {
    "diagram_format": "mermaid",
    "validation_strict": true,
    "audit_history_required": true
  }
}
```

---

## Validation Implementation Example

### Minimal Validator (Python pseudocode)

```python
import json
import jsonschema
from pathlib import Path

def validate_aurora_model(model_dir: str) -> dict:
    """Validate all cards and links in the model."""
    
    results = {
        "valid": True,
        "errors": [],
        "warnings": []
    }
    
    # Load schemas
    with open(f"{model_dir}/schemas/card.schema.json") as f:
        card_schema = json.load(f)
    with open(f"{model_dir}/schemas/link.schema.json") as f:
        link_schema = json.load(f)
    
    cards = {}
    links = {}
    
    # Validate cards
    for card_file in Path(f"{model_dir}/docs/cards").glob("*.json"):
        with open(card_file) as f:
            card = json.load(f)
        
        # Schema validation
        try:
            jsonschema.validate(card, card_schema)
            cards[card["id"]] = card
        except jsonschema.ValidationError as e:
            results["valid"] = False
            results["errors"].append(f"{card_file}: {e.message}")
        
        # Naming convention check
        if not is_valid_card_id(card["id"]):
            results["warnings"].append(f"{card_file}: ID '{card['id']}' violates naming conventions")
        
        # Audit history check
        if not card.get("audit_history"):
            results["warnings"].append(f"{card['id']}: No audit history")
    
    # Validate links
    for link_file in Path(f"{model_dir}/docs/links").glob("*.json"):
        with open(link_file) as f:
            link = json.load(f)
        
        try:
            jsonschema.validate(link, link_schema)
            links[link["id"]] = link
        except jsonschema.ValidationError as e:
            results["valid"] = False
            results["errors"].append(f"{link_file}: {e.message}")
        
        # Reference integrity
        if link["source"] not in cards:
            results["errors"].append(f"{link['id']}: Source '{link['source']}' not found")
        if link["target"] not in cards:
            results["errors"].append(f"{link['id']}: Target '{link['target']}' not found")
    
    # Check reference integrity in relations
    for card_id, card in cards.items():
        for rel_id in card.get("relations", []):
            if rel_id not in cards and rel_id not in links:
                results["warnings"].append(f"{card_id}: References unknown card/link '{rel_id}'")
    
    return results

def is_valid_card_id(card_id: str) -> bool:
    """Check if card ID follows naming conventions."""
    # namespace:element-name, lowercase, hyphens only
    import re
    return bool(re.match(r'^[a-z]+(?:\.[a-z]+)?:[a-z0-9\-]+$', card_id))
```

---

## Rendering Examples

### Mermaid Diagram Generation

**From a driver hierarchy card, generate a flowchart:**

```python
def generate_driver_diagram(cards: dict) -> str:
    """Generate Mermaid flowchart from driver cards."""
    diagram = "graph LR\n"
    
    # Nodes
    for card_id, card in cards.items():
        if card["type"] == "driver":
            label = f"{card_id}<br/>{card['name']}"
            diagram += f'  {card_id.replace(":", "_")}["{label}"]\n'
    
    # Edges from relations/links
    for card_id, card in cards.items():
        if card["type"] == "driver":
            for rel_id in card.get("relations", []):
                if rel_id.startswith("driver:"):
                    src = card_id.replace(":", "_")
                    tgt = rel_id.replace(":", "_")
                    diagram += f"  {src} -->|derives| {tgt}\n"
    
    return "```mermaid\n" + diagram + "```\n"
```

### Markdown Rendering

**From a card, generate human-readable Markdown:**

```python
def render_card_as_markdown(card: dict) -> str:
    """Render a card as Markdown documentation."""
    md = f"# {card['name']}\n\n"
    md += f"- **ID**: `{card['id']}`\n"
    md += f"- **Type**: `{card['type']}`\n"
    md += f"- **Status**: `{card['status']}`\n"
    md += f"- **Version**: `{card['version']}`\n\n"
    
    md += f"## Description\n\n{card.get('description', 'N/A')}\n\n"
    
    if card.get("acceptance_criteria"):
        md += "## Acceptance Criteria\n\n"
        for ac in card["acceptance_criteria"]:
            md += f"- {ac}\n"
        md += "\n"
    
    if card.get("audit_history"):
        md += "## Audit History\n\n"
        for event in card["audit_history"]:
            md += f"- {event['event']} — by {event['by']} at {event['event_time']}\n"
        md += "\n"
    
    return md
```

---

## Tooling Expectations (Requirements for Tools)

| Capability | Level | Description |
|-----------|-------|-----------|
| **Validation** | Must | Validate schema, naming, references, audit history. |
| **Loading/Parsing** | Must | Read JSON cards and links; support batching. |
| **Querying** | Should | Graph traversal, filtering, search. |
| **Rendering** | Should | Generate diagrams (Mermaid), markdown, reports. |
| **Change Tracking** | Should | Preserve audit_history; support diffs. |
| **Centered Element View** | Should | Display a card with related cards around it. |
| **Collaboration** | Nice to have | Support change proposals, approvals, workflows. |
| **Metrics** | Nice to have | Traceability coverage, status dashboards. |

---

## Serialization and Data Formats

### Accepted Formats

- **JSON** (required): All canonical data must be JSON.
- **Markdown** (optional): Human-readable views; derived from JSON, not canonical.
- **YAML** (not supported): Per AURORA design, YAML is not accepted for canonical models.

### Import/Export

- **Import**: Accept JSON; validate against schemas.
- **Export**: JSON (canonical), Markdown, SVG, HTML, CSV (reports).

### Interoperability

- **Standard JSON Schema (Draft 7)**: All schemas use standard JSON Schema for compatibility.
- **No proprietary formats**: Avoid vendor-specific extensions or binary formats.

---

## See Also

- [Card Field Reference](card-field-reference.md)
- [Link Types](link-types.md)
- [Conventions and Extensibility](conventions.md)
- [Behavioral Modeling Guide](behavioral-modeling.md)
- [Constraint Modeling Guide](constraint-modeling.md)
- [JSON Schema: card.schema.json](../schemas/card.schema.json)
- [JSON Schema: link.schema.json](../schemas/link.schema.json)
