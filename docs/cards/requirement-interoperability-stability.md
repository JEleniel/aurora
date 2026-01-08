# Maintain Stable Integration Points

---

- **ID**: `requirement:interoperability-stability`
- **Type**: `requirement`
- **Version**: `1.0.0`
- **Status**: `proposed`
- **Priority**: `medium`
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

Non-functional: integration-related contexts and ID patterns must be stable across minor releases to avoid breaking downstream tools.

## Acceptance Criteria

- Context and ID patterns are documented and versioned
- Breaking changes require major version increments and migration guidance

## Rationale

Derived from `driver:interoperability` to protect downstream integrations.

## Provenance

- **source**: `derived from drivers`
- **owner**: `JEleniel`
- **version**: `1.0`

## Audit History

- created — by JEleniel at 2025-12-09T00:00:00Z

## Related

**Relations**: [driver:interoperability](../cards/driver-interoperability.md)
**Links**: [link:req-interoperability-nf-driver-interoperability](../links/link-req-interoperability-nf-driver-interoperability.md)

## Raw JSON

```json
{
  "id": "requirement:interoperability-stability",
  "type": "requirement",
  "name": "Maintain Stable Integration Points",
  "description": "Non-functional: integration-related contexts and ID patterns must be stable across minor releases to avoid breaking downstream tools.",
  "classification": "non-functional",
  "version": "1.0.0",
  "status": "proposed",
  "priority": "medium",
  "owner": "integration-team",
  "relations": [
    "driver:interoperability"
  ],
  "links": [
    "link:req-interoperability-nf-driver-interoperability"
  ],
  "acceptance_criteria": [
    "Context and ID patterns are documented and versioned",
    "Breaking changes require major version increments and migration guidance"
  ],
  "rationale": "Derived from `driver:interoperability` to protect downstream integrations.",
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
