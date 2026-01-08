# Provide Machine-Verifiable Acceptance Criteria

---

- **ID**: `requirement:testability-acceptance`
- **Type**: `requirement`
- **Version**: `1.0.0`
- **Status**: `proposed`
- **Priority**: `high`
- **Owner**: `qa-team`

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

Requirements must include clear, machine-verifiable acceptance criteria so tests can validate compliance automatically.

## Acceptance Criteria

- Each requirement has at least one acceptance criterion that can be validated by an automated test
- Test artifacts reference the matching requirement IDs

## Rationale

Derived from `driver:testability` to ensure requirements are verifiable and automatable.

## Provenance

- **source**: `derived from drivers`
- **owner**: `JEleniel`
- **version**: `1.0`

## Audit History

- created — by JEleniel at 2025-12-09T00:00:00Z

## Related

**Relations**: [driver:testability](../cards/driver-testability.md)
**Links**: [link:req-testability-functional-driver-testability](../links/link-req-testability-functional-driver-testability.md)

## Raw JSON

```json
{
  "id": "requirement:testability-acceptance",
  "type": "requirement",
  "name": "Provide Machine-Verifiable Acceptance Criteria",
  "description": "Requirements must include clear, machine-verifiable acceptance criteria so tests can validate compliance automatically.",
  "classification": "functional",
  "version": "1.0.0",
  "status": "proposed",
  "priority": "high",
  "owner": "qa-team",
  "relations": [
    "driver:testability"
  ],
  "links": [
    "link:req-testability-functional-driver-testability"
  ],
  "acceptance_criteria": [
    "Each requirement has at least one acceptance criterion that can be validated by an automated test",
    "Test artifacts reference the matching requirement IDs"
  ],
  "rationale": "Derived from `driver:testability` to ensure requirements are verifiable and automatable.",
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
