# AURORA Naming Conventions and Extensibility

This guide documents how to name AURORA elements, extend the schema with custom types, and follow conventions for consistency.

---

## Naming Conventions

### Card IDs

All card IDs follow the pattern: `namespace:element-name`

#### Namespace

The namespace indicates the card type or category:

| Namespace | Represents | Examples |
|-----------|-----------|----------|
| `driver` | Drivers (high-level goals, concerns) | `driver:root`, `driver:security-and-privacy` |
| `requirement` | Requirements derived from drivers | `requirement:automation-pipeline`, `requirement:latency` |
| `behavior` | Use cases, processes, interactions | `behavior:user-authentication`, `behavior:order-fulfillment` |
| `interface` | System boundaries, exposed APIs, protocols | `interface:public-api`, `interface:events-bus` |
| `constraint` | Non-functional requirements, bounds | `constraint:latency`, `constraint:compliance` |
| `logical-component` | Logical architecture, services, subsystems | `logical-component:api-gateway`, `logical-component:data-store` |
| `deployable-node` | Physical deployment targets, hosts, clusters | `deployable-node:primary-database`, `deployable-node:app-cluster-us-east-1` |
| `actor` | Users, roles, external systems | `actor:end-user`, `actor:system-operator`, `actor:external-partner` |
| `test` | Test cases, acceptance tests, verification methods | `test:latency-measurement`, `test:security-audit` |
| `artifact` | Documents, configurations, code | `artifact:deployment-guide`, `artifact:terraform-config` |
| `view` | Derived views or projections of the model | `view:requirements-traceability`, `view:logical-deployment` |
| `note` | Documentation, notes, comments | `note:architecture-decision`, `note:constraint-group:security` |

#### Element Name

The element name is the local part of the ID, after the colon.

**Rules**:

- Use lowercase letters, numbers, and hyphens only.
- No underscores, spaces, or special characters.
- Use hyphens to separate words: `user-authentication`, not `user_authentication` or `userAuthentication`.
- Be descriptive and concise: `requirement:api-response-time-sla` is better than `requirement:perf-001`.
- Avoid abbreviations unless widely recognized: `requirement:rest-api` is okay; `requirement:rwdms` is not.

**Examples**:

- ✓ `behavior:submit-change-request`
- ✓ `constraint:data-retention-gdpr`
- ✗ `behavior:submitChangeRequest` (camelCase)
- ✗ `constraint:data_retention_gdpr` (underscores)
- ✗ `requirement:req-123` (generic ID; not descriptive)

### Sub-Namespaces

For large models, use dot notation to create sub-namespaces:

| Pattern | Use Case | Example |
|---------|----------|---------|
| `namespace.domain:name` | Domain-specific requirements | `requirement.security:encryption-at-rest`, `requirement.performance:p99-latency` |
| `namespace.aspect:name` | Aspect-specific elements | `constraint.compliance:hipaa-audit-logging`, `interface.api:public-rest-v1` |

**Examples**:

- `requirement.security:mfa-required` — Security-related requirement
- `constraint.reliability:99.95-percent-uptime` — Reliability constraint
- `interface.api:public-v1` — API interface, version 1
- `logical-component.backend:microservice-catalog` — Backend microservice

### Link IDs

Link artifact IDs follow a naming convention to indicate the relationship:

Pattern: `link:source-type-linktype-target-type:source-name:target-name` or shorter variants:

**Long form** (recommended for complex models):

```
link:requirement-satisfies-behavior:automation-pipeline:api-automation
```

**Short form** (acceptable for simple models):

```
link:req-automation-func-driver-automation
```

Use what makes sense for your model; be consistent.

---

## Extending AURORA

### Custom Card Types

AURORA can be extended with custom card types beyond the provided set.

#### Method 1: Use the `note` Type (Simplest)

For temporary or ad-hoc extensions, use `type: "note"`:

```json
{
  "id": "note:design-decision:database-choice",
  "type": "note",
  "name": "Database Technology Decision",
  "description": "Decision to use PostgreSQL over MySQL for ACID guarantees.",
  "attributes": {
    "decision_type": "architecture-decision",
    "alternatives_considered": ["MySQL", "MongoDB"],
    "rationale": "PostgreSQL provides strong ACID guarantees needed for financial transactions."
  }
}
```

#### Method 2: Extend the Schema (Recommended)

Create a custom schema file extending `card.schema.json`:

**`schemas/decision.schema.json`**:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AURORA Decision Card",
  "description": "Records architectural and technical decisions.",
  "allOf": [
    { "$ref": "card.schema.json" },
    {
      "type": "object",
      "properties": {
        "type": { "const": "decision" },
        "attributes": {
          "type": "object",
          "properties": {
            "decision_type": {
              "type": "string",
              "enum": ["architecture", "design", "technology", "process"]
            },
            "status": {
              "type": "string",
              "enum": ["proposed", "accepted", "deprecated", "superseded"]
            },
            "alternatives_considered": {
              "type": "array",
              "items": { "type": "string" }
            },
            "consequences": {
              "type": "array",
              "items": { "type": "string" }
            }
          }
        }
      },
      "required": ["type"]
    }
  ]
}
```

**Card example**:

```json
{
  "id": "decision:database-postgresql",
  "type": "decision",
  "name": "Adopt PostgreSQL",
  "description": "Standardize on PostgreSQL for all relational data.",
  "version": "1.0.0",
  "status": "approved",
  "attributes": {
    "decision_type": "technology",
    "status": "accepted",
    "alternatives_considered": ["MySQL 8.0", "Oracle", "MongoDB"],
    "consequences": [
      "Strong ACID compliance enables reliable financial transactions.",
      "Operational complexity increases; requires expertise in PostgreSQL tuning.",
      "Cost savings vs. Oracle; no licensing fees."
    ]
  },
  "relations": ["requirement:data-consistency", "constraint:compliance-financial"]
}
```

#### Method 3: Create Domain-Specific Card Types

For larger organizations, create specialized card types:

**`schemas/service.schema.json`** (for microservice definitions):

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AURORA Service Card",
  "description": "Defines a microservice including API, ownership, and deployment.",
  "allOf": [
    { "$ref": "card.schema.json" },
    {
      "type": "object",
      "properties": {
        "type": { "const": "service" },
        "attributes": {
          "type": "object",
          "properties": {
            "api_version": { "type": "string" },
            "repository": { "type": "string", "format": "uri" },
            "team": { "type": "string" },
            "dependencies": {
              "type": "array",
              "items": { "type": "string" }
            },
            "deployment_target": { "type": "string" },
            "monitoring_dashboards": {
              "type": "array",
              "items": { "type": "string", "format": "uri" }
            }
          }
        }
      }
    }
  ]
}
```

### Custom Link Types

Add new link types to the `link.schema.json` enum as needed:

**Current**: `satisfies`, `refines`, `depends-on`, `verified-by`, `derives-from`, `related-to`, `is-composed-of`, `aggregates`, `extends`, `implements`, `realizes`, `serves`, `data-flows`, `allocates`, `contains`, `triggers`, `conflicts-with`, `precedes`, `refines-into`

**To add a custom type**, update the schema and document it in `docs/link-types.md`:

```json
{
  "type": {
    "enum": [
      ...existing types...,
      "custom-link-type"
    ]
  }
}
```

**Examples of custom link types you might add**:

- `governs` — A constraint governs a component or behavior.
- `supersedes` — One decision supersedes another.
- `duplicates` — Two cards represent the same thing (merge candidates).
- `blocked-by` — Source is blocked by target (roadmap planning).

### Custom Attributes

The `attributes` object is free-form; add any key-value pairs relevant to your domain:

```json
{
  "id": "logical-component:search-service",
  "type": "logical-component",
  "attributes": {
    "language": "Python",
    "framework": "FastAPI",
    "database": "Elasticsearch",
    "search_types": ["full-text", "faceted", "similarity"],
    "rate_limit_rps": 10000,
    "index_refresh_interval_seconds": 1,
    "replica_shards": 3
  }
}
```

**Best Practice**: Document custom attributes in your model's conventions document or in a schema extension.

---

## Consistency Guidelines

### Terminology

Establish and document key terms used in your model:

**Glossary Example**:

- **Actor**: Human user or external system that interacts with the system.
- **Behavior**: Significant use case, process, or interaction involving one or more actors and system components.
- **Component**: Logical unit of functionality; may span multiple deployment nodes.
- **Deployment**: Physical instantiation of a component on a node or cluster.
- **Constraint**: Non-functional requirement (performance, security, compliance, scalability).
- **Interface**: System boundary where actors or external systems interact; defines protocol and schema.

### Card Templates

Create templates for common card types to ensure consistency:

**Template: `requirement` Card**

```json
{
  "id": "requirement:DOMAIN-TOPIC",
  "type": "requirement",
  "name": "VERB + OBJECT (e.g., 'Ensure Data Encryption')",
  "description": "Plain-language explanation (50-200 words). State the requirement, not the implementation.",
  "version": "1.0.0",
  "status": "proposed",
  "priority": "high|medium|low",
  "owner": "TEAM-OR-PERSON",
  "acceptance_criteria": [
    "Specific, testable criterion 1",
    "Specific, testable criterion 2"
  ],
  "rationale": "Why this requirement exists; derived from DRIVER-ID.",
  "relations": ["driver:PARENT-DRIVER"],
  "constraints": ["constraint:CONSTRAINT-ID"],
  "audit_history": [
    { "event": "created", "by": "AUTHOR", "event_time": "2025-12-09T10:00:00Z" }
  ]
}
```

**Template: `behavior` Card**

```json
{
  "id": "behavior:DOMAIN-ACTION",
  "type": "behavior",
  "name": "VERB + OBJECT + CONTEXT (e.g., 'Authenticate User via API')",
  "description": "What happens; sequence of actor interactions and system responses.",
  "version": "1.0.0",
  "status": "proposed",
  "attributes": {
    "actors": ["actor:ACTOR-ID"],
    "interfaces": ["interface:INTERFACE-ID"],
    "preconditions": ["Precondition 1", "Precondition 2"],
    "main_flow": ["Step 1", "Step 2", "Step 3"],
    "postconditions": ["Result 1", "Result 2"],
    "alternate_flows": {
      "error_case": ["Alt step 1", "Alt step 2"]
    }
  },
  "relations": ["requirement:RELATED-REQUIREMENT"],
  "constraints": ["constraint:CONSTRAINT-ID"]
}
```

### Documentation and Rationale

Always include `rationale` or `description` explaining:

- **What**: What does the card represent?
- **Why**: Why is it important? What problem does it solve?
- **How**: (Optional, for behavior/interface cards) How is it realized or implemented?

### Audit Trails

Record meaningful audit events:

- `created` — Initial creation.
- `approved` — Stakeholder approval.
- `updated` — Content changed; include note explaining what changed.
- `verified` — Verified to be complete or correct.
- `deprecated` — Marked for removal; include sunset date.

**Example**:

```json
"audit_history": [
  { "event": "created", "by": "alice", "event_time": "2025-12-09T10:00:00Z" },
  { "event": "updated", "by": "alice", "event_time": "2025-12-10T14:30:00Z", "note": "Added acceptance criteria based on stakeholder feedback" },
  { "event": "approved", "by": "architecture-board", "event_time": "2025-12-12T09:00:00Z" },
  { "event": "verified", "by": "qa-team", "event_time": "2025-12-15T16:45:00Z", "note": "All acceptance criteria passed in testing" }
]
```

---

## Versioning

### Card Versioning

Use semantic versioning for card content:

| Version | Meaning | Use Case |
|---------|---------|----------|
| `1.0.0` | Initial release; stable | First approved version. |
| `1.1.0` | Minor update; backward compatible | Clarification in description; added optional acceptance criteria. |
| `1.2.0` | Minor update; backward compatible | Added new related cards; expanded attributes. |
| `2.0.0` | Major change; may break compatibility | Fundamental shift in requirement or design (e.g., technology switch). |
| `1.0.0-beta` | Pre-release | Proposal under review. |
| `1.0.0-alpha.1` | Early draft | Work in progress. |

**When to bump version**:

- PATCH: Documentation, examples, typos.
- MINOR: Additions; new acceptance criteria; expanded but backward-compatible changes.
- MAJOR: Removals or breaking changes; fundamental redesigns.

### Schema Versioning

Schemas themselves should also be versioned:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AURORA Card",
  "version": "1.2.0",
  ...
}
```

---

## Tooling Integration

### Validation

Tooling should:

1. Validate all card IDs against naming conventions.
2. Warn on non-conforming IDs.
3. Validate custom types against extension schemas.
4. Enforce required fields (id, type, name, audit_history).

### Generation

Tooling can auto-generate:

- Card ID templates: `driver:` or `requirement.security:`
- Timestamps for audit_history.
- Link IDs based on source, target, and relationship type.

### Linting

Consider a linter that checks:

- All cards have non-empty descriptions and rationale.
- All audit_history entries have event_time.
- No circular dependencies (if applicable).
- Naming convention compliance.

---

## See Also

- [Card Field Reference](card-field-reference.md)
- [Link Types](link-types.md)
- [JSON Schema: card.schema.json](../schemas/card.schema.json)
- [JSON Schema: link.schema.json](../schemas/link.schema.json)
