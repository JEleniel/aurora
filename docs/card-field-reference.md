# AURORA Card Field Reference

This document defines all standard fields available on AURORA cards. For JSON schema validation, see [schemas/card.schema.json](../schemas/card.schema.json).

---

## Core Fields

### `id`

- **Type**: String (required)
- **Description**: Unique identifier for the card using the namespace convention `namespace:element-name`.
- **Example**: `driver:root`, `requirement:automation-pipeline`, `interface:public-api`
- **Notes**: Use lowercase with hyphens; include a namespace to enable tooling to categorize and query cards.

### `type`

- **Type**: String (required)
- **Description**: Card type defining the element's role in the architecture.
- **Valid Values**: `driver`, `requirement`, `behavior`, `interface`, `logical-component`, `deployable-node`, `actor`, `constraint`, `test`, `view`, `artifact`, `note`
- **Example**: `requirement`, `behavior`, `interface`

### `name`

- **Type**: String (required)
- **Description**: Human-friendly title, typically using verb+noun form.
- **Example**: "Ensure Human-Readable Descriptions", "Expose Public API", "Authenticate User"
- **Notes**: Keep concise (< 80 characters); should immediately convey the card's purpose.

### `description`

- **Type**: String (optional, but recommended)
- **Description**: Plain-language explanation of the card's intent, scope, and rationale.
- **Example**: "Provide clear, plain-language descriptions and examples on every card so human readers understand intent and usage without needing tools."
- **Notes**: Aim for 50–200 words; balance detail with readability.

---

## Lifecycle Fields

### `version`

- **Type**: String (optional, recommended)
- **Description**: Semantic version (semver) for the card's content.
- **Format**: `MAJOR.MINOR.PATCH` with optional pre-release and build metadata (e.g., `1.0.0`, `1.1.0-alpha`, `1.0.0+build.123`)
- **Example**: `1.0.0`, `2.1.3-beta`
- **Notes**: Increment when card content changes; use to track evolution of requirements, designs, or constraints.

### `status`

- **Type**: String (optional, default: `proposed`)
- **Description**: Lifecycle state of the card.
- **Valid Values**: `proposed`, `approved`, `implemented`, `verified`, `deprecated`, `retired`
- **Lifecycle Progression**: `proposed` → `approved` → `implemented` → `verified` → (optionally) `deprecated` → `retired`
- **Notes**:
    + `proposed`: Card is under consideration; no commitment yet.
    + `approved`: Decision made to proceed; resources allocated.
    + `implemented`: Card has been realized or addressed in the system.
    + `verified`: Card has passed acceptance criteria and validation.
    + `deprecated`: Card is no longer recommended; support phase-out.
    + `retired`: Card has been removed or superseded.

### `priority`

- **Type**: String (optional)
- **Description**: Card-level priority indicating urgency or importance.
- **Valid Values**: `critical`, `high`, `medium`, `low` (convention; not restricted by schema)
- **Example**: `high`, `medium`

### `owner`

- **Type**: String (optional)
- **Description**: Name, team, or role responsible for the card.
- **Example**: `architecture-team`, `JEleniel`, `identity-services`

---

## Relationship Fields

### `relations`

- **Type**: Array of strings (optional, default: `[]`)
- **Description**: References to related card IDs for simple, untyped associations.
- **Usage**: Use inline relations for quick references when you don't need to capture relationship metadata (rationale, strength).
- **Example**: `["driver:automation", "requirement:automation-pipeline"]`
- **When to Use**: Quick links; many lightweight relationships.

### `links`

- **Type**: Array of strings (optional, default: `[]`)
- **Description**: Explicit link artifact IDs that provide richer metadata (rationale, strength, relationship type).
- **Usage**: Reference explicit link artifacts (stored separately, validated by `schemas/link.schema.json`) when you need to capture _why_ cards are related or the _strength_ of the relationship.
- **Example**: `["link:req-automation-functional-driver-automation"]`
- **When to Use**: Important relationships; traceability requirements; need for rationale or strength metadata.

---

## Content Fields

### `acceptance_criteria`

- **Type**: Array of strings (optional, strongly recommended for requirements)
- **Description**: Machine- or human-verifiable criteria for satisfying a requirement or achieving a goal.
- **Example**:
    + `"Tooling can validate and render any card without human intervention given correct inputs"`
    + `"CI pipelines include schema validation and diagram regeneration steps"`
- **Notes**: Each criterion should be atomic and testable. Use for requirements, behaviors, and constraints.

### `rationale`

- **Type**: String (optional, recommended)
- **Description**: Explanation of why the card exists, how it was derived, or what problem it solves.
- **Example**: "Derived from `driver:automation` to reduce manual work and enable continuous delivery of architecture artifacts."
- **Notes**: Critical for traceability; helps reviewers understand intent and make informed decisions.

### `attributes`

- **Type**: Object (optional)
- **Description**: Flexible key-value metadata for card-specific data. Values may be primitive or structured.
- **Usage**: Capture domain-specific details that don't fit standard fields (e.g., SLA bounds, integration endpoints, state definitions).
- **Example**:

  ```json
  {
    "max_latency_ms": 500,
    "percentile": 99.9,
    "protocol": "HTTP/REST",
    "schema": { "type": "object", "properties": {...} }
  }
  ```

- **Notes**: No schema enforcement on keys or value types; use conventions and document in rationale or description.

### `constraints`

- **Type**: Array of strings (optional)
- **Description**: References to constraint card IDs that limit or govern this card.
- **Example**: `["constraint:latency", "constraint:compliance"]`
- **Usage**: Explicitly link to `constraint`-type cards to capture non-functional requirements (performance, security, compliance).

---

## Audit & Provenance Fields

### `audit_history`

- **Type**: Array of objects (optional, but strongly recommended)
- **Description**: Chronological record of events (creation, updates, approvals) affecting the card.
- **Structure**:

  ```json
  {
    "event": "created|updated|approved|verified|deprecated|retired",
    "by": "username or role",
    "event_time": "ISO 8601 timestamp (e.g., 2025-12-09T10:30:00Z)",
    "note": "optional description of the event"
  }
  ```

- **Required Fields**: `event_time`; other fields are optional but recommended.
- **Example**:

  ```json
  [
    { "event": "created", "by": "JEleniel", "event_time": "2025-12-09T10:00:00Z" },
    { "event": "approved", "by": "architecture-review", "event_time": "2025-12-10T14:30:00Z", "note": "Approved pending implementation planning" },
    { "event": "updated", "by": "JEleniel", "event_time": "2025-12-12T09:15:00Z", "note": "Added acceptance criteria" }
  ]
  ```

- **Notes**: Complete audit trail enables traceability, compliance, and change impact analysis.

### `provenance`

- **Type**: Object (optional)
- **Description**: Metadata about the card's origin, derivation, and ownership.
- **Common Fields**:
    + `source`: Where the card came from (e.g., "derived from drivers", "use-cases", "stakeholder input")
    + `owner`: Primary owner or team (for reference; also see top-level `owner`)
    + `version`: Version of the source from which this card was derived
    + Custom fields: Add as needed (e.g., `generated_at`, `template_id`, `parent_card`)
- **Example**:

  ```json
  {
    "source": "derived from drivers",
    "owner": "JEleniel",
    "version": "1.0",
    "generated_at": "2025-12-09T11:45:36Z"
  }
  ```

### `metadata`

- **Type**: Object (optional)
- **Description**: Serialization and tooling metadata.
- **Standard Fields**:
    + `format`: Serialization format (e.g., `"json"`)
    + `serialization`: Array of supported formats (e.g., `["json"]`)
    + Custom fields: Add any tooling-specific hints (e.g., `generated_by`, `location`, `tags`)
- **Example**:

  ```json
  {
    "serialization": ["json"],
    "generated_by": "tools/generate_docs.py",
    "tags": ["automation", "tooling"]
  }
  ```

---

## Field Guidance by Card Type

### `driver` Cards

**Typically Include**: name, description, version, status, rationale, audit_history
**May Include**: priority, owner, acceptance_criteria, constraints, attributes
**Usually Omit**: acceptance_criteria (drivers are aspirational, not verifiable)

### `requirement` Cards

**Typically Include**: name, description, version, status, acceptance_criteria, rationale, relations/links to drivers, audit_history
**May Include**: priority, owner, provenance, constraints, attributes
**Strongly Recommended**: acceptance_criteria, audit_history

### `behavior` Cards

**Typically Include**: name, description, version, status, attributes (for state, inputs, outputs), audit_history
**May Include**: acceptance_criteria, relations to actors/interfaces, constraints
**Example Attributes**: `{ "inputs": [...], "outputs": [...], "state_machine": {...}, "interactions": [...] }`

### `interface` Cards

**Typically Include**: name, description, version, status, attributes (protocol, schema), audit_history
**May Include**: rationale, owner, constraints (performance, security)
**Example Attributes**: `{ "protocol": "HTTP/REST", "schema": {...}, "authentication": "OAuth2" }`

### `constraint` Cards

**Typically Include**: name, description, version, status, attributes (bounds, units, formulas), audit_history
**May Include**: rationale, relations to requirements/components
**Example Attributes**: `{ "max_latency_ms": 500, "min_throughput_rps": 1000, "compliance_standard": "HIPAA" }`

### `test` Cards

**Typically Include**: name, description, version, status, acceptance_criteria, links to requirements being verified, audit_history
**May Include**: attributes (test type, framework, execution environment)

### `logical-component` & `deployable-node` Cards

**Typically Include**: name, description, version, status, attributes (composition, dependencies), audit_history
**May Include**: constraints, relations to interfaces/behaviors
**Example Attributes**: `{ "components": [...], "interfaces": [...], "capacity": "...", "os": "..." }`

---

## Best Practices

1. **Be Consistent**: Use the same structure and vocabulary across your model; establish conventions.
2. **Prioritize Traceability**: Use `rationale`, `audit_history`, and explicit `links` to preserve intent and change history.
3. **Keep Descriptions Concise**: Aim for 50–200 words; let acceptance criteria capture details.
4. **Use Attributes Wisely**: Capture domain-specific data in attributes; document assumptions in description or rationale.
5. **Version Strategically**: Update `version` when content changes; use semver to signal backward compatibility.
6. **Record Every Change**: Populate `audit_history` to enable impact analysis and compliance audits.
7. **Link Intentionally**: Use inline `relations` for lightweight references; use explicit `links` for important relationships needing rationale.

---

## See Also

- [JSON Schema: card.schema.json](../schemas/card.schema.json)
- [Link Types Reference](link-types.md)
- [Tooling Guide](tooling.md)
