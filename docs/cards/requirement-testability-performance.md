# Provide Test Execution Performance Targets

---

- **ID**: `requirement:testability-performance`
- **Type**: `requirement`
- **Version**: `1.0.0`
- **Status**: `proposed`
- **Priority**: `medium`
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

Non-functional: test runs for the core model must complete within agreed timeboxes to support fast feedback loops.

## Acceptance Criteria

- Full validation and test generation completes in under 2 minutes for a small project (<=100 cards)
- Test harness provides clear pass/fail diagnostics

## Rationale

Derived from `driver:testability` to keep iteration cycles short.

## Provenance

- **source**: `derived from drivers`
- **owner**: `JEleniel`
- **version**: `1.0`

## Audit History

- created — by JEleniel at 2025-12-09T00:00:00Z

## Related

**Relations**: [driver:testability](../cards/driver-testability.md)
**Links**: [link:req-testability-nf-driver-testability](../links/link-req-testability-nf-driver-testability.md)

## Raw JSON

```json
{
  "id": "requirement:testability-performance",
  "type": "requirement",
  "name": "Provide Test Execution Performance Targets",
  "description": "Non-functional: test runs for the core model must complete within agreed timeboxes to support fast feedback loops.",
  "classification": "non-functional",
  "version": "1.0.0",
  "status": "proposed",
  "priority": "medium",
  "owner": "qa-team",
  "relations": [
    "driver:testability"
  ],
  "links": [
    "link:req-testability-nf-driver-testability"
  ],
  "acceptance_criteria": [
    "Full validation and test generation completes in under 2 minutes for a small project (<=100 cards)",
    "Test harness provides clear pass/fail diagnostics"
  ],
  "rationale": "Derived from `driver:testability` to keep iteration cycles short.",
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
