# Enforce Card-Level Security and Privacy Constraints

---

- **ID**: `driver:security-and-privacy`
- **Type**: `driver`
- **Version**: `1.0.0`
- **Status**: `proposed`

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

Record security, privacy, and safety constraints at the card level and require traceability for compliance-relevant drivers.

## Provenance

- **source**: `project definition`
- **owner**: `JEleniel`
- **version**: `1.0`

## Audit History

- created — by JEleniel at 2025-12-09T00:00:00Z

## Related

**Relations**: [driver:root](../cards/driver-root.md)

## Raw JSON

```json
{
  "id": "driver:security-and-privacy",
  "type": "driver",
  "name": "Enforce Card-Level Security and Privacy Constraints",
  "description": "Record security, privacy, and safety constraints at the card level and require traceability for compliance-relevant drivers.",
  "version": "1.0.0",
  "status": "proposed",
  "attributes": {
    "sensitivity": "medium",
    "compliance_refs": []
  },
  "relations": [
    "driver:root"
  ],
  "audit_history": [
    {
      "event": "created",
      "by": "JEleniel",
      "event_time": "2025-12-09T00:00:00Z"
    }
  ],
  "provenance": {
    "source": "project definition",
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
