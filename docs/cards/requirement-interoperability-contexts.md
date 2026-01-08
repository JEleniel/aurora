# Provide Structured Contexts

---

- **ID**: `requirement:interoperability-contexts`
- **Type**: `requirement`
- **Version**: `1.0.0`
- **Status**: `proposed`
- **Priority**: `high`
- **Owner**: `integration-team`

## Field Reference

- **id**: Canonical identifier for the card (namespace:type).
- **type**: Card type (driver, requirement, actor, behavior, interface, constraint, link, view, etc.).
- **name**: Human-friendly title using verb+noun where applicable.
- **description**: Plain-language explanation of the card's intent and scope.
- **version**: Semver for the card's content (MAJOR.MINOR.PATCH).
- **status**: Lifecycle state (proposed, accepted, deprecated, retired, etc.).
- **priority**: Optional top-level priority (high/medium/low).
- **owner**: Optional top-level owner or team responsible for the card.
- **relations**: References to other card IDs indicating logical relationships.
- **links**: Explicit link artifact IDs that encode richer relationship metadata.
- **acceptance_criteria**: Machine-or-human-verifiable criteria for satisfying a requirement.
- **rationale**: Why this card exists; derivation or justification.
- **provenance**: Source and origin metadata (source, owner, version).
- **audit_history**: Chronological events describing create/update actions with timestamps.
- **metadata**: Format and serialization metadata for tooling (format, serialization).

## Description

Support namespace-friendly IDs and optional structured contexts to enable integration with external toolchains while keeping JSON as the canonical serialization.

## Acceptance Criteria

- IDs follow the agreed namespacing convention
- Context definitions are available for mappings to external ontologies where needed

## Rationale

Derived from `driver:interoperability` to ease integration while preserving a JSON-first canonical model.

## Provenance

- **source**: `derived from drivers`
- **owner**: `JEleniel`
- **version**: `1.0`

## Audit History

- created — by JEleniel at 2025-12-09T00:00:00Z

## Related

**Relations**: [driver:interoperability](../cards/driver-interoperability.md)
**Links**: [link:req-interoperability-functional-driver-interoperability](../links/link-req-interoperability-functional-driver-interoperability.md)

## Raw JSON

```json
{
  "id": "requirement:interoperability-contexts",
  "type": "requirement",
  "name": "Provide Structured Contexts",
  "description": "Support namespace-friendly IDs and optional structured contexts to enable integration with external toolchains while keeping JSON as the canonical serialization.",
  "classification": "functional",
  "version": "1.0.0",
  "status": "proposed",
  "priority": "high",
  "owner": "integration-team",
  "relations": [
    "driver:interoperability"
  ],
  "links": [
    "link:req-interoperability-functional-driver-interoperability"
  ],
  "acceptance_criteria": [
    "IDs follow the agreed namespacing convention",
    "Context definitions are available for mappings to external ontologies where needed"
  ],
  "rationale": "Derived from `driver:interoperability` to ease integration while preserving a JSON-first canonical model.",
  "audit_history": [
    {
      "event": "created",
      "by": "JEleniel",
      "event_time": "2025-12-09T00:00:00Z"
    }
  ],
  "provenance": {
    "source": "derived from drivers",
    "owner": "JEleniel",
    "version": "1.0"
  },
  "metadata": {
    "format": "aurora-card-v1",
    "serialization": [
      "json"
    ]
  }
}

```
