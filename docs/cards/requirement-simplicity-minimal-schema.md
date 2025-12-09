# Keep Core Schema Minimal

---

- **ID**: `requirement:simplicity-minimal-schema`
- **Type**: `requirement`
- **Version**: `1.0.0`
- **Status**: `proposed`
- **Priority**: `high`
- **Owner**: `arch-team`

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

Limit the core schema surface area to essential fields and compose additional concerns via attributes and references to preserve simplicity.

## Acceptance Criteria

- Core schema contains no more than the agreed essential fields
- Extensions use namespaced attributes and documented extension points

## Rationale

Derived from `driver:simplicity` to keep the model approachable and composable.

## Provenance

- **source**: `derived from drivers`
- **owner**: `JEleniel`
- **version**: `1.0`

## Audit History

- created — by JEleniel at 2025-12-09T00:00:00Z

## Related

**Relations**: [driver:simplicity](../cards/driver-simplicity.md)
**Links**: [link:req-simplicity-functional-driver-simplicity](../links/link-req-simplicity-functional-driver-simplicity.md)

## Raw JSON

```json
{
  "id": "requirement:simplicity-minimal-schema",
  "type": "requirement",
  "name": "Keep Core Schema Minimal",
  "description": "Limit the core schema surface area to essential fields and compose additional concerns via attributes and references to preserve simplicity.",
  "classification": "functional",
  "version": "1.0.0",
  "status": "proposed",
  "priority": "high",
  "owner": "arch-team",
  "relations": [
    "driver:simplicity"
  ],
  "links": [
    "link:req-simplicity-functional-driver-simplicity"
  ],
  "acceptance_criteria": [
    "Core schema contains no more than the agreed essential fields",
    "Extensions use namespaced attributes and documented extension points"
  ],
  "rationale": "Derived from `driver:simplicity` to keep the model approachable and composable.",
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
